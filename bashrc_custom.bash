#
# ./bashrc_custom.bash
#

# If not running interactively, don't do anything
# This is a custom bashrc, use rcfile option of Bash
[[ $- != *i* ]] && return

# Learning tool logic
export APP_ACTIVE=1
export IN_LEVEL=0
export LEVEL_TYPE=""
export CURRENT_LEVEL=-1
export TMP_FILE="/tmp/tmp_cli_learn"
export TEST_FILE="/tmp/tmp_test_file"

# temporary, should be executable path in the future
export PROJECT_PATH="$(pwd)/Cargo.toml"
export BINARY_PATH="$(pwd)/learn_cli"

# temporary, needs to be location of binary during distribution
#alias learn_cli='cargo run --manifest-path $PROJECT_PATH'
#alias check='cargo run --manifest-path $PROJECT_PATH --  --user-command '

# Actual aliases
alias learn_cli='$BINARY_PATH'
alias check='$BINARY_PATH --user-command'

alias ls='ls --color=auto'
alias grep='grep --color=auto'
PS1='[\u@\h \W]\$ '

export NUMBER_OF_USED_COMMANDS=0
export START_TIME=""
export END_TIME=""

echo "Welcome to the learning environment"
echo
echo -e "To get started, simply enter \033[3mlearn_cli play\033[0m into the command line to start the first level"
echo
echo -e "Run \033[3mlearn_cli help\033[0m to learn how to use the learning tool command and its subcommands"
echo
echo -e "To close the session, run the \033[3mexit\033[0m command"
echo

# used for reading the temp file contents, and acting on certain instructions it states
env_listener () {
  if [[ -f $TMP_FILE ]]; then
    command="$(awk '{print $1}' $TMP_FILE)"
    level="$(awk '{print $2}' $TMP_FILE)"
    type="$(awk '{print $3}' $TMP_FILE)"

    case $command in 
      "SELECTED_LEVEL")
        if [[ $level == "-1" ]] then
          echo "Command provided but level is not selected"
          IN_LEVEL=0
          CURRENT_LEVEL=-1
        else
          echo "If you want to end the level early, enter 'end_level' into the command line"
          echo
          learn_cli current -s
          IN_LEVEL=1
          CURRENT_LEVEL=$level
          LEVEL_TYPE=$type
          START_TIME="$(date +%S.%N)"
        fi

        NUMBER_OF_USED_COMMANDS=0
        ;;

      "END_LEVEL")
        if [[ $IN_LEVEL == "1" || $IN_LEVEL == 1 ]] then
          IN_LEVEL=0
          START_TIME=""
          END_TIME=""
          NUMBER_OF_USED_COMMANDS=0
        else
          echo "Currently not in level"
        fi
        ;;

    esac

    #cat $TMP_FILE
    truncate $TMP_FILE --size 0
  fi
}

test_listener() {
  if [[ -f $TEST_FILE ]]; then
    contents=$(cat $TEST_FILE)
    if [[ $contents == "TEST" ]] then
      truncate $TEST_FILE --size 0
      echo "SUCCESS" >> /tmp/tmp_test_file
      exit
    fi
  fi
}

command_listener() {
  if [[ $IN_LEVEL -eq 1 ]] && [[ $BASH_COMMAND != "prompt_command_fn" ]] && [[ $BASH_COMMAND != "test_listener" ]] && [[ $BASH_COMMAND != "env_listener" ]]; then
    NUMBER_OF_USED_COMMANDS=$((NUMBER_OF_USED_COMMANDS+1))
    END_TIME="$(date +%S.%N)"
    export TRAPPED_COMMAND="$BASH_COMMAND"
  fi
}

prompt_command_fn() {
  env_listener
  if [[ $IN_LEVEL -eq 1 ]] && [[ $TRAPPED_COMMAND != "command_listener" ]] && [[ $TRAPPED_COMMAND != "test_listener" ]] && [[ $TRAPPED_COMMAND != "env_listener" ]] && [[ $TRAPPED_COMMAND != "" ]]; then
    check "$TRAPPED_COMMAND"
    TRAPPED_COMMAND=""
    env_listener
  fi
}

env_listener
test_listener
PROMPT_COMMAND="prompt_command_fn"
trap command_listener debug
