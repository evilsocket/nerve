use std::collections::HashMap;

use anyhow::Result;
use async_trait::async_trait;
use base64::{engine::general_purpose::STANDARD as b64, Engine as _};
use enigo::{Axis, Button, Direction, Enigo, Key, Keyboard, Mouse};
use xcap::Monitor;

use crate::agent::state::SharedState;

use super::{Action, ActionOutput, Namespace};

// ref: https://arxiv.org/pdf/2411.10323

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
        log::info!("taking screenshot ...");

        let monitors = Monitor::all()?;

        for monitor in monitors {
            // nasty hack to save the image to a PNG tmp file
            let image = monitor.capture_image()?;
            let dimensions = image.dimensions();

            let temp_path = format!("target/temp.png");

            image.save(&temp_path)?;

            let bytes = std::fs::read(&temp_path)?;

            log::info!(
                "taken {}x{} screenshot ({} bytes) from monitor {}",
                dimensions.0,
                dimensions.1,
                bytes.len(),
                monitor.name()
            );

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

#[derive(Debug, Default, Clone)]
struct GetMousePosition {}

#[async_trait]
impl Action for GetMousePosition {
    fn name(&self) -> &str {
        "get_mouse_position"
    }

    fn description(&self) -> &str {
        include_str!("get_mouse_position.prompt")
    }

    async fn run(
        &self,
        _: SharedState,
        _: Option<HashMap<String, String>>,
        _: Option<String>,
    ) -> Result<Option<ActionOutput>> {
        match Enigo::new(&enigo::Settings::default()) {
            Ok(enigo) => {
                let (x, y) = enigo.location()?;
                Ok(Some(ActionOutput::text(format!("({}, {})", x, y))))
            }
            Err(e) => return Err(anyhow!("failed to create enigo: {}", e)),
        }
    }
}

#[derive(Debug, Default, Clone)]
struct MoveMouse {}

#[async_trait]
impl Action for MoveMouse {
    fn name(&self) -> &str {
        "move_mouse"
    }

    fn description(&self) -> &str {
        include_str!("move_mouse.prompt")
    }

    fn example_attributes(&self) -> Option<HashMap<String, String>> {
        let mut attributes = HashMap::new();

        attributes.insert("x".to_string(), "123".to_string());
        attributes.insert("y".to_string(), "456".to_string());

        Some(attributes)
    }

    async fn run(
        &self,
        _: SharedState,
        attributes: Option<HashMap<String, String>>,
        _: Option<String>,
    ) -> Result<Option<ActionOutput>> {
        log::info!("moving mouse to {:?}", attributes);

        let attributes = attributes.unwrap();
        let x: i32 = attributes.get("x").unwrap().parse()?;
        let y: i32 = attributes.get("y").unwrap().parse()?;

        match Enigo::new(&enigo::Settings::default()) {
            Ok(mut enigo) => enigo.move_mouse(x, y, enigo::Coordinate::Abs)?,
            Err(e) => return Err(anyhow!("failed to create enigo: {}", e)),
        }

        Ok(None)
    }
}

#[derive(Debug, Default, Clone)]
struct MoveLeftClick {}

#[async_trait]
impl Action for MoveLeftClick {
    fn name(&self) -> &str {
        "move_left_click"
    }

    fn description(&self) -> &str {
        include_str!("move_left_click.prompt")
    }

    async fn run(
        &self,
        _: SharedState,
        _: Option<HashMap<String, String>>,
        _: Option<String>,
    ) -> Result<Option<ActionOutput>> {
        log::info!("mouse left click");

        match Enigo::new(&enigo::Settings::default()) {
            Ok(mut enigo) => {
                enigo.button(Button::Left, Direction::Click)?;
                Ok(Some(ActionOutput::text(
                    "clicked at current mouse position",
                )))
            }
            Err(e) => return Err(anyhow!("failed to create enigo: {}", e)),
        }
    }
}

#[derive(Debug, Default, Clone)]
struct MoveRightClick {}

#[async_trait]
impl Action for MoveRightClick {
    fn name(&self) -> &str {
        "move_right_click"
    }

    fn description(&self) -> &str {
        include_str!("move_right_click.prompt")
    }

    async fn run(
        &self,
        _: SharedState,
        _: Option<HashMap<String, String>>,
        _: Option<String>,
    ) -> Result<Option<ActionOutput>> {
        log::info!("mouse right click");

        match Enigo::new(&enigo::Settings::default()) {
            Ok(mut enigo) => {
                enigo.button(Button::Right, Direction::Click)?;
                Ok(Some(ActionOutput::text(
                    "clicked at current mouse position",
                )))
            }
            Err(e) => return Err(anyhow!("failed to create enigo: {}", e)),
        }
    }
}

#[derive(Debug, Default, Clone)]
struct MouseVScroll {}

#[async_trait]
impl Action for MouseVScroll {
    fn name(&self) -> &str {
        "mouse_vertical_scroll"
    }

    fn description(&self) -> &str {
        include_str!("mouse_vscroll.prompt")
    }

    fn example_attributes(&self) -> Option<HashMap<String, String>> {
        let mut attributes = HashMap::new();

        attributes.insert("pixels".to_string(), "10".to_string());

        Some(attributes)
    }

    async fn run(
        &self,
        _: SharedState,
        attributes: Option<HashMap<String, String>>,
        _: Option<String>,
    ) -> Result<Option<ActionOutput>> {
        log::info!("vscroll mouse to {:?}", attributes);

        let attributes = attributes.unwrap();
        let pixels: i32 = attributes.get("pixels").unwrap().parse()?;

        match Enigo::new(&enigo::Settings::default()) {
            Ok(mut enigo) => enigo.scroll(pixels, Axis::Vertical)?,
            Err(e) => return Err(anyhow!("failed to create enigo: {}", e)),
        }

        Ok(None)
    }
}

#[derive(Debug, Default, Clone)]
struct MouseHScroll {}

#[async_trait]
impl Action for MouseHScroll {
    fn name(&self) -> &str {
        "mouse_horizontal_scroll"
    }

    fn description(&self) -> &str {
        include_str!("mouse_hscroll.prompt")
    }

    fn example_attributes(&self) -> Option<HashMap<String, String>> {
        let mut attributes = HashMap::new();

        attributes.insert("pixels".to_string(), "10".to_string());

        Some(attributes)
    }

    async fn run(
        &self,
        _: SharedState,
        attributes: Option<HashMap<String, String>>,
        _: Option<String>,
    ) -> Result<Option<ActionOutput>> {
        log::info!("hscroll mouse to {:?}", attributes);

        let attributes = attributes.unwrap();
        let pixels: i32 = attributes.get("pixels").unwrap().parse()?;

        match Enigo::new(&enigo::Settings::default()) {
            Ok(mut enigo) => enigo.scroll(pixels, Axis::Horizontal)?,
            Err(e) => return Err(anyhow!("failed to create enigo: {}", e)),
        }

        Ok(None)
    }
}

#[derive(Debug, Default, Clone)]
struct KeyboardKeyPress {}

#[async_trait]
impl Action for KeyboardKeyPress {
    fn name(&self) -> &str {
        "keyboard_key_press"
    }

    fn description(&self) -> &str {
        include_str!("keyboard_key_press.prompt")
    }

    fn example_attributes(&self) -> Option<HashMap<String, String>> {
        let mut attributes = HashMap::new();

        attributes.insert("key".to_string(), "Control".to_string());

        Some(attributes)
    }

    async fn run(
        &self,
        _: SharedState,
        attributes: Option<HashMap<String, String>>,
        _: Option<String>,
    ) -> Result<Option<ActionOutput>> {
        log::info!("keyboard_key_press {:?}", attributes);

        let attributes = attributes.unwrap();
        let keystroke = attributes.get("key").unwrap();
        let mut to_release: Vec<Key> = Vec::new();

        match Enigo::new(&enigo::Settings::default()) {
            Ok(mut enigo) => {
                for key in keystroke.split("+") {
                    log::info!("pressing key: {:?}", key);

                    let ref_key = key.trim().to_ascii_lowercase();

                    if ref_key == "control" {
                        enigo.key(Key::Control, Direction::Press)?;
                        to_release.push(Key::Control);
                    } else if ref_key == "shift" {
                        enigo.key(Key::Shift, Direction::Press)?;
                        to_release.push(Key::Shift);
                    } else if ref_key == "alt" {
                        enigo.key(Key::Alt, Direction::Press)?;
                        to_release.push(Key::Alt);
                    } else if ref_key == "meta" {
                        enigo.key(Key::Meta, Direction::Press)?;
                        to_release.push(Key::Meta);
                    } else if ref_key == "space" {
                        enigo.key(Key::Space, Direction::Press)?;
                        to_release.push(Key::Space);
                    } else if ref_key == "enter" || ref_key == "return" {
                        enigo.key(Key::Return, Direction::Press)?;
                        to_release.push(Key::Return);
                    } else if key.len() == 1 {
                        enigo.key(Key::Unicode(key.chars().next().unwrap()), Direction::Press)?;
                    } else {
                        enigo.text(key)?;
                    }
                }

                for key in to_release {
                    log::info!("releasing key: {:?}", key);

                    enigo.key(key, Direction::Release)?;
                }
            }
            Err(e) => return Err(anyhow!("failed to create enigo: {}", e)),
        }

        Ok(None)
    }
}

pub fn get_namespace() -> Namespace {
    Namespace::new_non_default(
        "Desktop".to_string(),
        include_str!("ns.prompt").to_string(),
        vec![
            Box::<TakeScreenshot>::default(),
            Box::<MoveMouse>::default(),
            Box::<MoveLeftClick>::default(),
            Box::<MoveRightClick>::default(),
            Box::<MouseVScroll>::default(),
            Box::<MouseHScroll>::default(),
            Box::<GetMousePosition>::default(),
            Box::<KeyboardKeyPress>::default(),
        ],
        None,
    )
}
