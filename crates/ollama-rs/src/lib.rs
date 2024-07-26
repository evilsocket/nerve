#[cfg(feature = "chat-history")]
use crate::history::WrappedMessageHistory;
use url::Url;

pub mod error;
pub mod generation;
#[cfg(feature = "chat-history")]
pub mod history;
pub mod models;

/// A trait to try to convert some type into a [`Url`].
///
/// This trait is "sealed", such that only types within ollama-rs can
/// implement it.
pub trait IntoUrl: IntoUrlSealed {}

impl IntoUrl for Url {}
impl IntoUrl for String {}
impl<'a> IntoUrl for &'a str {}
impl<'a> IntoUrl for &'a String {}

pub trait IntoUrlSealed {
    fn into_url(self) -> Result<Url, url::ParseError>;

    fn as_str(&self) -> &str;
}

impl IntoUrlSealed for Url {
    fn into_url(self) -> Result<Url, url::ParseError> {
        Ok(self)
    }

    fn as_str(&self) -> &str {
        self.as_str()
    }
}

impl<'a> IntoUrlSealed for &'a str {
    fn into_url(self) -> Result<Url, url::ParseError> {
        Url::parse(self)?.into_url()
    }

    fn as_str(&self) -> &str {
        self
    }
}

impl<'a> IntoUrlSealed for &'a String {
    fn into_url(self) -> Result<Url, url::ParseError> {
        (&**self).into_url()
    }

    fn as_str(&self) -> &str {
        self.as_ref()
    }
}

impl IntoUrlSealed for String {
    fn into_url(self) -> Result<Url, url::ParseError> {
        (&*self).into_url()
    }

    fn as_str(&self) -> &str {
        self.as_ref()
    }
}

#[derive(Debug, Clone)]
pub struct Ollama {
    pub(crate) url: Url,
    pub(crate) reqwest_client: reqwest::Client,
    #[cfg(feature = "chat-history")]
    pub(crate) messages_history: Option<WrappedMessageHistory>,
}

impl Ollama {
    /// # Panics
    ///
    /// Panics if the host is not a valid URL or if the URL cannot have a port.
    pub fn new(host: impl IntoUrl, port: u16) -> Self {
        let mut url: Url = host.into_url().unwrap();
        url.set_port(Some(port)).unwrap();

        Self::from_url(url)
    }

    /// Tries to create new instance by converting `url` into [`Url`].
    #[inline]
    pub fn try_new(url: impl IntoUrl) -> Result<Self, url::ParseError> {
        Ok(Self::from_url(url.into_url()?))
    }

    /// Create new instance from a [`Url`].
    #[inline]
    pub fn from_url(url: Url) -> Self {
        Self {
            url,
            ..Default::default()
        }
    }

    /// Returns the http URI of the Ollama instance
    ///
    /// # Panics
    ///
    /// Panics if the URL does not have a host.
    #[inline]
    pub fn uri(&self) -> String {
        self.url.host().unwrap().to_string()
    }

    /// Returns the URL of the Ollama instance as a [`Url`].
    pub fn url(&self) -> &Url {
        &self.url
    }

    /// Returns the URL of the Ollama instance as a [str].
    ///
    /// Syntax in pseudo-BNF:
    ///
    /// ```bnf
    ///   url = scheme ":" [ hierarchical | non-hierarchical ] [ "?" query ]? [ "#" fragment ]?
    ///   non-hierarchical = non-hierarchical-path
    ///   non-hierarchical-path = /* Does not start with "/" */
    ///   hierarchical = authority? hierarchical-path
    ///   authority = "//" userinfo? host [ ":" port ]?
    ///   userinfo = username [ ":" password ]? "@"
    ///   hierarchical-path = [ "/" path-segment ]+
    /// ```
    #[inline]
    pub fn url_str(&self) -> &str {
        self.url.as_str()
    }
}

impl From<Url> for Ollama {
    fn from(url: Url) -> Self {
        Self::from_url(url)
    }
}

impl Default for Ollama {
    /// Returns a default Ollama instance with the host set to `http://127.0.0.1:11434`.
    fn default() -> Self {
        Self {
            url: Url::parse("http://127.0.0.1:11434").unwrap(),
            reqwest_client: reqwest::Client::new(),
            #[cfg(feature = "chat-history")]
            messages_history: None,
        }
    }
}
