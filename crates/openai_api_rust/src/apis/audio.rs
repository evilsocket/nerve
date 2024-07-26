// Learn how to turn audio into text.

//! Audio API

use std::fs::File;

use crate::mpart::Mpart as Multipart;
use serde::{Deserialize, Serialize};

use crate::requests::Requests;
use crate::*;

use super::{AUDIO_TRANSCRIPTION_CREATE, AUDIO_TRANSLATIONS_CREATE};

#[derive(Debug)]
pub struct AudioBody {
	/// The audio file to transcribe,
	/// in one of these formats: mp3, mp4, mpeg, mpga, m4a, wav, or webm.
	pub file: File,
	/// ID of the model to use. Only whisper-1 is currently available.
	pub model: String,
	/// An optional text to guide the model's style or continue a previous audio segment.
	/// The prompt should match the audio language.
	pub prompt: Option<String>,
	/// The format of the transcript output, in one of these options: json, text, srt, verbose_json, or vtt.
	pub response_format: Option<String>,
	/// The sampling temperature, between 0 and 1.
	/// Higher values like 0.8 will make the output more random,
	/// while lower values like 0.2 will make it more focused and deterministic. If set to 0,
	/// the model will use log probability to automatically increase the temperature until certain thresholds are hit.
	pub temperature: Option<f32>,
	/// The language of the input audio. Supplying the input language in ISO-639-1 format will improve accuracy and latency.
	/// ISO-639-1: https://en.wikipedia.org/wiki/List_of_ISO_639-1_codes
	pub language: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Audio {
	pub text: Option<String>,
}

pub trait AudioApi {
	/// Transcribes audio into the input language.
	fn audio_transcription_create(&self, audio_body: AudioBody) -> ApiResult<Audio>;
	/// Translates audio into into English.
	fn audio_translation_create(&self, audio_body: AudioBody) -> ApiResult<Audio>;
}

impl AudioApi for OpenAI {
	fn audio_transcription_create(&self, audio_body: AudioBody) -> ApiResult<Audio> {
		let mut send_data = Multipart::new();

		send_data.add_text("model", audio_body.model);
		if let Some(prompt) = audio_body.prompt {
			send_data.add_text("prompt", prompt);
		}
		if let Some(response_format) = audio_body.response_format {
			send_data.add_text("response_format", response_format);
		}
		if let Some(temperature) = audio_body.temperature {
			send_data.add_text("temperature", temperature.to_string());
		}
		if let Some(language) = audio_body.language {
			send_data.add_text("language", language);
		}

		send_data.add_stream("file", audio_body.file, Some("audio.mp3"), None);

		let res = self.post_multipart(AUDIO_TRANSCRIPTION_CREATE, send_data)?;
		let audio: Audio = serde_json::from_value(res.clone()).unwrap();
		Ok(audio)
	}

	fn audio_translation_create(&self, audio_body: AudioBody) -> ApiResult<Audio> {
		let mut send_data = Multipart::new();

		send_data.add_text("model", audio_body.model);
		if let Some(prompt) = audio_body.prompt {
			send_data.add_text("prompt", prompt);
		}
		if let Some(response_format) = audio_body.response_format {
			send_data.add_text("response_format", response_format);
		}
		if let Some(temperature) = audio_body.temperature {
			send_data.add_text("temperature", temperature.to_string());
		}
		if let Some(language) = audio_body.language {
			send_data.add_text("language", language);
		}

		send_data.add_stream("file", audio_body.file, Some("audio.mp3"), None);

		let res = self.post_multipart(AUDIO_TRANSLATIONS_CREATE, send_data)?;
		let audio: Audio = serde_json::from_value(res.clone()).unwrap();
		Ok(audio)
	}
}

#[cfg(test)]
mod tests {
	use std::fs::File;

	use crate::{
		apis::audio::{AudioApi, AudioBody},
		openai::new_test_openai,
	};

	#[test]
	fn test_audio_transcription() {
		let openai = new_test_openai();
		let file = File::open("test_files/audio.mp3").unwrap();
		let multipart = AudioBody {
			file,
			model: "whisper-1".to_string(),
			prompt: None,
			response_format: None,
			temperature: None,
			language: Some("zh".to_string()),
		};
		let rs = openai.audio_transcription_create(multipart);
		let audio = rs.unwrap();
		let text = audio.text.unwrap();
		assert!(text.contains("千里"));
	}

	#[test]
	fn test_audio_translation() {
		let openai = new_test_openai();
		let file = File::open("test_files/audio.mp3").unwrap();
		let multipart = AudioBody {
			file,
			model: "whisper-1".to_string(),
			prompt: None,
			response_format: None,
			temperature: None,
			language: None,
		};
		let rs = openai.audio_translation_create(multipart);
		let audio = rs.unwrap();
		let text = audio.text.unwrap();
		assert!(text.contains("thousands of miles"));
	}
}
