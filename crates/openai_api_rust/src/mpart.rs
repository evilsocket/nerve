use log::debug;
use mime::Mime;
use rand::Rng;
use std::borrow::Cow;
use std::error::Error;
use std::fmt;
use std::io;
use std::io::Cursor;
use std::io::Read;
use std::io::Write;

#[derive(Debug)]
struct Field<'n, 'd> {
	name: Cow<'n, str>,
	data: Data<'n, 'd>,
}

enum Data<'n, 'd> {
	Text(Cow<'d, str>),
	Stream(Stream<'n, 'd>),
}

impl<'n, 'd> fmt::Debug for Data<'n, 'd> {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		match *self {
			Data::Text(ref text) => write!(f, "Data::Text({:?})", text),
			Data::Stream(_) => f.write_str("Data::Stream(Box<Read>)"),
		}
	}
}

struct Stream<'n, 'd> {
	filename: Option<Cow<'n, str>>,
	content_type: Mime,
	stream: Box<dyn Read + 'd>,
}

/// A `LazyError` wrapping `std::io::Error`.
pub type LazyIoError<'a> = LazyError<'a, io::Error>;

/// `Result` type for `LazyIoError`.
pub type LazyIoResult<'a, T> = Result<T, LazyIoError<'a>>;

/// An error for lazily written multipart requests, including the original error as well
/// as the field which caused the error, if applicable.
pub struct LazyError<'a, E> {
	/// The field that caused the error.
	/// If `None`, there was a problem opening the stream to write or finalizing the stream.
	pub field_name: Option<Cow<'a, str>>,
	/// The inner error.
	pub error: E,
	/// Private field for back-compat.
	_priv: (),
}

/// Take `self.error`, discarding `self.field_name`.
impl<'a> Into<io::Error> for LazyError<'a, io::Error> {
	fn into(self) -> io::Error {
		self.error
	}
}

impl<'a, E: Error> fmt::Display for LazyError<'a, E> {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		write!(f, "{}", self.error)
	}
}

impl<'a, E: fmt::Debug> fmt::Debug for LazyError<'a, E> {
	fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
		if let Some(ref field_name) = self.field_name {
			fmt.write_fmt(format_args!("LazyError (on field {:?}): {:?}", field_name, self.error))
		} else {
			fmt.write_fmt(format_args!("LazyError (misc): {:?}", self.error))
		}
	}
}

#[derive(Debug, Default)]
pub struct Mpart<'n, 'd> {
	fields: Vec<Field<'n, 'd>>,
}

impl<'n, 'd> Mpart<'n, 'd> {
	/// Initialize a new lazy dynamic request.
	pub fn new() -> Self {
		Default::default()
	}

	/// Add a text field to this request.
	pub fn add_text<N, T>(&mut self, name: N, text: T) -> &mut Self
	where
		N: Into<Cow<'n, str>>,
		T: Into<Cow<'d, str>>,
	{
		self.fields.push(Field { name: name.into(), data: Data::Text(text.into()) });

		self
	}

	/// Add a generic stream field to this request,
	pub fn add_stream<N, R, F>(
		&mut self,
		name: N,
		stream: R,
		filename: Option<F>,
		mime: Option<Mime>,
	) -> &mut Self
	where
		N: Into<Cow<'n, str>>,
		R: Read + 'd,
		F: Into<Cow<'n, str>>,
	{
		self.fields.push(Field {
			name: name.into(),
			data: Data::Stream(Stream {
				content_type: mime.unwrap_or(mime::APPLICATION_OCTET_STREAM),
				filename: filename.map(|f| f.into()),
				stream: Box::new(stream),
			}),
		});

		self
	}

	/// Export the multipart data contained in this lazy request as an adaptor which implements `Read`.
	///
	/// During this step, if any files were added by path then they will be opened for reading
	/// and their length measured.
	pub fn prepare(&mut self) -> LazyIoResult<'n, PreparedFields<'d>> {
		PreparedFields::from_fields(&mut self.fields)
	}
}

/// The result of [`Multipart::prepare()`](struct.Multipart.html#method.prepare).
///
/// Implements `Read`, contains the entire request body.
///
/// Individual files/streams are dropped as they are read to completion.
///
/// ### Note
/// The fields in the request may have been reordered to simplify the preparation step.
/// No compliant server implementation will be relying on the specific ordering of fields anyways.
pub struct PreparedFields<'d> {
	text_data: Cursor<Vec<u8>>,
	streams: Vec<PreparedField<'d>>,
	end_boundary: Cursor<String>,
}

