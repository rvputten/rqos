#!/bin/bash

# go home
printf "\e[H"
echo "Line 1    "
echo
printf "\e[3HLine 3    \n"
printf "\e[2ALine 2    \n"
printf "\e[2BLine 5    \n"
printf "\e[4HLine 4    \n"
printf "\e[B"
echo "The lines must be in order without gaps from the top of the screen."
