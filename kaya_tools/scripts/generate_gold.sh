#!/bin/env bash

for file in ../../testdata/json/*; do
    path=${file%/*}
    base=${file##*/}
    name=${base%%.*}
    gold=$path/../gold/$name.states
    bare=$path/../bare/$name.states
    html=$path/../html/$name.html
    resultFile=$gold
    echo $file "->" $resultFile
    cargo run --bin aquascope_json_to_kaya -- --show-code --show-locations $file > $resultFile
    resultFile=$bare
    echo $file "->" $resultFile
    cargo run --bin aquascope_json_to_kaya -- $file > $resultFile
    resultFile=$html
    echo $bare "->" $resultFile
    cargo run --bin render_kaya -- $bare --output $resultFile
done
