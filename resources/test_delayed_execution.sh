#!/bin/bash

date +%F-%T
for i in {1..10}; do
    echo "This is a test message $i"
    sleep 0.1
done
