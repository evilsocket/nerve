use std::collections::HashMap;
use std::fs::{self, FileType, OpenOptions};
use std::io::Write;
use std::os::unix::fs::{FileTypeExt, PermissionsExt};

use async_trait::async_trait;
use chrono::{DateTime, Local};
use libc::{S_IRGRP, S_IROTH, S_IRUSR, S_IWGRP, S_IWOTH, S_IWUSR, S_IXGRP, S_IXOTH, S_IXUSR};

use anyhow::Result;
use serde::Serialize;

use super::{Tool, ToolOutput, Namespace};
use crate::agent::state::SharedState;
use crate::agent::task::variables::get_variable;

// cast needed for Darwin apparently
#[allow(clippy::unnecessary_cast)]
fn parse_permissions(mode: u32) -> String {
    let user = triplet(mode, S_IRUSR as u32, S_IWUSR as u32, S_IXUSR as u32);
    let group = triplet(mode, S_IRGRP as u32, S_IWGRP as u32, S_IXGRP as u32);
    let other = triplet(mode, S_IROTH as u32, S_IWOTH as u32, S_IXOTH as u32);
    [user, group, other].join("")
}

fn parse_type(file_type: FileType) -> String {
    if file_type.is_symlink() {
        "symlink"
    } else if file_type.is_dir() {
        "dir"
    } else if file_type.is_block_device() {
        "block device"
    } else if file_type.is_char_device() {
        "char device"
    } else if file_type.is_fifo() {
        "fifo"
    } else if file_type.is_socket() {
        "socket"
    } else if file_type.is_file() {
        "file"
    } else {
        "unknown"
    }
    .to_string()
}

fn triplet(mode: u32, read: u32, write: u32, execute: u32) -> String {
    match (mode & read, mode & write, mode & execute) {
        (0, 0, 0) => "---",
        (_, 0, 0) => "r--",
        (0, _, 0) => "-w-",
        (0, 0, _) => "--x",
        (_, 0, _) => "r-x",
        (_, _, 0) => "rw-",
        (0, _, _) => "-wx",
        (_, _, _) => "rwx",
    }
    .to_string()
}

#[derive(Debug, Default, Clone)]
struct ReadFolder {}

#[async_trait]
impl Tool for ReadFolder {
    fn name(&self) -> &str {
        "list_folder_contents"
    }

    fn description(&self) -> &str {
        include_str!("read_folder.prompt")
    }

    fn example_payload(&self) -> Option<&str> {
        Some("/path/to/folder")
    }

    async fn run(
        &self,
        _: SharedState,
        _: Option<HashMap<String, String>>,
        payload: Option<String>,
    ) -> Result<Option<ToolOutput>> {
        // adapted from https://gist.github.com/mre/91ebb841c34df69671bd117ead621a8b
        let folder = payload.unwrap();
        let ret = fs::read_dir(&folder);
        if let Ok(paths) = ret {
            let mut output = format!("Contents of {} :\n\n", &folder);

            for path in paths {
                if let Ok(entry) = path {
                    let full = entry.path().canonicalize();
                    if let Ok(full_path) = full {
                        let metadata = entry.metadata().unwrap();
                        let size = metadata.len();
                        let modified: DateTime<Local> =
                            DateTime::from(metadata.modified().unwrap());
                        let mode = metadata.permissions().mode();

                        output += &format!(
                            "{} {:>5} {} [{}] {}\n",
                            parse_permissions(mode),
                            size,
                            modified.format("%_d %b %H:%M"),
                            parse_type(metadata.file_type()),
                            full_path.display()
                        );
                    } else {
                        log::error!("can't canonicalize {:?}: {:?}", entry, full.err());
                    }
                } else {
                    log::error!("{:?}", path);
                }
            }

            Ok(Some(output.into()))
        } else {
            Err(anyhow!("can't read {}: {:?}", folder, ret))
        }
    }
}

#[derive(Debug, Default, Clone)]
struct ReadFile {}

#[async_trait]
impl Tool for ReadFile {
    fn name(&self) -> &str {
        "read_file"
    }

    fn description(&self) -> &str {
        include_str!("read_file.prompt")
    }

    fn example_payload(&self) -> Option<&str> {
        Some("/path/to/file/to/read")
    }

    async fn run(
        &self,
        _: SharedState,
        _: Option<HashMap<String, String>>,
        payload: Option<String>,
    ) -> Result<Option<ToolOutput>> {
        let filepath = payload.unwrap();
        let ret = std::fs::read_to_string(&filepath);
        if let Ok(contents) = ret {
            Ok(Some(contents.into()))
        } else {
            let err = ret.err().unwrap();
            Err(anyhow!(err))
        }
    }
}

const DEFAULT_APPEND_TO_FILE_TARGET: &str = "findings.jsonl";

#[derive(Debug, Default, Serialize)]
struct InvalidJSON {
    data: String,
}

#[derive(Debug, Default, Clone)]
struct AppendToFile {}

#[async_trait]
impl Tool for AppendToFile {
    fn name(&self) -> &str {
        "append_to_file"
    }

    fn description(&self) -> &str {
        include_str!("append_to_file.prompt")
    }

    fn example_payload(&self) -> Option<&str> {
        Some(
            r#"{
      "title": "Example title",
      "description": "Example description.",
    }"#,
        )
    }

    async fn run(
        &self,
        _: SharedState,
        _: Option<HashMap<String, String>>,
        payload: Option<String>,
    ) -> Result<Option<ToolOutput>> {
        let payload = payload.unwrap();

        let filepath = match get_variable("filesystem.append_to_file.target") {
            Some(filepath) => filepath,
            None => {
                log::warn!(
                    "filesystem.append_to_file.target not defined, using default {}",
                    DEFAULT_APPEND_TO_FILE_TARGET
                );
                DEFAULT_APPEND_TO_FILE_TARGET.to_string()
            }
        };

        // get lowercase file extension from filepath
        let extension = filepath.rsplit('.').next().unwrap_or("").to_lowercase();

        let content_to_append = if extension == "json" || extension == "jsonl" {
            // parse the payload as a JSON object
            let parsed = serde_json::from_str::<serde_json::Value>(&payload);
            if let Ok(value) = parsed {
                // reconvert to make sure it's on a single line
                serde_json::to_string(&value).unwrap()
            } else {
                log::error!(
                    "can't parse payload as JSON: {} - {}",
                    parsed.err().unwrap(),
                    payload
                );
                serde_json::to_string(&InvalidJSON { data: payload }).unwrap()
            }
        } else {
            // add as it is
            payload
        };

        // append the JSON to the file
        let mut file = OpenOptions::new()
            .append(true)
            .create(true)
            .open(&filepath)?;

        writeln!(file, "{}", content_to_append)?;

        Ok(None)
    }
}

pub fn get_namespace() -> Namespace {
    Namespace::new_non_default(
        "Filesystem".to_string(),
        include_str!("ns.prompt").to_string(),
        vec![
            Box::<ReadFile>::default(),
            Box::<ReadFolder>::default(),
            Box::<AppendToFile>::default(),
        ],
        None,
    )
}
