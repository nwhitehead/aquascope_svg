#!/bin/env bash

for file in ../testdata/json/*; do
    path=${file%/*}
    base=${file##*/}
    name=${base%%.*}
    gold=$path/../gold/$name.gold
    bare=$path/../gold/$name.bare.gold
    resultFile=$gold
    echo $file "->" $resultFile
    cargo run --bin aquascope_json_to_states -- --show-code --show-locations $file > $resultFile
    resultFile=$bare
    echo $file "->" $resultFile
    cargo run --bin aquascope_json_to_states -- $file > $resultFile
done
