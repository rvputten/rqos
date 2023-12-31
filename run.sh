#!/bin/bash

set -o pipefail

call_args=( "$@" )
dirname=$(cd "$(dirname "$0")"; pwd -P)
exe_name=`basename $dirname`
debug=0
if [[ "$1" == "debug" ]]; then
    debug=1
    shift
fi
# doesn't work for workspaces; workaround:
exe_name=${1:-"rqsh"}

# ---------
# FUNCTIONS
# ---------

scan_error_code_first_40_lines() {
  awk '/^error/{c++; if (c==2) {exit}} {print}' | head -40
}

scan_error_test_last_40_lines() {
  tail -40
}

function error_to_clipboard() {
  error_lines_count=`cat error.txt | wc -l`
  clear
  echo "error_lines_count $1: $error_lines_count"

  # remove escape codes
  [[ $error_lines_count -gt 7 ]] && cat error.txt |
      sed 's/\x1b\[[0-9;]*m//g' |
      scan_error_code_first_40_lines |
      xclip -i
  echo --------------------------------------------------------------------------------
  cat error.txt
}

function wait_and_run() {
    echo "ret: $r"
    echo --------------------------------------------------------------------------------
    inotifywait -q -e close_write */src Cargo.toml */Cargo.toml resources/*.frag scripts/*.sh
    clear

    exec ./run.sh ${call_args[@]}
}

# -----
# CARGO
# -----

# CARGO FMT
cargo fmt

# CARGO BUILD
(cargo build -p $exe_name --color=always 2>&1 | scan_error_code_first_40_lines | tee error.txt)
r=$?
error_to_clipboard build
[[ "$r" -ne 0 ]] && wait_and_run

# CARGO CLIPPY
cargo clippy --color=always -- -D warnings 2>&1 | scan_error_code_first_40_lines | tee error.txt
r=$?
error_to_clipboard clippy
[[ "$r" -ne 0 ]] && wait_and_run

cargo clippy --tests -- -D warnings 2>&1 | scan_error_code_first_40_lines | tee error.txt
r=$?
error_to_clipboard clippy_tests
[[ "$r" -ne 0 ]] && wait_and_run

# DEBUG
if [[ "$debug" -eq 1 ]]; then
    cargo build -p $exe_name --color=always
    r=$?
    [[ "$r" -eq 0 ]] && {
        rust-gdb target/debug/$exe_name
        wait_and_run
    }
fi

# CARGO TEST
[[ -f .env ]] && source .env
cargo test --workspace --color=always -- --nocapture 2>&1 | tee error.txt
r=$?
[[ "$r" -ne 0 ]] && wait_and_run
error_to_clipboard tests

# CARGO RUN
cargo run -p $exe_name --color=always
r=$?
echo $! > .pid
[[ "$r" -ne 0 ]] && wait_and_run


rg --color=always '#\[allow\((dead_code|unused_variables)\)\]'
#unbuffer cargo run -p $exe_name --color=always &
wait_and_run
