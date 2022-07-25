# eth2move

EVM to Move static bytecode translator.

## e2m
Converts **bin** and **abi** files to **binary move** code

### Installation
Cloning the repository and installing e2m:

```bash
git clone https://github.com/pontem-network/eth2move
cargo +nightly install --path crates/e2m
```

### See help:
```bash
e2m --help
```

### Input parameters
* `-a`, `--abi`         Path to the abi file
* `-b`, `--bin`         Path to the bin file
* `-o`, `--output`      Where to save the converted file
* `--module`            The name of the module. If not specified, the name will be taken from the abi path
* `--address`           The address of the module [default: 0x1]
* `-m`, `--math`        Math backend u128 or u256 [default: u128]
* `-t`, `--trace`       Output of debugging information [possible values: true, false][defualt: false]
* `-h`, `--help`        Print help information
* `-V`, `--version`     Print version information

### Example
#### Required parameters
Required parameters are the paths to :
* abi file (``--abi``, ``-a``)
* bin file (``--bin``, ``-b``)

```bash
e2m -a path/to/file/NameFile.abi -b path/to/file/NameFile.bin  
```

The file is created by the path "path/to/file/NameFile.mv".
If the save path was not specified, then the file is saved in the same directory and with the same name as the abi file, 
but with the extension **mv**.
The module name will be taken from the file name.
Module address: **0x1**\
Module name: **NameFile**

#### Path to save

```bash
e2m -a path/to/file/NameFile.abi -b path/to/file/NameFile.bin -o ./Demo.vm
```
The move binary file will be created in the current directory named **Demo.vm**\
Module address: **0x1** \
Module name: **NameFile**

#### Specifying the module name and address

```bash
e2m --module DemoName --address 0x3 -a path/to/file/NameFile.abi -b path/to/file/NameFile.bin -o ./Demo.vm 
```
Module address: **0x3** \
Module name: **DemoName**

## s2m
For the converter to work, **solc** must be installed on the computer and accessible from the terminal by a short command **solc**
During the conversion, the solc utility compiles the sol code. The compiled sol code is stored in a temporary directory with a random name.
The received abi and bin files are translated into **move binary code**

### Install solc

How to install **solc** [see the documentation](https://docs.soliditylang.org/en/develop/installing-solidity.html)

### Checking solc

The **solc** version must be at least **0.8.15**

```bash
solc --version
```
> If this command is not available for execution from the terminal, s2m will not work.


### Installation
Cloning the repository and installing s2m:

```bash
git clone https://github.com/pontem-network/eth2move
cargo +nightly install --path crates/s2m
```

### See help:
```bash
s2m --help
```

### Input parameters
* `-p`, `--path`        Path to the sol file
* `-o`, `--output`      Where to save the converted file
* `--module`            The name of the module. If not specified, the name will be taken from the abi path
* `--address`           The address of the module [default: 0x1]
* `-m`, `--math`        Math backend u128 or u256 [default: u128]
* `-t`, `--trace`       Output of debugging information [possible values: true, false][defualt: false]
* `-h`, `--help`        Print help information
* `-V`, `--version`     Print version information

### Example
#### Required parameters
Required parameters are the paths to sol file (``--path``, ``-p``)

```bash
s2m -p path/to/file/NameFile.sol  
```

After completing the command, you will see the path to the created file (Example: "tmp/RANDOM_NAME/NameFile.mv").
Module address: **0x1**\
Module name: **NameFile**

#### Path to save

```bash
s2m -p path/to/file/NameFile.sol -o ./Demo.vm
```

The move binary file will be created in the current directory named **Demo.vm**\
Module address: **0x1** \
Module name: **NameFile**

#### Specifying the module name and address

```bash
s2m --module DemoName --address 0x3 -p path/to/file/NameFile.abi -o ./Demo.vm 
```

Module address: **0x3** \
Module name: **DemoName**
