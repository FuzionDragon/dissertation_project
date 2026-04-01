use anyhow::{Result, bail};
use clap::Parser;
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    env::{self},
    fs::{self, OpenOptions},
    io::prelude::*,
    process::Command,
};

#[cfg(test)]
mod test {
    use anyhow::bail;

    use super::*;
    const TEST_FILE_PATH: &str = "/tmp/tmp_test_file";
    const TEST_MESSAGE: &str = "TEST";
    const SUCCESS_MESSAGE: &str = "SUCCESS";

    fn create_test_data_path() -> Result<String> {
        let mut exe_path = env::current_exe()?;
        exe_path.pop();

        // temporary, due to rust project file tree
        // would not need back movements
        Ok(format!(
            "{}/../../../tests/test_data.json",
            exe_path.into_os_string().into_string().unwrap()
        ))
        //Ok(format!(
        //    {}/data.json",
        //    exe_path.into_os_string().into_string().unwrap()
        //))
    }

    fn create_test_config_path() -> Result<String> {
        let mut exe_path = env::current_exe()?;
        exe_path.pop();

        // temporary, due to rust project file tree
        // would not need back movements
        Ok(format!(
            "{}/../../../bashrc_custom.bash",
            exe_path.clone().into_os_string().into_string().unwrap()
        ))
        //Ok(format!(
        //    {}/bashrc_custom.bash",
        //    exe_path.clone().into_os_string().into_string().unwrap()
        //))
    }

    #[test]
    fn shell_launches_properly() -> Result<()> {
        fs::File::create(TEST_FILE_PATH)?;
        fs::write(TEST_FILE_PATH, TEST_MESSAGE)?;
        launch_shell(&create_test_config_path()?)?;

        if fs::read_to_string(TEST_FILE_PATH)?
            .strip_suffix("\n")
            .unwrap_or(&fs::read_to_string(TEST_FILE_PATH)?)
            != SUCCESS_MESSAGE
        {
            bail!("Custom shell not launched")
        }
        fs::write(TEST_FILE_PATH, "")?;

        Ok(())
    }

    #[test]
    fn shell_launcher_breaks_on_incorrect_config() -> Result<()> {
        if launch_shell("/incorrect/path").is_ok() {
            bail!("launch_shell doesn't break on providing incorrect path")
        }

        Ok(())
    }

    #[test]
    fn json_handler_new_sucessful() -> Result<()> {
        JsonHandler::new(&create_test_data_path()?)?;

        Ok(())
    }

    #[test]
    fn json_handler_breaks_on_incorrect_file() -> Result<()> {
        if JsonHandler::new("/incorrect/path").is_ok() {
            bail!("JsonHandler::new() doesn't break on providing incorrect file")
        }

        Ok(())
    }

    // json_handler needs more tests, some regarding the fetching of its data and if the function
    // returns accordingly
    //
    // Message also needs testing
}

pub mod args;
pub mod level_checker;
use crate::args::{Args, Commands};
use crate::level_checker::*;

// to pass data from Rust to Bash
enum Message {
    Play(Option<String>, bool),
    End(bool),
}

// issue, whole interface methods require the JSON data, and a mutable form too
// requires changing JSON data at certain points and overriding them (for saving level results and
// changing the current_level value) and also reading the current_level in the first place as a
// fallback for play and also for end level to find the correct hashmap entry to change
impl Message {
    const SELECTED_LEVEL: &str = "SELECTED_LEVEL";
    const END_LEVEL: &str = "END_LEVEL";

