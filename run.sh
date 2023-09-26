#!/bin/bash

set -o pipefail

dirname=$(cd "$(dirname "$0")"; pwd -P)
exe_name=`basename $dirname`

################
exe_name=rqsh
################

scan_error_first_40_lines() {
  awk '/^error/{c++; if (c==2) {exit}} {print}' | head -40
}

function error_to_clipboard() {
  error_lines_count=`cat error.txt | wc -l`
  clear
  echo "error_lines_count $1: $error_lines_count"

  # remove escape codes
  [[ $error_lines_count -gt 7 ]] && cat error.txt |
      sed 's/\x1b\[[0-9;]*m//g' |
      scan_error_first_40_lines |
      xclip -i
  echo --------------------------------------------------------------------------------
  cat error.txt
}

function wait_and_run() {
  echo "ret: $r"
  echo --------------------------------------------------------------------------------
  inotifywait -q -e close_write */src Cargo.toml */Cargo.toml resources/*.{frag,sh}
  clear

  exec ./run.sh
}

pid=`ps -eo pid,comm|grep -w "$exe_name"|awk '{print $1}'`
[[ -n $pid ]] && { echo "Kill $pid"; kill $pid; }

cargo fmt
cargo clippy --color=always -- -D warnings 2>&1 | scan_error_first_40_lines | tee error.txt
r=$?
error_to_clipboard clippy
[[ "$r" -ne 0 ]] && wait_and_run

cargo clippy --tests -- -D warnings 2>&1 | scan_error_first_40_lines | tee error.txt
r=$?
error_to_clipboard clippy_tests
[[ "$r" -ne 0 ]] && wait_and_run


[[ -f .env ]] && source .env
cargo test --workspace --color=always -- --nocapture 2>&1 | scan_error_first_40_lines | tee error.txt
r=$?
echo one
[[ "$r" -ne 0 ]] && wait_and_run
echo two
error_to_clipboard tests


rg --color=always '#\[allow\((dead_code|unused_variables)\)\]'
#unbuffer cargo run -p font_editor &
unbuffer cargo run &
wait_and_run
