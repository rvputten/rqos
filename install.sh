#!/bin/bash
RES_PATH=$HOME/.local/share/rqos
cargo install --path . --bin rqsh &&
    mkdir -p $RES_PATH &&
    cp -a resources $RES_PATH/
