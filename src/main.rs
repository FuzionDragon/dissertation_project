use serde;
use serde_json;
use std::{env, process::Command};

const CUSTOM_BASHRC_PATH: &str = "./bashrc_custom.bash";

fn main() {
    match env::var("APP_ACTIVE") {
        Ok(var) => {
            println!("Currently in learning environment");
            println!("{var}");
        }
        Err(e) => {
            println!("{e}");
            Command::new("bash")
                .arg("--rcfile")
                .arg(CUSTOM_BASHRC_PATH)
                .spawn()
                .expect("failed to execute process")
                .wait()
                .expect("failed to wait");
        }
    }
}
