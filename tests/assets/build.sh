# SOLC

DIR=tests/assets
SOLC=$DIR/bin/solc
EXISTING_SOLC=$(which solc)
if [ $EXISTING_SOLC ]; then SOLC=$EXISTING_SOLC; fi

if ! $SOLC --version > /dev/null 2>&1
then
  echo 'solc command was not found.
      \rPlease install solc on your computer. See: https://docs.soliditylang.org/en/develop/installing-solidity.html
      ' >&2;

  exit 1;
fi

$SOLC -o $DIR/bin --bin --abi --ast-compact-json --overwrite --asm $DIR/const_fn.sol
$SOLC -o $DIR/bin --bin --abi --ast-compact-json --overwrite $DIR/parameters.sol
$SOLC -o $DIR/bin --bin --abi --ast-compact-json --overwrite --asm $DIR/math_fn.sol
$SOLC -o $DIR/bin --bin --abi --ast-compact-json --overwrite --asm $DIR/mult_fn.sol
$SOLC -o $DIR/bin --bin --optimize-runs=0 --abi --ast-compact-json --overwrite --asm $DIR/a_plus_b.sol
$SOLC -o $DIR/bin --bin --optimize-runs=0 --abi --ast-compact-json --overwrite --asm $DIR/hello_world.sol
$SOLC -o $DIR/bin --bin --optimize-runs=0 --abi --ast-compact-json --overwrite --asm $DIR/stateful.sol
$SOLC -o $DIR/bin --bin --optimize-runs=0 --abi --ast-compact-json --overwrite --asm $DIR/two_functions.sol

# APTOS

if ! aptos --version > /dev/null 2>&1
then
  echo 'Error: aptos command was not found.
      \rPlease install aptos on your computer. See: https://github.com/aptos-labs/aptos-core
      ' >&2;

  exit 2;
fi

aptos move compile --package-dir ./tests/assets/move

