

DIR=test/contracts
SOLC=$DIR/bin/solc
EXISTING_SOLC=$(which solc)
if [ $EXISTING_SOLC ]; then SOLC=$EXISTING_SOLC; fi

$SOLC -o $DIR/bin --bin --optimize-runs=0 --abi --ast-compact-json --asm $DIR/a_plus_b.sol
$SOLC -o $DIR/bin --bin --optimize-runs=0 --abi --ast-compact-json --asm $DIR/hello_world.sol
$SOLC -o $DIR/bin --bin --optimize-runs=0 --abi --ast-compact-json --asm $DIR/stateful.sol
$SOLC -o $DIR/bin --bin --optimize-runs=0 --abi --ast-compact-json --asm $DIR/two_functions.sol
