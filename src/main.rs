use anyhow::Result;
use clap::{Parser, Subcommand};
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    env::{self},
    fs::{self, OpenOptions, write},
    io::prelude::*,
    process::Command,
};

#[derive(Serialize, Deserialize, Debug)]
struct JsonData {
    current_level: String,
    levels: HashMap<String, Level>,
}

#[derive(Serialize, Deserialize, Debug)]
struct Level {
    level_title: String,
    level_description: String,
    level_type: LevelType,
    highest_score: Option<i32>,
    shortest_time: Option<f32>,
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
#[derive(Serialize, Deserialize, Debug, Clone)]
struct DirectoryItem {
    name: String,
    content: Option<String>,
}

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
    #[arg(short, long, hide = true)]
    user_command: Option<String>,

    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// selecting level to be played, by default will play the current level
    Play {
        #[arg(short, long)]
        level: Option<String>,

        #[arg(short, long, default_value_t = false)]
        interactive: bool,
    },

    /// Ends the current level early, otherwise does nothing
    End {
        #[arg(short, long, default_value_t = false)]
        early: bool,
    },
}

trait Messenger {
    const SELECTED_LEVEL: &str;
    const END_LEVEL: &str;

    fn send_message(&self) -> Result<()> {
        Ok(())
    }
}

// to pass data from Rust to Bash
enum Message {
    Play(Option<String>, bool),
    End(bool),
}

impl Messenger for Message {
    const SELECTED_LEVEL: &str = "SELECTED_LEVEL";
    const END_LEVEL: &str = "END_LEVEL";

    fn send_message(&self) -> Result<()> {
        match self {
            Message::Play(level, interactive) => {
                println!("Sending selected level");
                let selected_level: &str;
                if interactive.to_owned() {
                    println!("Interative flag raised");
                    // temporary
                    // needs to open interactive level select (with Ratatui)
                    selected_level = "-1";
                } else if let Some(some_level) = level {
                    println!("Level selected {some_level}");
                    selected_level = some_level;
                } else {
                    println!("Level command ran but without specified level");
                    selected_level = "-1";
                }

                let command = Self::SELECTED_LEVEL;
                fs::write(TMP_FILE_PATH, format!("{command} {selected_level}"))?;
            }

            Message::End(early) => {
                if !*early {
                    // save logic
                }

                let command = Self::END_LEVEL;
                fs::write(TMP_FILE_PATH, command)?;
            }
        }

        Ok(())
    }
}

//const CUSTOM_BASHRC_PATH: &str = "./bashrc_custom.bash";
const TMP_FILE_PATH: &str = "/tmp/tmp_cli_learn";

fn main() -> Result<()> {
    let args = Args::parse();
    let mut exe_path = env::current_exe()?;
    exe_path.pop();

    // temporary, due to rust project file tree
    // would not need back movements
    let bashrc_path = format!(
        "{}/../../bashrc_custom.bash",
        exe_path.clone().into_os_string().into_string().unwrap()
    );
    //let bashrc_path = format!(
    //    {}/bashrc_custom.bash",
    //    exe_path.clone().into_os_string().into_string().unwrap()
    //);

    let mut exe_path = env::current_exe()?;
    exe_path.pop();

    // temporary, due to rust project file tree
    // would not need back movements
    let data_path = format!(
        "{}/../../data.json",
        exe_path.into_os_string().into_string().unwrap()
    );
    //let data_path = format!(
    //    {}/data.json",
    //    exe_path.into_os_string().into_string().unwrap()
    //);

    let mut readable_json_file = OpenOptions::new().read(true).open(&data_path)?;
    let mut writeable_json_file = OpenOptions::new()
        .write(true)
        .append(false)
        .open(&data_path)?;

    let mut raw_json = String::new();
    readable_json_file.read_to_string(&mut raw_json)?;
    let mut json_data: JsonData = serde_json::from_str(&raw_json)?;

    // this give key value pair, not jut the value
    //println!("{:?}", &json_data);
    let current_level = json_data.levels.get(&json_data.current_level);

    println!("Current Level: {:?}", current_level);
    // needs to create a file in /tmp, which will be regularly written to and cleared out in order
    // for the Bash trap to capture and read
    if env::var("APP_ACTIVE").is_err() {
        // checks if there is a specified level
        fs::File::create(TMP_FILE_PATH)?;
        match &args.command {
            Some(Commands::Play { level, interactive }) => {
                Message::Play(level.to_owned(), *interactive).send_message()?;
                if let Some(selected_level) = level {
                    json_data.current_level = selected_level.to_owned();
                    let data = serde_json::to_vec_pretty(&json_data)?;
                    writeable_json_file.write_all(&data)?;
                }
            }
            Some(Commands::End { early: _ }) => {
                println!(
                    "Not currently in the learning environment\nUse this command when inside the learning environment and during a level"
                );
                return Ok(());
            }
            None => (),
        }
        let command = format!("bash --rcfile {bashrc_path}");
        Command::new("bash")
            .arg("-c")
            .arg(command)
            .spawn()?
            .wait()?;
    } else {
        println!("App is active");

        println!("main print IN_LEVEL: {}", env::var("IN_LEVEL")?);
        // Only triggers if the user has selected a level on initialisation
        match args.command {
            Some(Commands::Play { level, interactive }) => {
                Message::Play(level.to_owned(), interactive).send_message()?;
                if let Some(selected_level) = level {
                    json_data.current_level = selected_level;
                    let data = serde_json::to_vec_pretty(&json_data)?;
                    writeable_json_file.write_all(&data)?;
                }
            }

            Some(Commands::End { early }) => {
                Message::End(early).send_message()?;
            }

            None => (),
        }
        if let Some(level) = current_level
            && env::var("IN_LEVEL")? == "1"
        {
            process_level(level)?;
        }
    }

    Ok(())
}

