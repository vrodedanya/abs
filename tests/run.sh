#!/usr/bin/bash

ABS_PATH:=../target/debug/abs
PROFILE:=debug

IS_FAILED=false

for d in */ ; do
    echo "$d"
    cd $d
    ../$ABS_PATH build -p $PROFILE
    if [[ "$?" != "0" ]]; then
        echo "========Failed $d========"
        IS_FAILED=true
    else
        echo "========Done $d========"
    fi
    cd ..
done

if $IS_FAILED ; then
    exit 1
fi