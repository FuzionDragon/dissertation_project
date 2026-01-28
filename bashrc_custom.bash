#
# ./bashrc_custom.bash
#

# If not running interactively, don't do anything
# This is a custom bashrc, use rcfile option of Bash
[[ $- != *i* ]] && return

# Learning tool logic
export APP_ACTIVE=1
export IN_LEVEL=0
export CURRENT_LEVEL=0
export NUMBER_OF_USED_COMMANDS=0

# temporary, should be executable path in the future
export PROJECT_PATH="$(pwd)/Cargo.toml"

# temporary, needs to be location of binary during distribution
alias check='cargo run --manifest-path $PROJECT_PATH -- -u'

alias ls='ls --color=auto'
alias grep='grep --color=auto'
PS1='[\u@\h \W]\$ '
PROMPT_COMMAND=''

debug_fn () {
  #echo "Command: $BASH_COMMAND with exit status $?";
  if [[ $IN_LEVEL -eq 1 ]]; then
    check "$BASH_COMMAND"
  fi
}

trap debug_fn debug
echo "Welcome to the learning environment"
