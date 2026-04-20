use anyhow::{Ok, Result, bail};
use clap::Parser;
use colored::Colorize;
use serde::{Deserialize, Serialize};
use std::{
    collections::{self, HashMap},
    env::{self},
    fs::{self, OpenOptions},
    io::{self, prelude::*},
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

        if fs::read_to_string(TEST_FILE_PATH)?.trim() != SUCCESS_MESSAGE {
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
        if JsonHandler::new(&create_test_data_path()?).is_err() {
            bail!("JsonHandler::new() breaks when providing correct file")
        };

        Ok(())
    }

    #[test]
    fn json_handler_breaks_on_incorrect_file() -> Result<()> {
        if JsonHandler::new("/incorrect/path").is_ok() {
            bail!("JsonHandler::new() doesn't break on providing incorrect file")
        }

        Ok(())
    }

    #[test]
    fn json_handler_saves_correctly() -> Result<()> {
        let json_handler = JsonHandler::new(&create_test_data_path()?)?;
        let data = json_handler.data.clone();
        json_handler.save()?;

        let saved_data = JsonHandler::new(&create_test_data_path()?);

        if saved_data.is_err() {
            bail!("JsonHandler::new() breaks after using save")
        }

        if data != saved_data.unwrap().data {
            bail!("Saved data doesn't match with exact data saved")
        }

        Ok(())
    }

    #[test]
    fn json_handler_fetches_current_level_correctly() -> Result<()> {
        let json_handler = JsonHandler::new(&create_test_data_path()?)?;
        let current_level = json_handler
            .data
            .levels
            .get(&json_handler.data.current_level);
        let fetched_level = json_handler.current_level()?;

        if current_level != fetched_level {
            bail!("json_handler.current_level() didn't fetch correct level")
        }

        Ok(())
    }

    #[test]
    fn json_handler_updates_current_level_correctly() -> Result<()> {
        let mut json_handler = JsonHandler::new(&create_test_data_path()?)?;
        let mut current_level = json_handler.current_level()?.unwrap().clone();
        current_level.highest_score = Some(20);
        current_level.shortest_time = Some(3.5);
        current_level.commands_used = Some(2);
        current_level.rank = Some(Rank::Gold);
        json_handler.update_current_level(current_level.clone())?;
        let updated_level = json_handler.current_level()?.unwrap().clone();

        if current_level != updated_level {
            bail!("Updated level doesn't match with edited level")
        }

        Ok(())
    }

    #[test]
    fn json_handler_sets_current_level_correctly() -> Result<()> {
        let mut json_handler = JsonHandler::new(&create_test_data_path()?)?;
        let selected_level = "1";

        json_handler.set_current_level(selected_level.to_string())?;
        let new_json_handler = JsonHandler::new(&create_test_data_path()?)?;

        if selected_level != new_json_handler.data.current_level {
            bail!("Current level not set correctly")
        }

        Ok(())
    }

    // Message also needs testing
}

