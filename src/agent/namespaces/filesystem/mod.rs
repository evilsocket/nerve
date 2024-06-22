use std::collections::HashMap;
use std::fs::{self, FileType};
use std::os::unix::fs::{FileTypeExt, PermissionsExt};

use chrono::{DateTime, Local};
use libc::{S_IRGRP, S_IROTH, S_IRUSR, S_IWGRP, S_IWOTH, S_IWUSR, S_IXGRP, S_IXOTH, S_IXUSR};

use anyhow::Result;
use colored::Colorize;

use super::{Action, Namespace};
use crate::agent::state::State;

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

#[derive(Debug, Default)]
struct ReadFolder {}

impl Action for ReadFolder {
    fn name(&self) -> &str {
        "read-folder"
    }

    fn description(&self) -> &str {
        include_str!("read_folder.prompt")
    }

    fn example_payload(&self) -> Option<&str> {
        Some("/path/to/folder")
    }

    fn run(
        &self,
        _state: &State,
        _attributes: Option<HashMap<String, String>>,
        payload: Option<String>,
    ) -> Result<Option<String>> {
        // adapted from https://gist.github.com/mre/91ebb841c34df69671bd117ead621a8b
        let folder = payload.unwrap();
        let ret = fs::read_dir(&folder);
        if let Ok(paths) = ret {
            let mut output = format!("Contents of {} :\n\n", &folder);

            for path in paths {
                if let Ok(entry) = path {
                    let full_path = entry.path().canonicalize().unwrap();
                    let metadata = entry.metadata().unwrap();
                    let size = metadata.len();
                    let modified: DateTime<Local> = DateTime::from(metadata.modified().unwrap());
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
                    eprintln!("ERROR: {:?}", path);
                }
            }

            println!(
                "<{}> {} -> {} bytes",
                self.name().bold(),
                folder.yellow(),
                output.len()
            );

            Ok(Some(output))
        } else {
            eprintln!("<{}> {} -> {:?}", self.name().bold(), folder.red(), &ret);
            Err(anyhow!("can't read {}: {:?}", folder, ret))
        }
    }
}

#[derive(Debug, Default)]
struct ReadFile {}

impl Action for ReadFile {
    fn name(&self) -> &str {
        "read-file"
    }

    fn description(&self) -> &str {
        include_str!("read_file.prompt")
    }

    fn example_payload(&self) -> Option<&str> {
        Some("/path/to/file/to/read")
    }

    fn run(
        &self,
        _state: &State,
        _attributes: Option<HashMap<String, String>>,
        payload: Option<String>,
    ) -> Result<Option<String>> {
        let filepath = payload.unwrap();
        let ret = std::fs::read_to_string(&filepath);
        if let Ok(contents) = ret {
            println!(
                "<{}> {} -> {} bytes",
                self.name().bold(),
                filepath.yellow(),
                contents.len()
            );
            Ok(Some(contents))
        } else {
            let err = ret.err().unwrap();
            println!(
                "<{}> {} -> {:?}",
                self.name().bold(),
                filepath.yellow(),
                &err
            );

            Err(anyhow!(err))
        }
    }
}

pub(crate) fn get_namespace() -> Namespace {
    Namespace::new_non_default(
        "Filesystem".to_string(),
        include_str!("ns.prompt").to_string(),
        vec![Box::<ReadFile>::default(), Box::<ReadFolder>::default()],
        None,
    )
}
