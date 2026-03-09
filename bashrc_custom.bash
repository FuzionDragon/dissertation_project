#
# ./bashrc_custom.bash
#

# If not running interactively, don't do anything
# This is a custom bashrc, use rcfile option of Bash
[[ $- != *i* ]] && return

# Learning tool logic
export APP_ACTIVE=1
export IN_LEVEL=0
export CURRENT_LEVEL=-1
export TMP_FILE="/tmp/tmp_cli_learn"

# temporary, should be executable path in the future
export PROJECT_PATH="$(pwd)/Cargo.toml"

# temporary, needs to be location of binary during distribution
alias run_binary='cargo run --manifest-path $PROJECT_PATH'
alias check='cargo run --manifest-path $PROJECT_PATH --  --user-command '
#alias play_level='run_binary --  play'
#alias end_level='run_binary --  end && env_listener'

alias ls='ls --color=auto'
alias grep='grep --color=auto'
PS1='[\u@\h \W]\$ '
PROMPT_COMMAND=''

export NUMBER_OF_USED_COMMANDS=0

echo "Welcome to the learning environment"
echo "To get started, simply enter play_level into the command line to play the latest level"
echo
echo "To select a specific level, you can add the --level or -l flag after typing play_level"
echo "Something like: 'play_level -l LEVEL_NUMBER'"
echo
echo "To close the game, run the 'exit' command"
echo

play_level () {
  run_binary -- play $1 $2
  env_listener
}

end_level () {
  run_binary -- end
  env_listener
}

# used for reading the temp file contents, and acting on certain instructions it states
env_listener () {
  if [[ -f $TMP_FILE ]]; then
    command="$(awk '{print $1}' $TMP_FILE)"
    value="$(awk '{print $2}' $TMP_FILE)"

    case $command in 
      "SELECTED_LEVEL")
        if [[ $value == "-1" ]] then
          echo "Command provided but level is not selected"
          IN_LEVEL=0
          CURRENT_LEVEL=-1
        else
          echo "Level starting.."
          echo "If you want to end the level early, enter 'end_level' into the command line"
          IN_LEVEL=1
          CURRENT_LEVEL=$value
          echo "Playing level $CURRENT_LEVEL"
        fi
        ;;

      "END_LEVEL")
        if [[ $IN_LEVEL == "1" || $IN_LEVEL == 1 ]] then
          echo "Ending level"
          IN_LEVEL=0
        else
          echo "Currently still in level"
        fi
        ;;

      *)
        # temporary
        echo "Unknown command"
        ;;
    esac

    #cat $TMP_FILE
    truncate $TMP_FILE --size 0
  fi
}

debug_trap () {
  #echo "Command: $BASH_COMMAND with exit status $?";
  if [[ $IN_LEVEL -eq 1 ]]; then
    check "$BASH_COMMAND"
  fi
}

env_listener

trap env_listener exit
trap debug_trap debug
