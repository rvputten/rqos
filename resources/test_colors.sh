#!/bin/bash

RED='\033[31m'
WHITE='\033[37m'
WHITE_BG='\033[47m'
RED_BG='\033[41m'
RESET='\033[0m'
BOLD='\033[1m'
echo -e "\e[31mThis is red text\e[0m"
echo -e "\e[32mThis is green text\e[0m"
echo -e "\e[1;31mThis is bold red text\e[0m"
echo -e "\e[1;32mThis is bold green text\e[0m"
echo -e "${WHITE}${RED_BG}This is ${BOLD}white$RESET$WHITE$RED_BG text on red background$RESET"
