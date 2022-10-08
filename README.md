# shi

small history / 史

An experimental alternative to bash history using sqlite.

## Installation

```
git clone https://github.com/kyoheiu/shi.git
cd shi
cargo install --path .
```

And add the following lines to the end of `.bashrc` file.

_bash-preexec required. See [https://github.com/rcaloras/bash-preexec](https://github.com/rcaloras/bash-preexec)_

```bash
source ~/.bash-preexec.sh
preexec() { shi --insert "$@"; }
```

## Usage

```
shi [option]              Print the last 50 commands and time

Options:
  -a, --all               Print all the history with the directory path where the command was executed
  -i, --insert <COMMAND>  Insert the command to the history
  -d, --delete <ID>       Delete the command that matches the id
  -r, --remove            Drop the database table, delete all history
  -p, --path <PATH>       Show commands that were executed in directories that match the query
  -c, --command <COMMAND> Show commands that match the query
  -o, --output            Export all the history to `~/.shi/history.json`
```
