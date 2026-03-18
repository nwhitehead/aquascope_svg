#!/bin/env bash

for file in ../testdata/json/*; do
    path=${file%/*}
    base=${file##*/}
    name=${base%%.*}
    resultFile=$path/../gold/$name.gold
    echo $file "->" $resultFile
    cargo run --bin aquascope_json_to_states -- $file > $resultFile
done
