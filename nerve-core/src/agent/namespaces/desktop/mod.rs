use std::collections::HashMap;

use anyhow::Result;
use async_trait::async_trait;
use base64::{engine::general_purpose::STANDARD as b64, Engine as _};
use xcap::{image, Monitor};

use crate::agent::state::SharedState;

use super::{Action, ActionOutput, Namespace};

#[derive(Debug, Default, Clone)]
struct TakeScreenshot {}

#[async_trait]
impl Action for TakeScreenshot {
    fn name(&self) -> &str {
        "take_screenshot"
    }

    fn description(&self) -> &str {
        include_str!("take_screenshot.prompt")
    }

    async fn run(
        &self,
        _: SharedState,
        _: Option<HashMap<String, String>>,
        _: Option<String>,
    ) -> Result<Option<ActionOutput>> {
        log::info!("taking screenshot");

        let monitors = Monitor::all()?;

        for monitor in monitors {
            // nasty hack to save the image to a PNG tmp file
            let image = monitor.capture_image()?;

            let temp_path = format!("target/temp.png");

            image.save(&temp_path)?;

            let bytes = std::fs::read(&temp_path)?;

            log::info!("returning {} bytes for screenshot", bytes.len());

            let b64_string = b64.encode(&bytes);

            std::fs::remove_file(&temp_path)?;

            return Ok(Some(ActionOutput::image(
                b64_string,
                "image/png".to_string(),
            )));
        }

        Ok(None)
    }
}

pub fn get_namespace() -> Namespace {
    Namespace::new_non_default(
        "Desktop".to_string(),
        include_str!("ns.prompt").to_string(),
        vec![Box::<TakeScreenshot>::default()],
        None,
    )
}
