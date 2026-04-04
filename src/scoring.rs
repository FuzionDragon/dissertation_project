use std::{env, process::Command};

use anyhow::Result;
const START_TIME: &str = "START_TIME";
const COMMANDS_USED: &str = "NUMBER_OF_USED_COMMANDS";

pub fn calculate_time() -> Result<f32> {
    let start_time = env::var(START_TIME)?.parse::<f32>()?;

    let command_output = Command::new("bash")
        .arg("-c")
        .arg("date +%S.%N")
        .output()?
        .stdout;

    let end_time_raw = str::from_utf8(&command_output)?;
    let end_time = end_time_raw
        .strip_suffix("\n")
        .unwrap_or(end_time_raw)
        .parse::<f32>()?;

    Ok(end_time - start_time)
}

pub fn fetch_command_count() -> Result<i32> {
    let count = env::var(COMMANDS_USED)?.parse::<i32>()?;

    Ok(count)
}
