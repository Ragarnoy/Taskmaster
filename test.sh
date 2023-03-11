#! /bin/bash

if [ $((RANDOM % 2)) -eq 0 ]; then
	echo "Early exit"
	exit 0
fi

sleep 2

if [ $((RANDOM % 10)) -eq 0 ]; then
	echo "Success"
	exit 0
fi
exit -1
