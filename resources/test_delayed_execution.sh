#!/bin/bash

date +%F-%T
echo "Messages one per line..."
for i in {1..10}; do
    echo "This is a test message $i"
    sleep 0.1
done

echo "Multiple messages on one line..."
for i in {1..10}; do
	echo -n "$i "
	sleep 0.1
done
echo