mod args;
mod interactive;
mod level_checker;
mod scoring;
use crate::args::{Args, Commands};
use crate::level_checker::*;
use crate::scoring::Rank;

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

    fn send_message(&self, json_handler: &mut JsonHandler) -> Result<()> {
        fs::File::create(TMP_FILE_PATH)?;
        match self {
            Message::Play(level, interactive) => {
                if interactive.to_owned() {
                    let id = interactive::level_select_tui(&json_handler.data.levels)?;
                    let temp_handler = json_handler.clone();
                    let selected_level = temp_handler.data.levels.get(&id.to_string());
                    if let Some(some_level) = selected_level {
                        json_handler.set_current_level(id.to_string())?;
                        fs::write(
                            TMP_FILE_PATH,
                            format!(
                                "{} {} {}",
                                Self::SELECTED_LEVEL,
                                id,
                                some_level.level_type.as_str().to_uppercase()
                            ),
                        )?;
                    } else {
                        println!("No level selected");
                    }

                    return Ok(());
                }

                if let Some(some_level) = level {
                    println!("Level selected {some_level}");
                    json_handler.set_current_level(some_level.to_owned())?;
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
            }

            Message::End(completed) => {
                let in_level = env::var("IN_LEVEL")?;
                if in_level == "1" {
                    if *completed {
                        let time = scoring::calculate_time()?;
                        let command_count = scoring::fetch_command_count()?;
                        let score = scoring::calculate_score(time, command_count)?;
                        let rank = scoring::assign_rank(score)?;

                        println!("{}", "Level Stats:".bold());
                        println!("{}: {}", "Score".bold(), score);
                        println!("{}: {}", "Time".bold(), time);
                        println!("{}: {}", "Command Count".bold(), command_count);

                        let rank_string = match rank {
                            Rank::Gold => rank.as_str().yellow(),
                            Rank::Silver => rank.as_str().white(),
                            Rank::Bronze => rank.as_str().red(),
                        };
                        println!("{}: {}", "Rank".bold(), rank_string);

                        let mut current_level = json_handler.current_level()?.unwrap().to_owned();

                        if current_level.shortest_time.is_none()
                            || current_level.shortest_time > Some(time) && time > 0.
                        {
                            current_level.shortest_time = Some(time);
                        }

                        if current_level.commands_used.is_none()
                            || current_level.commands_used > Some(command_count)
                        {
                            current_level.commands_used = Some(command_count);
                        }

                        if current_level.highest_score.is_none()
                            || current_level.highest_score < Some(score) && score > 0
                        {
                            current_level.highest_score = Some(score);
                            current_level.rank = Some(rank);
                        }

                        if score <= 0 {
                            println!(
                                "{}", "Error with score calculations, stats not saved. Please retry the level again.".red().bold()
                            );
                        }

                        json_handler.update_current_level(current_level)?;
                        let stdin = io::stdin();
                        let input = &mut String::new();
                        println!("Want to set current level to the next one? (y/N)");
                        input.clear();
                        stdin.read_line(input)?;
                        if input.trim_end() == "y" {
                            let new_level = json_handler
                                .data
                                .current_level
                                .clone()
                                .parse::<i32>()
                                .unwrap()
                                + 1;
                            let res = json_handler.set_current_level(new_level.to_string());
                            if res.is_err() {
                                println!(
                                    "{}",
                                    "Congratulations! You have completed the last level!"
                                        .bold()
                                        .yellow()
                                );
                            } else {
                                println!("Set current level to be the next");
                                println!(
                                    "Enter {} to play the next level.",
                                    "learn_cli play".bold()
                                );
                            }
                        } else {
                            println!(
                                "Enter {} to replay the current level.",
                                "learn_cli play".bold()
                            );
                        }
                    } else {
                        println!("{}", "Level ending early".bold());
                    }

                    fs::write(TMP_FILE_PATH, Self::END_LEVEL)?;
                } else {
                    println!("{}", "Not currently in level".red().bold());
                }
            }
        }

        Ok(())
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
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

        //println!("Json Data: {:?}", data);
        Ok(JsonHandler { data, path })
    }

    fn set_current_level(&mut self, selected_level: String) -> Result<()> {
        if !self.data.levels.contains_key(&selected_level) {
            bail!("No entry for entered level")
        }

        self.data.current_level = selected_level;
        self.save()?;

        Ok(())
    }

    fn update_current_level(&mut self, new_level_data: Level) -> Result<()> {
        self.data
            .levels
            .entry(self.data.current_level.clone())
            .and_modify(|l| *l = new_level_data);

        self.save()?;

        Ok(())
    }

    fn restart_data(&mut self) -> Result<()> {
        for (_, level) in self.data.levels.iter_mut() {
            level.highest_score = None;
            level.shortest_time = None;
            level.commands_used = None;
            level.rank = None;
        }

        self.set_current_level(0.to_string())?;

        self.save()?;

        Ok(())
    }

    fn current_level(&self) -> Result<Option<&Level>> {
        Ok(self.data.levels.get(&self.data.current_level))
    }

    fn save(&self) -> Result<()> {
        let mut write_handle = OpenOptions::new()
            .write(true)
            .append(false)
            .truncate(true)
            .open(&self.path)?;

        write_handle.write_all(&serde_json::to_vec_pretty(&self.data)?)?;

        Ok(())
    }
}

