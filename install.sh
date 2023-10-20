#!/bin/bash

# NOTE:
# =====
# Do not install font_editor, as it's edits will be overwritten by edits of the development version.

xdg_data_home=${XDG_DATA_HOME:-$HOME/.local/share}
RQOS_PATH=$xdg_data_home/rqos
cargo install --path rqsh &&
    mkdir -p $RQOS_PATH &&
    cp -a resources $RQOS_PATH/
