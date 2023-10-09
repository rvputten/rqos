#!/bin/bash

set -o pipefail

dirname=$(cd "$(dirname "$0")"; pwd -P)
exe_name=`basename $dirname`

# doesn't work for workspaces; workaround:
exe_name=${1:-"rqsh"}

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
  inotifywait -q -e close_write */src Cargo.toml */Cargo.toml resources/*.{frag,sh}
  clear

  exec ./run.sh "$exe_name"
}

pid=`ps -eo pid,comm|grep -w "$exe_name"|awk '{print $1}'`
[[ -n $pid ]] && { echo "Kill $pid"; kill $pid; }

cargo fmt

cargo run -p $exe_name --color=always
r=$?
[[ "$r" -ne 0 ]] && wait_and_run

cargo clippy --color=always -- -D warnings 2>&1 | scan_error_code_first_40_lines | tee error.txt
r=$?
error_to_clipboard clippy
[[ "$r" -ne 0 ]] && wait_and_run

cargo clippy --tests -- -D warnings 2>&1 | scan_error_code_first_40_lines | tee error.txt
r=$?
error_to_clipboard clippy_tests
[[ "$r" -ne 0 ]] && wait_and_run


[[ -f .env ]] && source .env
cargo test --workspace --color=always -- --nocapture 2>&1 | tee error.txt
r=$?
echo one
[[ "$r" -ne 0 ]] && wait_and_run
echo two
error_to_clipboard tests


rg --color=always '#\[allow\((dead_code|unused_variables)\)\]'
#unbuffer cargo run -p $exe_name --color=always &
wait_and_run
