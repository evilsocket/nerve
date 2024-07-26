use serde::Serialize;

use crate::generation::{
    images::Image,
    options::GenerationOptions,
    parameters::{FormatType, KeepAlive},
};

use super::GenerationContext;

/// A generation request to Ollama.
#[derive(Debug, Clone, Serialize)]
pub struct GenerationRequest {
    #[serde(rename = "model")]
    pub model_name: String,
    pub prompt: String,
    pub images: Vec<Image>,
    pub options: Option<GenerationOptions>,
    pub system: Option<String>,
    pub template: Option<String>,
    pub context: Option<GenerationContext>,
    pub format: Option<FormatType>,
    pub keep_alive: Option<KeepAlive>,
    pub(crate) stream: bool,
}

impl GenerationRequest {
    pub fn new(model_name: String, prompt: String) -> Self {
        Self {
            model_name,
            prompt,
            images: Vec::new(),
            options: None,
            system: None,
            template: None,
            context: None,
            format: None,
            keep_alive: None,
            // Stream value will be overwritten by Ollama::generate_stream() and Ollama::generate() methods
            stream: false,
        }
    }

    /// A list of images to be used with the prompt
    pub fn images(mut self, images: Vec<Image>) -> Self {
        self.images = images;
        self
    }

    /// Add an image to be used with the prompt
    pub fn add_image(mut self, image: Image) -> Self {
        self.images.push(image);
        self
    }

    /// Additional model parameters listed in the documentation for the Modelfile
    pub fn options(mut self, options: GenerationOptions) -> Self {
        self.options = Some(options);
        self
    }

    /// System prompt to (overrides what is defined in the Modelfile)
    pub fn system(mut self, system: String) -> Self {
        self.system = Some(system);
        self
    }

    /// The full prompt or prompt template (overrides what is defined in the Modelfile)
    pub fn template(mut self, template: String) -> Self {
        self.template = Some(template);
        self
    }

    /// The context parameter returned from a previous request to /generate, this can be used to keep a short conversational memory
    pub fn context(mut self, context: GenerationContext) -> Self {
        self.context = Some(context);
        self
    }

    // The format to return a response in. Currently the only accepted value is `json`
    pub fn format(mut self, format: FormatType) -> Self {
        self.format = Some(format);
        self
    }

    /// Used to control how long a model stays loaded in memory, by default models are unloaded after 5 minutes of inactivity
    pub fn keep_alive(mut self, keep_alive: KeepAlive) -> Self {
        self.keep_alive = Some(keep_alive);
        self
    }
}