impl<'d> PreparedFields<'d> {
	fn from_fields<'n>(fields: &mut Vec<Field<'n, 'd>>) -> Result<Self, LazyIoError<'n>> {
		debug!("Field count: {}", fields.len());

		// One of the two RFCs specifies that any bytes before the first boundary are to be
		// ignored anyway
		let mut boundary = format!("\r\n--{}", gen_boundary());

		let mut text_data = Vec::new();
		let mut streams = Vec::new();
		for field in fields.drain(..) {
			match field.data {
				Data::Text(text) => write!(
					text_data,
					"{}\r\nContent-Disposition: form-data; \
                     name=\"{}\"\r\n\r\n{}",
					boundary, field.name, text
				)
				.unwrap(),
				Data::Stream(stream) => {
					streams.push(PreparedField::from_stream(
						&field.name,
						&boundary,
						&stream.content_type,
						stream.filename.as_ref().map(|f| &**f),
						stream.stream,
					));
				},
			}
		}

		// So we don't write a spurious end boundary
		if text_data.is_empty() && streams.is_empty() {
			boundary = String::new();
		} else {
			boundary.push_str("--");
		}

		Ok(PreparedFields {
			text_data: Cursor::new(text_data),
			streams,
			end_boundary: Cursor::new(boundary),
		})
	}

	/// Get the boundary that was used to serialize the request.
	pub fn boundary(&self) -> &str {
		let boundary = self.end_boundary.get_ref();

		// Get just the bare boundary string
		&boundary[4..boundary.len() - 2]
	}
}

impl<'d> PreparedField<'d> {
	fn from_stream(
		name: &str,
		boundary: &str,
		content_type: &Mime,
		filename: Option<&str>,
		stream: Box<dyn Read + 'd>,
	) -> Self {
		let mut header = Vec::new();

		write!(header, "{}\r\nContent-Disposition: form-data; name=\"{}\"", boundary, name)
			.unwrap();

		if let Some(filename) = filename {
			write!(header, "; filename=\"{}\"", filename).unwrap();
		}

		write!(header, "\r\nContent-Type: {}\r\n\r\n", content_type).unwrap();

		PreparedField { header: Cursor::new(header), stream }
	}
}

impl<'d> Read for PreparedFields<'d> {
	fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
		if buf.is_empty() {
			debug!("PreparedFields::read() was passed a zero-sized buffer.");
			return Ok(0);
		}

		let mut total_read = 0;

		while total_read < buf.len() && !cursor_at_end(&self.end_boundary) {
			let buf = &mut buf[total_read..];

			total_read += if !cursor_at_end(&self.text_data) {
				self.text_data.read(buf)?
			} else if let Some(mut field) = self.streams.pop() {
				match field.read(buf) {
					Ok(0) => continue,
					res => {
						self.streams.push(field);
						res
					},
				}?
			} else {
				self.end_boundary.read(buf)?
			};
		}

		Ok(total_read)
	}
}

struct PreparedField<'d> {
	header: Cursor<Vec<u8>>,
	stream: Box<dyn Read + 'd>,
}

impl<'d> Read for PreparedField<'d> {
	fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
		debug!("PreparedField::read()");

		if !cursor_at_end(&self.header) {
			self.header.read(buf)
		} else {
			self.stream.read(buf)
		}
	}
}

impl<'d> fmt::Debug for PreparedField<'d> {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		f.debug_struct("PreparedField")
			.field("header", &self.header)
			.field("stream", &"Box<Read>")
			.finish()
	}
}

fn cursor_at_end<T: AsRef<[u8]>>(cursor: &Cursor<T>) -> bool {
	cursor.position() == (cursor.get_ref().as_ref().len() as u64)
}

fn gen_boundary() -> String {
	const BOUNDARY_LEN: usize = 16;
	//::random_alphanumeric(BOUNDARY_LEN)
	rand::thread_rng()
		.sample_iter(&rand::distributions::Alphanumeric)
		.take(BOUNDARY_LEN)
		.map(|c| c as char)
		.collect()
}
