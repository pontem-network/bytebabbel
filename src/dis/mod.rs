//! Simple EVM-bytecode disassembler.

use std::collections::BTreeMap;
use std::io::Cursor;
use std::io::Read;
pub use error::Error;
pub use ops::Instruction;
use ops::read_next_byte;

pub mod error;
pub mod ops;

pub fn read_hex<S: AsRef<str>>(input: S) -> Result<BTreeMap<usize, Instruction>, Error> {
    const HEX_PREFIX: &str = "0x";
    let input = input.as_ref();
    let input = if input[0..2] == *HEX_PREFIX {
        &input[(HEX_PREFIX.len())..]
    } else {
        input
    };
    read(hex::decode(input)?.as_slice())
}

pub fn read<R: Read + AsRef<[u8]>>(bytes: R) -> Result<BTreeMap<usize, Instruction>, Error> {
    let mut cursor = Cursor::new(bytes);
    let mut instructions = BTreeMap::new();

    loop {
        let result = read_next_byte(&mut cursor);
        match result {
            Err(Error::Io(_)) => break,
            Ok((offset, instruction)) => {
                instructions.insert(offset, instruction);
            }
            Err(err) => {
                if let Error::TooFewBytesForPush = err {
                    // the solidity compiler sometimes puts push instructions at the end, however,
                    // this is considered normal behaviour
                    break;
                }
                return Err(err);
            }
        }
    }

    Ok(instructions)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::error::Error;

    #[test]
    fn hex() -> Result<(), Error> {
        let program = "0x608040526002610100";
        let program_bytes = vec![0x60, 0x80, 0x40, 0x52, 0x60, 0x02, 0x61, 0x01, 0x00];
        let disas = BTreeMap::from_iter(maplit::hashmap! {
             0 => Instruction::Push(vec!(0x80)),
             2 => Instruction::Blockhash,
             3 => Instruction::MStore,
             4 => Instruction::Push(vec!(0x2)),
             6 => Instruction::Push(vec!(0x1, 0x00)),
        });

        assert_eq!(read_hex(program)?, disas);
        assert_eq!(read(&program_bytes[..])?, disas);

        Ok(())
    }

    #[test]
    fn a_plus_b() -> Result<(), Error> {
        const BIN: &[u8] = include_bytes!(concat!("../../", env!("APlusB")));
        read(&BIN[..])?;
        Ok(())
    }

    #[test]
    fn two_functions() -> Result<(), Error> {
        const BIN: &[u8] = include_bytes!(concat!("../../", env!("TwoFunctions")));
        read(&BIN[..])?;
        Ok(())
    }
}
