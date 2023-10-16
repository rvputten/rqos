#!/bin/bash

RED='\033[31m'
WHITE='\033[37m'
WHITE_BG='\033[47m'
RED_BG='\033[41m'
RESET='\033[0m'
BOLD='\033[1m'

echo -e "\e[32mThis text goes to stdout\e[0m"
echo -e "\e[31mThis text goes to stderr\e[0m" >&2
