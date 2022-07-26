# eth2move

EVM to Move static bytecode translator.

## s2m
Converts solidity file to **binary move** code.\
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
// @todo
You can find the files from the examples in the examples folder

#### Required parameters
Required parameters are the paths to sol file (``--path``, ``-p``)

```bash
s2m -p examples/a_plus_b.sol 
```

#### Result
> Saved in "/tmp/PZH107KZQ7JWT47CNFZM2NZJ3C/APlusB.mv

After completing the command, you will see the path to the created file (Example: "tmp/RANDOM_NAME/NameSolModule.mv").
Move module address: **0x1**\
Move module name: **APlusB**

#### Path to save

```bash
s2m -p examples/a_plus_b.sol -o ./Test.mv
```

#### Result
> Saved in "./Test.mv"

The move binary file will be created in the current directory named **Test.vm**\
Move module address: **0x1** \
Move module name: **APlusB**

#### Specifying the move module name and address

```bash
 s2m -p examples/const_fn.sol -o ./MyMove.mv --module DemoName --address 0x3 
```

#### Result
> Saved in "./MyMove.mv"

Move module address: **0x3** \
Move module name: **DemoName**

## What the converter can already do.

See the [folder](https://github.com/pontem-network/eth2move/tree/master/translator/test_infra/sol)