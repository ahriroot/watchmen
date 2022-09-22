#!/bin/bash

function main() {

    declare -i i=0
    
    while [ $i -lt 10 ]; do
        i=$i+1
        echo "Result from shell task: $i"
        if [ $i -eq 5 ]; then
            break
        fi
        sleep 1
    done
}

main
