use anyhow::Result;
use clap::Parser;
use serde::{Deserialize, Serialize};
use std::{
    fs::{self},
    path::Path,
    process::Command,
};

use crate::args::Args;

#[cfg(test)]
mod test {
    use anyhow::bail;

    use super::*;
    const TEST_FILE_PATH: &str = "/tmp/tmp_test_file";
    const TEST_MESSAGE: &str = "TEST";
    const SUCCESS_MESSAGE: &str = "SUCCESS";

    fn create_dummy(level_type: LevelType) -> Result<Level> {
        Ok(Level {
            level_title: "level".into(),
            level_description: "some description".into(),
            level_type,
            highest_score: None,
            shortest_time: None,
            commands_used: None,
        })
    }

    // needs tests for Level and its function level_checker()
    // requires the custom Bash shell to be running
    #[test]
    fn level_type_command_complete() -> Result<()> {
        Ok(())
    }

    #[test]
    fn level_type_command_not_complete() -> Result<()> {
        Ok(())
    }

    #[test]
    fn level_type_file_complete() -> Result<()> {
        fs::File::create(TEST_FILE_PATH)?;
        fs::write(TEST_FILE_PATH, "")?;

        let level: Level = create_dummy(LevelType::File {
            target_file: TEST_FILE_PATH.into(),
            correct_content: None,
        })?;

        if !level.check()? {
            bail!("Level.check() gave false when it should have been true")
        }

        fs::write(TEST_FILE_PATH, TEST_MESSAGE)?;
        let level: Level = create_dummy(LevelType::File {
            target_file: TEST_FILE_PATH.into(),
            correct_content: Some(TEST_MESSAGE.into()),
        })?;

        if !level.check()? {
            bail!("Level.check() gave false when it should have been true")
        }

        Ok(())
    }

    #[test]
    fn level_type_file_not_complete() -> Result<()> {
        let level: Level = create_dummy(LevelType::File {
            target_file: "/incorrect/path".into(),
            correct_content: None,
        })?;

        if level.check()? {
            bail!("Level.check() gave a true when it should have been false")
        }

        Ok(())
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Level {
    pub level_title: String,
    pub level_description: String,
    pub level_type: LevelType,
    pub highest_score: Option<i32>,
    pub shortest_time: Option<f32>,
    pub commands_used: Option<i32>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum LevelType {
    Command {
        checker_command: String,
    },
    File {
        target_file: String,
        correct_content: Option<String>,
    },
    Directory {
        target_directory: String,
        correct_file_tree: Vec<DirectoryItem>,
    },
}

impl Level {
    pub fn check(&self) -> Result<bool> {
        let mut level_complete: bool = false;
        match &self.level_type {
            LevelType::Command { checker_command } => {
                let option_user_command = Args::parse().user_command;
                if let Some(user_command) = option_user_command {
                    // function to check output of the command
                    level_complete = check_command(checker_command, &user_command)?;
                }
            }
            LevelType::File {
                target_file,
                correct_content,
            } => {
                level_complete = check_file(target_file, correct_content)?;
            }
            LevelType::Directory {
                target_directory,
                correct_file_tree,
            } => {
                level_complete =
                    check_directory(target_directory, correct_file_tree.clone().to_vec())?;
            }
        }

        Ok(level_complete)
    }

    pub fn clean_filesystem(&self) -> Result<()> {
        if let LevelType::File {
            target_file,
            correct_content: _,
        } = &self.level_type
        {
            fs::remove_file(target_file)?;
        }

        if let LevelType::Directory {
            target_directory,
            correct_file_tree: _,
        } = &self.level_type
        {
            fs::remove_dir_all(target_directory)?;
        }

        Ok(())
    }

    // needs to be prettier
    pub fn print(&self) {
        println!("Title: {}", self.level_title);
        println!("Description: {}", self.level_description);
        println!("Level Type: {}", self.level_type.as_str());

        if let Some(highscore) = self.highest_score {
            println!("Highscore: {}", highscore);
        } else {
            println!("Highscore: N/A");
        }
        if let Some(shortest_time) = self.shortest_time {
            println!("Shortest Time: {}", shortest_time);
        } else {
            println!("Shortest Time: N/A");
        }
        if let Some(command_count) = self.commands_used {
            println!("Commands Used: {}", command_count);
        } else {
            println!("Commands Used: N/A");
        }
    }
}

impl LevelType {
    pub fn as_str(&self) -> &'static str {
        match self {
            LevelType::Command { checker_command: _ } => "Command",

            LevelType::File {
                target_file: _,
                correct_content: _,
            } => "File",

            LevelType::Directory {
                target_directory: _,
                correct_file_tree: _,
            } => "Directory",
        }
    }
}

// implicit assumption that an item with contents is a file, reguardless if empty or not this field
// will need to be added in the json
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct DirectoryItem {
    name: String,
    content: Option<String>,
    item_type: ItemType,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
enum ItemType {
    File,
    Directory,
}

// requires being in the custom Bash shell for accuracy
fn check_command(checking_command: &str, user_command: &str) -> Result<bool> {
    let user_output = Command::new("bash").arg("-c").arg(user_command).output()?;

    let correct_output = Command::new("bash")
        .arg("-c")
        .arg(checking_command)
        .output()?;

    if correct_output == user_output {
        Ok(true)
    } else {
        Ok(false)
    }
}

// needs to check if content is present, and also check if file exists
// needs to remove the file after check is true
fn check_file(target_file: &str, correct_content: &Option<String>) -> Result<bool> {
    if !fs::exists(target_file)? {
        println!("File not found: {}", target_file);
        return Ok(false);
    }
    if let Some(content) = correct_content {
        println!("content: {:?}", content);
        println!(
            "{}",
            fs::read_to_string(target_file)?
                .strip_suffix("\n")
                .unwrap_or(&fs::read_to_string(target_file)?)
        );

        if fs::read_to_string(target_file)?
            .strip_suffix("\n")
            .unwrap_or(&fs::read_to_string(target_file)?)
            == content
        {
            Ok(true)
        } else {
            println!(
                "File {} found but missing the required content: {}",
                target_file, content
            );
            Ok(false)
        }
    } else {
        Ok(true)
    }
}

// needs to check stuff like correct file tree structure of the directory, and also if the
// contained files have the exact contents
// needs more complex code, one that appends the directory item names onto the target_directory to
// do stuff with it
//
// scrap all of the above, there is now a item_type entry with a corresponding ItemType enum
// use that instead
fn check_directory(target_directory: &str, correct_file_tree: Vec<DirectoryItem>) -> Result<bool> {
    for directory_item in correct_file_tree {
        let item_path = format!("{}/{}", target_directory, &directory_item.name);
        let item = Path::new(&item_path);
        match directory_item.item_type {
            ItemType::File => {
                if !check_file(&item_path, &directory_item.content)? {
                    return Ok(false);
                }
            }
            ItemType::Directory => {
                if !item.is_dir() {
                    println!("Missing directory: {}", item_path);
                    return Ok(false);
                }
            }
        }
    }

    Ok(true)
}
