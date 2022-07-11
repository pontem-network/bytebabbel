#!/bin/bash

# SOLC

DIR=tests/assets
SOLC=$DIR/bin/solc
EXISTING_SOLC=$(which solc)
if [ $EXISTING_SOLC ]; then
  SOLC=$EXISTING_SOLC
fi

if ! $SOLC --version >/dev/null 2>&1; then
  echo 'solc command was not found.
      \rPlease install solc on your computer. See: https://docs.soliditylang.org/en/develop/installing-solidity.html
      ' >&2

  exit 1
fi

for path in $(find $DIR/sol/ -name "*.sol"); do
  path=$(realpath ${path})
  echo "build: ${path}"

  result=$(${SOLC} -o ${DIR}/bin --bin --optimize-runs=0 --abi --ast-compact-json --overwrite --error-recovery --asm ${path} 2>&1)
  echo $result

  if [[ $result == Error* || $result == Warning* ]]; then
    echo $result >&2
    exit 3
  fi
done

# APTOS

if ! aptos --version >/dev/null 2>&1; then
  echo 'Error: aptos command was not found.
      \rPlease install aptos on your computer. See: https://github.com/aptos-labs/aptos-core
      ' >&2

  exit 2
fi

aptos move compile --package-dir ./tests/assets/move
