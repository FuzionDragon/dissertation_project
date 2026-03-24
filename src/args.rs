use clap::{Parser, Subcommand};

// needs persistent storage of the user data: current level, completed level, score, etc, inside a
// json file, to be modified and fetched on each start and finish of a level.
// the bash env vars will hold temporary data like: current level (so it can be fetched from the
// file like a dictionary), number of commands used (possibly not including certain ones like ls
// and other commands that will be constantly used), other current level details the user will need
// which can save having to fetch the data from the file again.
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
pub struct Args {
    /// piped user output, to be checked
    #[arg(short, long, hide = true)]
    pub user_command: Option<String>,

    #[command(subcommand)]
    pub command: Option<Commands>,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
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
