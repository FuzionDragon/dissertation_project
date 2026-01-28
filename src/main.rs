use clap::Parser;
use std::{
    env::{self},
    process::Command,
};

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
}

fn main() {
    let args = Args::parse();

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
