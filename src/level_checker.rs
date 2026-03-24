use anyhow::Result;
use std::{
    env::{self},
    fs::{self},
    process::Command,
};

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
pub fn check_home(user_command: &str) {
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

pub fn check_command_question(checking_command: &str, user_command: &str) -> Result<bool> {
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
pub fn check_file_question(target_file: &str, correct_content: &Option<String>) -> Result<bool> {
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
