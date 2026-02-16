use anyhow::Result;
use clap::Parser;
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    env::{self},
    fs,
    process::Command,
};

#[derive(Serialize, Deserialize, Debug)]
struct JsonData {
    current_level: i32,
    levels: HashMap<String, Level>,
}

#[derive(Serialize, Deserialize, Debug)]
struct Level {
    level_id: i32,
    level_title: String,
    level_description: String,
    level_type: LevelType,
}

#[derive(Serialize, Deserialize, Debug)]
enum LevelType {
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

// implicit assumption that an item with contents is a file, reguardless if empty or not this field
// will need to be added in the json
#[derive(Serialize, Deserialize, Debug)]
struct DirectoryItem {
    name: String,
    contents: Option<String>,
}

//const CUSTOM_BASHRC_PATH: &str = "./bashrc_custom.bash";

// needs persistent storage of the user data: current level, completed level, score, etc, inside a
// json file, to be modified and fetched on each start and finish of a level.
// the bash env vars will hold temporary data like: current level (so it can be fetched from the
// file like a dictionary), number of commands used (possibly not including certain ones like ls
// and other commands that will be constantly used), other current level details the user will need
// which can save having to fetch the data from the file again.
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// piped user output, to be checked
    #[arg(short, long)]
    user_command: Option<String>,

    /// selecting level to be played
    play: Option<String>,
}

fn main() -> Result<()> {
    let args = Args::parse();
    let mut data_path = env::home_dir().unwrap();
    data_path.push(".local/share/cli_learning_tool/data.json");
    data_path.to_str().unwrap();
    let raw_json = fs::read_to_string(data_path)?;
    let json_data: JsonData = parse_data(&raw_json)?;

    match env::current_exe() {
        Ok(mut exe_path) => {
            exe_path.pop();

            // temporary, due to rust project file tree
            // would not need back movements
            let bashrc_path = format!(
                "{}/../../bashrc_custom.bash",
                exe_path.into_os_string().into_string().unwrap()
            );

            match env::var("APP_ACTIVE") {
                Ok(_v) => {
                    if let Some(user_command) = args.user_command {
                        // function to check output of the command
                        check_home(&user_command);
                    } else {
                        println!("No user output piped");
                    }
                }
                Err(_e) => {
                    println!("Not currently in learning environment, spawning custom Bash session");
                    Command::new("bash")
                        .arg("--rcfile")
                        .arg(bashrc_path)
                        .spawn()
                        .expect("failed to execute process")
                        .wait()
                        .expect("failed to wait");
                }
            }
        }

        Err(e) => println!("failed to get current exe path: {e}"),
    };

    Ok(())
}

fn parse_data(raw_json: &str) -> Result<JsonData, serde_json::Error> {
    let data: JsonData = serde_json::from_str(raw_json)?;

    Ok(data)
}

// different types of checkers: file based, directory based, output based
// file and directory are self explanatory, simply check if they are in their place and the
// contents are correct
// output based will be based on if the user has entered a command and its output matches the one
// required
// minimum recommened commands used for the exercise
//
// lets have a level that asks the user to show their home directory
// needs to check if the command entered by the user is successful, before checking its output and
// comparing it to the checker
//
// this function below is used to show how the checker mechanism will work (for output)
fn check_home(user_command: &str) {
    let home = env::var("HOME").unwrap();
    let list = Command::new("ls")
        .arg(home)
        .output()
        .expect("unable to list home directory");

    let user_command = Command::new("bash")
        .arg("-c")
        .arg(user_command)
        .output()
        .expect("unable to get output from user command");

    let correct_output = String::from_utf8_lossy(&list.stdout);
    let user_output = String::from_utf8_lossy(&user_command.stdout);

    println!("Home directory has {}", &correct_output);
    println!("User output is {}", &user_output);

    if correct_output == user_output {
        println!("Same out put detected, checker found condition has been met");
    } else {
        println!("Checker condition not met");
    }
}

// must fetch and deserialise the contents of data.json (stored in dedicated .local/share directory)
fn fetch_user_data() -> Result<String> {
    Ok(String::new())
}

// must update and serialise the contents of data.json (stored in dedicated .local/share directory)
fn update_user_data() -> Result<()> {
    Ok(())
}

fn check_command_question(checking_command: &str, user_command: &str) -> Result<bool> {
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

fn check_file_question(target_file: &str, correct_content: &str) -> Result<bool> {
    let user_file_data = fs::read_to_string(target_file)?;

    if user_file_data == correct_content {
        Ok(true)
    } else {
        Ok(false)
    }
}

// needs to check stuff like correct file tree structure of the directory, and also if the
// contained files have the exact contents
// needs more complex code, one that appends the directory item names onto the target_directory to
// do stuff with it
fn check_directory_question(
    target_directory: &str,
    correct_file_tree: Vec<DirectoryItem>,
) -> Result<bool> {
    for directory_item in correct_file_tree {
        if !fs::exists(&directory_item.name)? {
            return Ok(false);
        } else if let Some(content) = directory_item.contents
            && fs::read_to_string(&directory_item.name)? != content
        {
            return Ok(false);
        }
    }

    Ok(true)
}
