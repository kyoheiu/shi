pub const HELP: &str = r#"
small history / 史

An alternative to bash history using sqlite.

## Installation

```
git clone https://github.com/kyoheiu/shi.git
cd shi
cargo install --path .
```

And add the following lines to the end of `.bashrc` file.

bash-preexec required. See https://github.com/rcaloras/bash-preexec

```
source ~/.bash-preexec.sh
preexec() { shi --insert "$@"; }
```

## Usage

At the first launch, `shi` creates sqlite database in `~/.shi/.history`.

shi [ROWS]                       Print executed commands and time. (Default: 50 rows)

Options:
  -a, --all                      Print all the history with the directory path where the command was executed
  -i, --insert <COMMAND>         Insert the command to the history
  -r, --remove <ID>              Delete the command that matches the id
  -p, --path <PATH> [ROWS]       Show commands that were executed in directories that match the query
  -c, --command <COMMAND> [ROWS] Show commands that match the query
  -o, --output                   Export all the history to `$XDG_DATA_HOME/shi/history.csv`
  --drop                         Drop the database table, deleting all history

Unless you set `-a` option, you can choose the number (leftest chars) to copy to system clipboard.
To copy, set the environment variable `$SHI_CLIP`: i.e. `SHI_CLIP=wl-copy`.
"#;
