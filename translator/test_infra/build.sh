#!/bin/bash
# SOLC

if ! solc --version >/dev/null 2>&1; then
  echo 'solc command was not found.
      \rPlease install solc on your computer. See: https://docs.soliditylang.org/en/develop/installing-solidity.html
      ' >&2

  exit 10
fi
solc --version

for path in $(find sol -name "*.sol"); do
  echo "build: ${path}"

  result=$( solc -o bin --bin --optimize-runs=0 --abi --ast-compact-json --overwrite --error-recovery --asm ${path} 2>&1)
  echo $result

  if [[ $result == Error* || $result == Warning* ]]; then
    echo $result >&2
    exit 11
  fi
done
