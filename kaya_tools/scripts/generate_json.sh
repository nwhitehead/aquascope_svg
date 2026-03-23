#!/bin/env bash

: "${AQUASCOPE_CLI:=../../../aquascope/target/debug/aquascope_cli}"

for file in ../../testdata/rust/*; do
    shouldFail=""
    if [[ $file == *"error"* ]]; then
        shouldFail="--should-fail"
    fi
    path=${file%/*}
    base=${file##*/}
    name=${base%%.*}
    resultFile=$path/../json/$name.json
    echo $file "->" $resultFile
    ${AQUASCOPE_CLI} $shouldFail --filename=$file > $resultFile
done
