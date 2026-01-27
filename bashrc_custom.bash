#
# ./bashrc_custom.bash
#

# If not running interactively, don't do anything
# This is a custom bashrc, use rcfile option of Bash
[[ $- != *i* ]] && return

alias ls='ls --color=auto'
alias grep='grep --color=auto'
PS1='[\u@\h \W]\$ '
PROMPT_COMMAND=''

# Learning tool logic
export APP_ACTIVE=1
export CURRENT_LEVEL=0
export NUMBER_OF_USED_COMMANDS=0

debug_fn () {
  echo "Command: $BASH_COMMAND with exit status $?";
}

trap debug_fn debug
echo "Welcome to the learning environment"
