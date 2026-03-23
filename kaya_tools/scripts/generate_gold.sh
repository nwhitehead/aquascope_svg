#!/bin/env bash

for file in ../../testdata/json/*; do
    path=${file%/*}
    base=${file##*/}
    name=${base%%.*}
    gold=$path/../gold/$name.states
    bare=$path/../bare/$name.states
    resultFile=$gold
    echo $file "->" $resultFile
    cargo run --bin aquascope_json_to_kaya -- --show-code --show-locations $file > $resultFile
    resultFile=$bare
    echo $file "->" $resultFile
    cargo run --bin aquascope_json_to_kaya -- $file > $resultFile
done