    // needs to take in JsonHandler as param
    fn send_message(&self, json_handler: &mut JsonHandler) -> Result<()> {
        fs::File::create(TMP_FILE_PATH)?;
        match self {
            Message::Play(level, interactive) => {
                if interactive.to_owned() {
                    println!("Interative flag raised");
                    // temporary
                    // needs to open interactive level select (with Ratatui)
                    fs::write(TMP_FILE_PATH, format!("{} -1", Self::SELECTED_LEVEL))?;

                    return Ok(());
                }

                if let Some(some_level) = level {
                    println!("Level selected {some_level}");
                    json_handler.save_level_data(some_level.to_owned())?;
                } else {
                    // needs to fetch current_level in json_data
                    println!("Level command ran but without specified level, using current level");
                }

                let current_level = json_handler.current_level()?.unwrap();
                fs::write(
                    TMP_FILE_PATH,
                    format!(
                        "{} {} {}",
                        Self::SELECTED_LEVEL,
                        &json_handler.data.current_level,
                        current_level.level_type.as_str().to_uppercase()
                    ),
                )?;

                //                if selected_level != "-1" || selected_level == json_data.current_level {
                //                    json_data.current_level = selected_level.to_owned();
                //                    let data = serde_json::to_vec_pretty(&json_data)?;
                //                    writeable_json_file.write_all(&data)?;
                //                }
            }

            Message::End(completed) => {
                // needs changes for in_level, not in_level and also with flag
                // could be done manually with flag or dynamically
                let in_level = env::var("IN_LEVEL")?;
                if in_level == "1" {
                    if *completed {
                        println!("End of level, saving stats");
                        // save logic
                    } else {
                        println!("completed ending of level");
                    }

                    fs::write(TMP_FILE_PATH, Self::END_LEVEL)?;
                } else {
                    println!("Not currently in level");
                }
            }
        }

        Ok(())
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct JsonData {
    current_level: String,
    levels: HashMap<String, Level>,
}

#[derive(Clone)]
struct JsonHandler {
    data: JsonData,
    path: String,
}

// allows reading and writing of data
impl JsonHandler {
    fn new(path: &str) -> Result<JsonHandler> {
        let mut read_handle = OpenOptions::new().read(true).open(path)?;
        let mut raw_json = String::new();

        read_handle.read_to_string(&mut raw_json)?;
        let data: JsonData = serde_json::from_str(&raw_json)?;
        let path = path.to_owned();

        Ok(JsonHandler { data, path })
    }

    fn save_level_data(&mut self, selected_level: String) -> Result<()> {
        if !self.data.levels.contains_key(&selected_level) {
            bail!("No entry for entered level")
        }

        self.data.current_level = selected_level;
        let mut write_handle = OpenOptions::new()
            .write(true)
            .append(false)
            .open(&self.path)?;

        write_handle.write_all(&serde_json::to_vec_pretty(&self.data)?)?;

        Ok(())
    }

    fn current_level(&self) -> Result<Option<&Level>> {
        Ok(self.data.levels.get(&self.data.current_level))
    }
}

//const CUSTOM_BASHRC_PATH: &str = "./bashrc_custom.bash";
const TMP_FILE_PATH: &str = "/tmp/tmp_cli_learn";

pub fn init() -> Result<()> {
    let mut json_handler = JsonHandler::new(&create_data_path()?)?;

    let args = Args::parse();
    if let Some(Commands::AllLevels) = &args.command {
        print_all_levels(&json_handler)?;
        return Ok(());
    }
    if let Some(Commands::CurrentLevel) = &args.command {
        print_current_level(&json_handler)?;
        return Ok(());
    }

    if let Some(Commands::Play { level, interactive }) = &args.command {
        Message::Play(level.to_owned(), *interactive).send_message(&mut json_handler)?;
        if let Some(selected_level) = level {
            json_handler.save_level_data(selected_level.to_owned())?;
        }
    }

    // needs to write to a file in /tmp, which will be regularly written to and cleared out in order
    // for the Bash trap to capture and read
    if env::var("APP_ACTIVE").is_err() {
        if let Some(Commands::End { completed: _ }) = args.command {
            println!(
                "Not currently in the learning environment\nUse this command when inside the learning environment and during a level"
            );
            return Ok(());
        }

        launch_shell(&create_config_path()?)?;
    } else {
        println!("App is active");

        // Only triggers if the user has selected a level on initialisation
        if let Some(Commands::End { completed }) = args.command {
            Message::End(completed).send_message(&mut json_handler)?;
        }

        if let Some(level) = json_handler.current_level()?
            && env::var("IN_LEVEL")? == "1"
        {
            println!("in game process");
            // needs more logic
            if level.check()? {
                println!("Level has been completed");
                Message::End(true).send_message(&mut json_handler)?;
            }
            // this else statement is temporary, for debugging and testing
            else {
                println!("Not complete");
            }
        }
    }
    Ok(())
}

fn launch_shell(config_path: &str) -> Result<()> {
    if !fs::exists(config_path)? {
        bail!("The 'bashrc_custom.bash' file is missing not found in the correct directory")
    }
    Command::new("bash")
        .arg("-c")
        .arg(format!("bash --rcfile {}", config_path))
        .spawn()?
        .wait()?;
    Ok(())
}

fn create_data_path() -> Result<String> {
    let mut exe_path = env::current_exe()?;
    exe_path.pop();

    Ok(format!(
        "{}/../../data.json",
        exe_path.into_os_string().into_string().unwrap()
    ))
    //Ok(format!(
    //    {}/data.json",
    //    exe_path.into_os_string().into_string().unwrap()
    //))
}

fn create_config_path() -> Result<String> {
    let mut exe_path = env::current_exe()?;
    exe_path.pop();

    Ok(format!(
        "{}/../../bashrc_custom.bash",
        exe_path.clone().into_os_string().into_string().unwrap()
    ))
    //Ok(format!(
    //    {}/bashrc_custom.bash",
    //    exe_path.clone().into_os_string().into_string().unwrap()
    //))
}

// future function to be added after ensuring level system works
// it allows the user to list all the levels provided by the tool, even highlighting core
// information like scores and if the level had been completed or not.
fn print_all_levels(json_handler: &JsonHandler) -> Result<()> {
    println!("All Levels");
    for level in &json_handler.data.levels {
        level.1.print();
        println!();
    }

    Ok(())
}

// needs to be nicer looking
fn print_current_level(json_handler: &JsonHandler) -> Result<()> {
    if let Some(level) = json_handler.current_level()? {
        println!("Current Level:");
        level.print();
    } else {
        bail!("No current level found")
    }

    Ok(())
}