//const CUSTOM_BASHRC_PATH: &str = "./bashrc_custom.bash";
const TMP_FILE_PATH: &str = "/tmp/tmp_cli_learn";

pub fn init() -> Result<()> {
    let mut json_handler = JsonHandler::new(&create_data_path()?)?;

    let args = Args::parse();

    if let Some(Commands::Restart) = &args.command {
        let stdin = io::stdin();
        let input = &mut String::new();
        println!(
            "Are you sure you want to restart all progress? You will have to start from the beginning. (y/N)"
        );
        input.clear();
        stdin.read_line(input)?;
        if input.trim_end() == "y" {
            println!("Restarting Progress");
            json_handler.restart_data()?;
        }

        return Ok(());
    }

    if let Some(Commands::All) = &args.command {
        print_all_levels(&json_handler)?;
        return Ok(());
    }
    if let Some(Commands::Current { short }) = &args.command {
        if *short {
            if let Some(level) = json_handler.current_level()? {
                level.print_essential();
                println!();
            } else {
                bail!("No current level found")
            }
        } else {
            print_current_level(&json_handler)?;
        }
        return Ok(());
    }

    if let Some(Commands::Play { level, interactive }) = &args.command {
        Message::Play(level.to_owned(), *interactive).send_message(&mut json_handler)?;
        if let Some(selected_level) = level {
            json_handler.set_current_level(selected_level.to_owned())?;
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
        // Only triggers if the user has selected a level on initialisation
        if let Some(Commands::End { completed }) = args.command {
            Message::End(completed).send_message(&mut json_handler)?;
        }

        if let Some(level) = json_handler.clone().current_level()?
            && env::var("IN_LEVEL")? == "1"
            && level.check()?
        {
            println!("Level has been completed");
            Message::End(true).send_message(&mut json_handler)?;
            level.clean_filesystem()?;
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

    //Ok(format!(
    //    "{}/../../data.json",
    //    exe_path.into_os_string().into_string().unwrap()
    //))
    Ok(format!(
        "{}/data.json",
        exe_path.into_os_string().into_string().unwrap()
    ))
}

fn create_config_path() -> Result<String> {
    let mut exe_path = env::current_exe()?;
    exe_path.pop();

    //Ok(format!(
    //    "{}/../../bashrc_custom.bash",
    //    exe_path.clone().into_os_string().into_string().unwrap()
    //))
    Ok(format!(
        "{}/bashrc_custom.bash",
        exe_path.clone().into_os_string().into_string().unwrap()
    ))
}

// future function to be added after ensuring level system works
// it allows the user to list all the levels provided by the tool, even highlighting core
// information like scores and if the level had been completed or not.
fn print_all_levels(json_handler: &JsonHandler) -> Result<()> {
    println!("{}", "All Levels".bold().underline());
    let level_btree = collections::BTreeMap::from_iter(json_handler.data.levels.clone());
    for level in &level_btree {
        if level.0 == &json_handler.data.current_level {
            println!(
                "{}: {} {}",
                "Level ID".bold(),
                level.0.bright_green(),
                "(Current Level)".bold()
            );
        } else {
            println!("{}: {}", "Level ID".bold(), level.0.bright_green());
        }
        level.1.print();
        println!();
    }

    Ok(())
}

// needs to be nicer looking
fn print_current_level(json_handler: &JsonHandler) -> Result<()> {
    if let Some(level) = json_handler.current_level()? {
        println!(
            "{}: {}",
            "Current Level ID".bold(),
            json_handler.data.current_level
        );
        level.print();
    } else {
        bail!("No current level found")
    }

    Ok(())
}
