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

# temporary, needs to be location of binary during distribution
alias run_binary='cargo run --manifest-path $PROJECT_PATH'
alias check='cargo run --manifest-path $PROJECT_PATH --  --user-command '
alias current_level='cargo run --manifest-path $PROJECT_PATH -- current-level'
alias all_levels='cargo run --manifest-path $PROJECT_PATH -- all-levels'

alias ls='ls --color=auto'
alias grep='grep --color=auto'
PS1='[\u@\h \W]\$ '

export NUMBER_OF_USED_COMMANDS=0
export START_TIME=""

echo "Welcome to the learning environment"
echo "To get started, simply enter play_level into the command line to play the latest level"
echo
echo "To select a specific level, you can add the --level or -l flag after typing play_level"
echo "Something like: 'play_level -l LEVEL_NUMBER'"
echo
echo "To close the game, run the 'exit' command"
echo

play_level () {
  # $1 is the -l flag, $2 is the level number
  echo "Play level"
  run_binary -- play $1 $2
  env_listener
}

end_level() {
  echo "ending level"
  run_binary -- end $1 $2 $3
}

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
          current_level
          IN_LEVEL=1
          CURRENT_LEVEL=$level
          LEVEL_TYPE=$type
          START_TIME="$(date +%S.%N)"
          echo "Playing level $CURRENT_LEVEL"
        fi

        NUMBER_OF_USED_COMMANDS=0
        ;;

      "END_LEVEL")
        if [[ $IN_LEVEL == "1" || $IN_LEVEL == 1 ]] then
          echo "Ending level"
          IN_LEVEL=0
          START_TIME=""
          NUMBER_OF_USED_COMMANDS=0
          # needs to submit these results (start and end msec and sec times, and also number of commands used)
        else
          echo "Currently not in level"
        fi
        ;;

      #*)
      #  # temporary
      #  echo "Unknown command"
      #  ;;
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
  #echo "Command: $BASH_COMMAND with exit status $?";
  #if [[ $IN_LEVEL -eq 1 ]] && [[ $LEVEL_TYPE == "COMMAND" ]]; then
  if [[ $IN_LEVEL -eq 1 ]] && [[ $BASH_COMMAND != "file_listener" ]]; then
    NUMBER_OF_USED_COMMANDS=$((NUMBER_OF_USED_COMMANDS+1))
    export TRAPPED_COMMAND="$BASH_COMMAND"
  fi
}

file_listener() {
  #if [[ $IN_LEVEL -eq 1 ]] && [[ $LEVEL_TYPE == "FILE" ]]; then
  if [[ $IN_LEVEL -eq 1 ]]; then
    check "$TRAPPED_COMMAND"
    TRAPPED_COMMAND=""
    env_listener
  fi
}

env_listener
test_listener
PROMPT_COMMAND="file_listener"
trap command_listener debug
