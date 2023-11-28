#!/bin/bash

echo "Lines 1-4 must be in order without gaps."
echo "Line 1"
echo
echo "Line 3"
printf "\e[2ALine 2\n"
printf "\e[2BLine 4\n"
