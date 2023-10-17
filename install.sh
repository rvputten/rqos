#!/bin/bash

# NOTE:
# =====
# Do not install font_editor, as it's edits will be overwritten by edits of the development version.

RES_PATH=$HOME/.local/share/rqos
cargo install --path rqsh &&
    mkdir -p $RES_PATH &&
    cp -a resources $RES_PATH/
