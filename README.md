# eth2move

EVM to Move static bytecode translator.

## s2m
Converts **solidity file** to **binary move** code.\
For the converter to work, **solc** must be installed on the computer and accessible from the terminal by a short command **solc**.

### Install solc

How to install **solc**, [see the documentation](https://docs.soliditylang.org/en/develop/installing-solidity.html)

### Checking solc

The **solc** version must be at least **0.8.15**

```bash
solc --version
```
> IMPORTANT!\
> If this command is not available for execution from the terminal, s2m will not work.

### Installation s2m
Cloning the repository and installing s2m:

```bash
git clone https://github.com/pontem-network/eth2move
cargo +nightly install --path cli/s2m
```

### See help:
```bash
s2m --help
```

### Input parameters
* `-p`, `--path`        Path to the sol file
* `-o`, `--output`      Where to save the converted Move binary file
* `--module`            The name of the move module. If not specified, the name will be taken from the abi path
* `--address`           The address of the move module [default: 0x1]
* `-m`, `--math`        Math backend u128 or u256 [default: u128]
* `-t`, `--trace`       Output of debugging information [possible values: true, false][defualt: false]
* `-h`, `--help`        Print help information
* `-V`, `--version`     Print version information

### Example
You can find the files from the examples in the [eth2move/examples](https://github.com/pontem-network/eth2move/tree/master/examples) folder

#### Required parameters
Required parameters are the paths to sol file (``--path``, ``-p``).\
The name from the passed **solidity library** will be used as the filename and the name of the **move module**.\
After completing the command, you will see the path to the created file (Example: "tmp/RANDOM_NAME/NameSolModule.mv").

##### examples/a_plus_b.sol
```bash
s2m -p examples/a_plus_b.sol 
```

###### Result
> Saved in "/tmp/PZH107KZQ7JWT47CNFZM2NZJ3C/APlusB.mv

Move module address: **0x1**\
Move module name: **APlusB**

##### examples/const_fn.sol
```bash
s2m -p examples/const_fn.sol 
```

###### Result
> Saved in "/tmp/9CG89C4R7J40P1KJ4TPDKFNPZG/ConstFn.mv"

Move module address: **0x1**\
Move module name: **ConstFn**


#### Path to save
The `-o`, `--output` parameter is responsible for specifying the location where the converted file will be saved.

##### examples/a_plus_b.sol

```bash
s2m -p examples/a_plus_b.sol -o ./Test.mv
```

##### Result
> Saved in "./Test.mv"

The move binary file will be created in the current directory named **Test.vm**\
Move module address: **0x1** \
Move module name: **APlusB**

##### examples/const_fn.sol

```bash
s2m -p examples/const_fn.sol -o ./Cons.mv
```

##### Result
> Saved in "./Cons.mv"

Move module address: **0x1** \
Move module name: **ConstFn**

#### Explicit indication of the module name in the received move bytecode
The `--module` argument is responsible for explicitly specifying the move module name.

##### examples/const_fn.sol

```bash
s2m -p examples/const_fn.sol --module CnFn
```

##### Result
> Saved in "/tmp/9CG89C4R7J40P1KJ4TPDKFNPZG/ConstFn.mv"

Move module address: **0x1** \
Move module name: **CnFn**

##### examples/two_functions.sol

```bash
s2m -p examples/two_functions.sol --module TF
```

##### Result
> Saved in "/tmp/YEVS62B4FPDA8K4VV7VPQ3KFAG/TwoFunctions.mv"

Move module address: **0x1** \
Move module name: **TF**

#### Explicit indication of the module address in the received move bytecode
The `--address` argument is responsible for explicitly specifying the move module address.

##### examples/const_fn.sol

```bash
s2m -p examples/const_fn.sol --address 0x3
```

##### Result
> Saved in "/tmp/9CG89C4R7J40P1KJ4TPDKFNPZG/ConstFn.mv"

Move module address: **0x3** \
Move module name: **ConstFn**

##### examples/two_functions.sol

```bash
s2m -p examples/two_functions.sol --address 0x0123
```

##### Result
> Saved in "/tmp/YEVS62B4FPDA8K4VV7VPQ3KFAG/TwoFunctions.mv"

Move module address: **0x0123** \
Move module name: **TwoFunctions**

#### Combined arguments

```bash
 s2m -p examples/const_fn.sol -o ./MyMove.mv --module DemoName --address 0x3 
```

##### Result
> Saved in "./MyMove.mv"

Move module address: **0x3** \
Move module name: **DemoName**

## What the converter can already do.

See the [folder](https://github.com/pontem-network/eth2move/tree/master/translator/test_infra/sol)