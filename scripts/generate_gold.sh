#!/bin/env bash

for file in ../testdata/json/*; do
    path=${file%/*}
    base=${file##*/}
    name=${base%%.*}
    resultFile=$path/../gold/$name.gold
    echo $file "->" $resultFile
    cargo run -- $file > $resultFile
done
