# shi

small history / 史

An experimental alternative to bash history using sqlite.

## Install

`git clone` this repo, and paste the code below in `.bashrc`.

_bash-preexec required. See [https://github.com/rcaloras/bash-preexec](https://github.com/rcaloras/bash-preexec)_

```bash
shi_func() {
    if [[ ! $1 =~ '^\s.*' ]]; then
        shi insert "$@"
    fi
}

source ~/.bash-preexec.sh
preexec() { shi insert "$@"; }
```

## Usage

| command                 | What is this                                                            |
| ----------------------- | ----------------------------------------------------------------------- |
| shi                     | Print the last 50 commands and time.                                    |
| shi all                 | Print all the history with the directory where the command is executed. |
| shi insert \<command\>  | Insert the command to the database.                                     |
| shi command \<command\> | Search command history that matches the query.                          |
| shi dir \<path\>        | Search directory history that matches the query.                        |
| shi drop                | Drop the database table and delete history.                             |
| shi export              | Export the history to json format.                                      |