fn process_level(current_level: &Level) -> Result<()> {
    println!("in game process");

    let mut level_complete = false;
    match &current_level.level_type {
        LevelType::Command { checker_command } => {
            let option_user_command = Args::parse().user_command;
            if let Some(user_command) = option_user_command {
                // function to check output of the command
                level_complete = check_command_question(checker_command, &user_command)?;
            }
        }
        LevelType::File {
            target_file,
            correct_content,
        } => {
            level_complete = check_file_question(target_file, correct_content)?;
        }
        LevelType::Directory {
            target_directory,
            correct_file_tree,
        } => {
            level_complete =
                check_directory_question(target_directory, correct_file_tree.clone().to_vec())?;
        }
    }

    // needs more logic
    if level_complete {
        println!("Level has been completed");
        complete()?;
    }
    // this else statement is temporary, for debugging and testing
    else {
        println!("Not complete");
    }

    Ok(())
}

fn create_tmp_file() -> Result<()> {
    fs::File::create(TMP_FILE_PATH)?;
    Ok(())
}

// starts processes needed for playing a level, by default would play the 'current_level' set
// otherwise, if a level is provided then it should select that as the current_level and play it
// instead
fn play(level: Option<String>, interactive: bool) -> Result<String> {
    println!("Playing level {:?}", &level);
    // should open interactive level select menu
    if interactive {
        println!("Interative flag raised");
        return Ok("-1".to_owned());
    }

    if env::var("IN_LEVEL").is_ok() {
        if env::var("IN_LEVEL")? == "0" {
            println!("Not in level, changing level");
            fs::write(TMP_FILE_PATH, "IN_LEVEL=1")?;
            //Command::new("bash")
            //    .arg("-c")
            //    .arg("export IN_LEVEL=1")
            //    .status()?;
            //Command::new("bash")
            //    .arg("-c")
            //    .arg("TEST='This is a test'")
            //    .status()?;
        } else {
            // potential room for allowing mid level jumping
            println!("Already in level");
        }

        println!("After setting IN_LEVEL: {}", env::var("IN_LEVEL")?);
    }

    if let Some(level) = level {
        Ok(level)
    } else {
        Ok("-1".to_owned())
    }
}

fn complete() -> Result<()> {
    if env::var("IN_LEVEL")? == "1" {
        Command::new("bash").arg("-c").arg("IN_LEVEL=0").status()?;
    } else {
        println!("Not currently in a level");
    }

    Ok(())
}

// future function to be added after ensuring level system works
// it allows the user to list all the levels provided by the tool, even highlighting core
// information like scores and if the level had been completed or not.
fn print_levels() -> Result<()> {
    Ok(())
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

    //println!("Home directory has {}", &correct_output);
    //println!("User output is {}", &user_output);

    if correct_output == user_output {
        println!("Same out put detected, checker found condition has been met");
    } else {
        println!("Checker condition not met");
    }
}

// must update and serialise the contents of data.json (stored in dedicated .local/share directory)
fn update_user_data() -> Result<()> {
    Ok(())
}

fn check_command_question(checking_command: &str, user_command: &str) -> Result<bool> {
    println!("Checking current command: {user_command}");
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
fn check_file_question(target_file: &str, correct_content: &Option<String>) -> Result<bool> {
    if !fs::exists(target_file)? {
        return Ok(false);
    }
    let user_file_data = fs::read_to_string(target_file)?;

    if let Some(content) = correct_content {
        if user_file_data == *content {
            Ok(true)
        } else {
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
fn check_directory_question(
    target_directory: &str,
    correct_file_tree: Vec<DirectoryItem>,
) -> Result<bool> {
    for directory_item in correct_file_tree {
        let item_path = format!("{}/{}", target_directory, &directory_item.name);
        if !fs::exists(&item_path)? {
            return Ok(false);
        } else if let Some(content) = directory_item.content
            && fs::read_to_string(&item_path)? != content
        {
            return Ok(false);
        }
    }

    Ok(true)
}
