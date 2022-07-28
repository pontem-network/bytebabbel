# eth2move

EVM to Move static bytecode translator.

## e2m
Converts **solidity file** to **binary move** code. You can convert from **abi + bin** files or a **sol** file

> **! IMPORTANT**\
> To convert from a **sol** file, **solc** must be installed on the computer and accessible from the terminal using the short command **solc**.

### Install solc

How to install **solc**, [see the documentation](https://docs.soliditylang.org/en/develop/installing-solidity.html)

### Checking solc

The **solc** version must be at least **0.8.15**

```bash
solc --version
```
> **! IMPORTANT**\
> If this command is not available for execution from the terminal, e2m will not work.

### Installation e2m
Cloning the repository and installing e2m:

```bash
git clone https://github.com/pontem-network/eth2move
cargo +nightly install --path cli/e2m
```

### See help:
```bash
e2m --help
```

### Input parameters
* `<PATH>`              Path to the file. Specify the path to sol file or abi | bin
* `-o`, `--output`      Where to save the converted Move binary file
* `--module`            The name of the move module. If not specified, the name will be taken from the abi path
* `-a`, `--address`     The address of the move module [default: 0x1]
* `-m`, `--math`        Math backend u128 or u256 [default: u128]
* `-t`, `--trace`       Output of debugging information [possible values: true, false][defualt: false]
* `-h`, `--help`        Print help information
* `-V`, `--version`     Print version information

### Example
You can find the files from the examples in the [eth2move/examples](https://github.com/pontem-network/eth2move/tree/master/examples) folder

#### Required parameters
Required parameters are the paths to sol file (`<PATH>`).\
The file can be extensions:
* `sol` - The file will be compiled using the solc utility. The resulting **abi** and **bin** will be translated into **move binarycode**
* `bin` - It is expected that there is an `abi` file with the same name in the same folder. These **abi** and **bin** will be translated into **move binarycode**
* `abi` - It is expected that there is an `bin` file with the same name in the same folder. These **abi** and **bin** will be translated into **move binarycode**

The name from the passed **solidity library** will be used as the filename and the name of the **move module**.\
After completing the command, you will see the path to the created file (Example: "./NameSolModule.mv"). 
By default, the file is saved to the current directory.


##### examples/a_plus_b.sol
```bash
e2m examples/a_plus_b.sol 
```

###### Result
> Saved in "./APlusB.mv

Move module address: **0x1**\
Move module name: **APlusB**

##### examples/APlusB.abi
```bash
e2m examples/APlusB.abi
```

###### Result
> Saved in "./APlusB.mv"

Move module address: **0x1**\
Move module name: **APlusB**


##### examples/APlusB.bin
```bash
e2m examples/APlusB.bin
```

###### Result
> Saved in "./APlusB.mv"

Move module address: **0x1**\
Move module name: **APlusB**


##### ! Fail: examples/BinNotFound.abi
```bash
e2m examples/BinNotFound.abi
```

###### Result
> Error: Couldn't find bin.
Path:"examples/BinNotFound.bin"

> **! IMPORTANT**\
> A successful broadcast always requires a **bin** and an **abi** **solidity library**

#### Path to save
The `-o`, `--output` parameter is responsible for specifying the location where the converted file will be saved.

##### examples/const_fn.sol

```bash
e2m examples/const_fn.sol -o ./Test.mv
```

##### Result
> Saved in "./Test.mv"

The move binary file will be created in the current directory named **Test.vm**\
Move module address: **0x1** \
Move module name: **Cons**

##### examples/APlusB.bin

```bash
e2m examples/APlusB.bin -o ./AB.mv
```

##### Result
> Saved in "./AB.mv"

Move module address: **0x1** \
Move module name: **APlusB**

#### Explicit indication of the module name in the received move bytecode
The `--module` argument is responsible for explicitly specifying the move module name.

##### examples/APlusB.abi

```bash
e2m examples/APlusB.abi --module ApB
```

##### Result
> Saved in "./APlusB.mv"

Move module address: **0x1** \
Move module name: **ApB**

##### examples/two_functions.sol

```bash
e2m examples/two_functions.sol --module TF
```

##### Result
> Saved in "./TwoFunctions.mv"

Move module address: **0x1** \
Move module name: **TF**

#### Explicit indication of the module address in the received move bytecode
The `-a`,`--address` argument is responsible for explicitly specifying the move module address.

##### examples/const_fn.sol

```bash
e2m examples/const_fn.sol -a 0x3
```

##### Result
> Saved in "./ConstFn.mv"

Move module address: **0x3** \
Move module name: **ConstFn**

##### examples/two_functions.sol

```bash
e2m examples/two_functions.sol --address 0x0123
```

##### Result
> Saved in "./TwoFunctions.mv"

Move module address: **0x0123** \
Move module name: **TwoFunctions**

#### Combined arguments

```bash
 e2m examples/const_fn.sol -o ./MyMove.mv --module DemoName --address 0x3 
```

##### Result
> Saved in "./MyMove.mv"

Move module address: **0x3** \
Move module name: **DemoName**

## What the converter can already do.

See the [folder](https://github.com/pontem-network/eth2move/tree/master/translator/test_infra/sol)