#!/bin/bash

set -o pipefail

dirname=$(cd "$(dirname "$0")"; pwd -P)
exe_name=`basename $dirname`

pid=`ps -eo pid,comm|grep -w "$exe_name"|awk '{print $1}'`
[[ -n $pid ]] && { echo "Kill $pid"; kill $pid; }

cargo fmt
cargo clippy 2>&1 | head -40 | tee error.txt
if [[ "$?" -eq 0 ]]; then
    [[ -f .env ]] && source .env
    cargo test && {
	echo --------------------------------------------------------------------------------
	unbuffer cargo build &
    }
else
    clear
    unbuffer cargo clippy 2>&1 | head -40
fi

error_lines_count=`cat error.txt | wc -l`
echo "error_lines_count: $error_lines_count"

# remove escape codes
[[ $error_lines_count -gt 7 ]] && cat error.txt |
    sed 's/\x1b\[[0-9;]*m//g' |
    awk '/^error/{c++; if (c==2) {exit}} {print}' error.txt |
    xclip -i

echo --------------------------------------------------------------------------------
inotifywait -q -e close_write src Cargo.toml run.sh
clear

exec ./run.sh
