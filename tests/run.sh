#!/usr/bin/bash

ABS_PATH=../target/debug/abs

for d in */ ; do
    echo "$d"
    cd $d
    ../$ABS_PATH build
    if [[ "$?" != "0" ]]; then
        echo "Failed $d"
        exit 1
    fi
    echo $?
    cd ..
done