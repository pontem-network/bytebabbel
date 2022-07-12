use crate::mv::function::code::intrinsic::math::Math;
use move_binary_format::file_format::{Bytecode, CodeOffset, LocalIndex, SignatureIndex};

const EU128_OVERFLOW: u64 = 1;

pub struct U256Math {
    pub tmp_u64: LocalIndex,
    pub tmp_u128: LocalIndex,
    pub vec_sig_index: SignatureIndex,
}

impl Math for U256Math {
    /// u128 -> u256 ([u64; 4])
    fn cast_from_u128(&self, bytecode: &mut Vec<Bytecode>, local: Option<LocalIndex>) {
        let input = if let Some(input) = local {
            input
        } else {
            bytecode.push(Bytecode::StLoc(self.tmp_u128));
            self.tmp_u128
        };
        bytecode.push(Bytecode::LdU64(0));
        bytecode.push(Bytecode::LdU64(0));
        bytecode.push(Bytecode::CopyLoc(input));
        bytecode.push(Bytecode::LdU8(64));
        bytecode.push(Bytecode::Shr);
        bytecode.push(Bytecode::CastU64);
        bytecode.push(Bytecode::CopyLoc(input));
        bytecode.push(Bytecode::LdU128(u64::MAX as u128));
        bytecode.push(Bytecode::BitAnd);
        bytecode.push(Bytecode::CastU64);
        bytecode.push(Bytecode::VecPack(self.vec_sig_index, 4));
    }

    /// u256([u64; 4]) -> u128   
    fn cast_to_u128(&self, bytecode: &mut Vec<Bytecode>) {
        bytecode.push(Bytecode::VecUnpack(self.vec_sig_index, 4));
        bytecode.push(Bytecode::CastU128);
        bytecode.push(Bytecode::StLoc(self.tmp_u128));
        bytecode.push(Bytecode::CastU128);
        bytecode.push(Bytecode::LdU8(64));
        bytecode.push(Bytecode::Shl);
        bytecode.push(Bytecode::CopyLoc(self.tmp_u128));
        bytecode.push(Bytecode::Add);
        bytecode.push(Bytecode::StLoc(self.tmp_u128));

        bytecode.push(Bytecode::StLoc(self.tmp_u64));
        bytecode.push(Bytecode::Pop);
        bytecode.push(Bytecode::CopyLoc(self.tmp_u64));
        bytecode.push(Bytecode::LdU64(0));
        bytecode.push(Bytecode::Eq);

        let pc = bytecode.len() as CodeOffset;
        bytecode.push(Bytecode::BrTrue(pc + 3));
        bytecode.push(Bytecode::LdU64(EU128_OVERFLOW));
        bytecode.push(Bytecode::Abort);
        bytecode.push(Bytecode::CopyLoc(self.tmp_u128));
    }
}

/*
module 1.U256 {
struct U256 has copy, drop, store {
        ret: vector<u64>
}

public add(a: U256, b: U256): U256 {
L0:     carry: u64
L1:     i: u64
L2:     is_overflow: bool
L3:     is_overflow1: bool
L4:     is_overflow2: bool
L5:     res: u64
L6:     res1: u64
L7:     res2: u64
L8:     ret: vector<u64>
B0:
        0: Call[0](empty<u64>(): vector<u64>)
        1: StLoc[12](ret: vector<u64>)
        2: LdU64(0)
        3: StLoc[4](carry: u64)
        4: LdU64(0)
        5: StLoc[5](i: u64)
B1:
        6: CopyLoc[5](i: u64)
        7: LdAddr[6](U64: [4, 0, 0, 0, 0, 0, 0, 0])
        8: Lt
        9: BrTrue(11)
B2:
        10: Branch(78)
B3:
        11: ImmBorrowLoc[0](a: U256)
        12: ImmBorrowField[0](U256.ret: vector<u64>)
        13: CopyLoc[5](i: u64)
        14: Call[1](borrow<u64>(&vector<u64>, u64): &u64)
        15: ReadRef
        16: StLoc[2](a1: u64)
        17: ImmBorrowLoc[1](b: U256)
        18: ImmBorrowField[0](U256.ret: vector<u64>)
        19: CopyLoc[5](i: u64)
        20: Call[1](borrow<u64>(&vector<u64>, u64): &u64)
        21: ReadRef
        22: StLoc[3](b1: u64)
        23: CopyLoc[4](carry: u64)
        24: LdU64(0)
        25: Neq
        26: BrTrue(28)
B4:
        27: Branch(58)
B5:
        28: MoveLoc[2](a1: u64)
        29: MoveLoc[3](b1: u64)
        30: Call[6](overflowing_add(u64, u64): u64 * bool)
        31: StLoc[7](is_overflow1: bool)
        32: StLoc[10](res1: u64)
        33: MoveLoc[10](res1: u64)
        34: MoveLoc[4](carry: u64)
        35: Call[6](overflowing_add(u64, u64): u64 * bool)
        36: StLoc[8](is_overflow2: bool)
        37: StLoc[11](res2: u64)
        38: MutBorrowLoc[12](ret: vector<u64>)
        39: MoveLoc[11](res2: u64)
        40: Call[2](push_back<u64>(&mut vector<u64>, u64))
        41: LdU64(0)
        42: StLoc[4](carry: u64)
        43: MoveLoc[7](is_overflow1: bool)
        44: BrTrue(46)
B6:
        45: Branch(50)
B7:
        46: MoveLoc[4](carry: u64)
        47: LdU64(1)
        48: Add
        49: StLoc[4](carry: u64)
B8:
        50: MoveLoc[8](is_overflow2: bool)
        51: BrTrue(53)
B9:
        52: Branch(57)
B10:
        53: MoveLoc[4](carry: u64)
        54: LdU64(1)
        55: Add
        56: StLoc[4](carry: u64)
B11:
        57: Branch(73)
B12:
        58: MoveLoc[2](a1: u64)
        59: MoveLoc[3](b1: u64)
        60: Call[6](overflowing_add(u64, u64): u64 * bool)
        61: StLoc[6](is_overflow: bool)
        62: StLoc[9](res: u64)
        63: MutBorrowLoc[12](ret: vector<u64>)
        64: MoveLoc[9](res: u64)
        65: Call[2](push_back<u64>(&mut vector<u64>, u64))
        66: LdU64(0)
        67: StLoc[4](carry: u64)
        68: MoveLoc[6](is_overflow: bool)
        69: BrTrue(71)
B13:
        70: Branch(73)
B14:
        71: LdU64(1)
        72: StLoc[4](carry: u64)
B15:
        73: MoveLoc[5](i: u64)
        74: LdU64(1)
        75: Add
        76: StLoc[5](i: u64)
        77: Branch(6)
B16:
        78: MoveLoc[12](ret: vector<u64>)
        79: Pack[0](U256)
        80: Ret
}
public as_u128(a: U256): u128 {
L0:     a2: u64
L1:     z: u64
B0:
        0: ImmBorrowLoc[0](a: U256)
        1: ImmBorrowField[0](U256.ret: vector<u64>)
        2: LdU64(0)
        3: Call[1](borrow<u64>(&vector<u64>, u64): &u64)
        4: ReadRef
        5: StLoc[1](a1: u64)
        6: ImmBorrowLoc[0](a: U256)
        7: ImmBorrowField[0](U256.ret: vector<u64>)
        8: LdU64(1)
        9: Call[1](borrow<u64>(&vector<u64>, u64): &u64)
        10: ReadRef
        11: StLoc[2](a2: u64)
        12: ImmBorrowLoc[0](a: U256)
        13: ImmBorrowField[0](U256.ret: vector<u64>)
        14: LdU64(2)
        15: Call[1](borrow<u64>(&vector<u64>, u64): &u64)
        16: ReadRef
        17: StLoc[3](z: u64)
        18: MoveLoc[3](z: u64)
        19: LdU64(0)
        20: Eq
        21: BrTrue(24)
B1:
        22: LdAddr[1](U64: [0, 0, 0, 0, 0, 0, 0, 0])
        23: Abort
B2:
        24: MoveLoc[2](a2: u64)
        25: CastU128
        26: LdU8(64)
        27: Shl
        28: MoveLoc[1](a1: u64)
        29: CastU128
        30: Add
        31: Ret
}
public compare(a: &U256, b: &U256): u8 {
L0:     i: u64
B0:
        0: LdAddr[6](U64: [4, 0, 0, 0, 0, 0, 0, 0])
        1: StLoc[4](i: u64)
B1:
        2: CopyLoc[4](i: u64)
        3: LdU64(0)
        4: Gt
        5: BrTrue(7)
B2:
        6: Branch(42)
B3:
        7: MoveLoc[4](i: u64)
        8: LdU64(1)
        9: Sub
        10: StLoc[4](i: u64)
        11: CopyLoc[0](a: &U256)
        12: ImmBorrowField[0](U256.ret: vector<u64>)
        13: CopyLoc[4](i: u64)
        14: Call[1](borrow<u64>(&vector<u64>, u64): &u64)
        15: ReadRef
        16: StLoc[2](a1: u64)
        17: CopyLoc[1](b: &U256)
        18: ImmBorrowField[0](U256.ret: vector<u64>)
        19: CopyLoc[4](i: u64)
        20: Call[1](borrow<u64>(&vector<u64>, u64): &u64)
        21: ReadRef
        22: StLoc[3](b1: u64)
        23: CopyLoc[2](a1: u64)
        24: CopyLoc[3](b1: u64)
        25: Neq
        26: BrTrue(28)
B4:
        27: Branch(41)
B5:
        28: MoveLoc[1](b: &U256)
        29: Pop
        30: MoveLoc[0](a: &U256)
        31: Pop
        32: MoveLoc[2](a1: u64)
        33: MoveLoc[3](b1: u64)
        34: Lt
        35: BrTrue(37)
B6:
        36: Branch(39)
B7:
        37: LdAddr[3](U8: [1])
        38: Ret
B8:
        39: LdAddr[2](U8: [2])
        40: Ret
B9:
        41: Branch(2)
B10:
        42: MoveLoc[1](b: &U256)
        43: Pop
        44: MoveLoc[0](a: &U256)
        45: Pop
        46: LdAddr[0](U8: [0])
        47: Ret
}
public from_u128(val: u128): U256 {
L0:     a2: u64
L1:     ret: vector<u64>
B0:
        0: MoveLoc[0](val: u128)
        1: Call[8](split_u128(u128): u64 * u64)
        2: StLoc[1](a1: u64)
        3: StLoc[2](a2: u64)
        4: MoveLoc[1](a1: u64)
        5: Call[3](singleton<u64>(u64): vector<u64>)
        6: StLoc[3](ret: vector<u64>)
        7: MutBorrowLoc[3](ret: vector<u64>)
        8: MoveLoc[2](a2: u64)
        9: Call[2](push_back<u64>(&mut vector<u64>, u64))
        10: MutBorrowLoc[3](ret: vector<u64>)
        11: LdU64(0)
        12: Call[2](push_back<u64>(&mut vector<u64>, u64))
        13: MutBorrowLoc[3](ret: vector<u64>)
        14: LdU64(0)
        15: Call[2](push_back<u64>(&mut vector<u64>, u64))
        16: MoveLoc[3](ret: vector<u64>)
        17: Pack[0](U256)
        18: Ret
}
public from_u64(val: u64): U256 {
B0:
        0: MoveLoc[0](val: u64)
        1: CastU128
        2: Call[3](from_u128(u128): U256)
        3: Ret
}
public mul(a: U256, b: U256): U256 {
L0:     %#8: u64
L1:     a1: u64
L2:     b1: u64
L3:     carry: u64
L4:     existing_hi: &mut u64
L5:     existing_low: &mut u64
L6:     final: vector<u64>
L7:     hi: u64
L8:     hi#5: u64
L9:     hi#6: u64
L10:    hi#7: u64
L11:    i: u64
L12:    i#1: u64
L13:    i#9: u64
L14:    j: u64
L15:    low: u64
L16:    low#3: u64
L17:    o: bool
L18:    o0: bool
L19:    o1: bool
L20:    overflow: u64
L21:    ret: vector<u64>
B0:
        0: Call[0](empty<u64>(): vector<u64>)
        1: StLoc[25](ret: vector<u64>)
        2: LdU64(0)
        3: StLoc[15](i: u64)
B1:
        4: CopyLoc[15](i: u64)
        5: LdAddr[6](U64: [4, 0, 0, 0, 0, 0, 0, 0])
        6: LdU64(2)
        7: Mul
        8: Lt
        9: BrTrue(11)
B2:
        10: Branch(19)
B3:
        11: MutBorrowLoc[25](ret: vector<u64>)
        12: LdU64(0)
        13: Call[2](push_back<u64>(&mut vector<u64>, u64))
        14: MoveLoc[15](i: u64)
        15: LdU64(1)
        16: Add
        17: StLoc[15](i: u64)
        18: Branch(4)
B4:
        19: LdU64(0)
        20: StLoc[16](i#1: u64)
B5:
        21: CopyLoc[16](i#1: u64)
        22: LdAddr[6](U64: [4, 0, 0, 0, 0, 0, 0, 0])
        23: Lt
        24: BrTrue(26)
B6:
        25: Branch(143)
B7:
        26: LdU64(0)
        27: StLoc[7](carry: u64)
        28: ImmBorrowLoc[1](b: U256)
        29: ImmBorrowField[0](U256.ret: vector<u64>)
        30: CopyLoc[16](i#1: u64)
        31: Call[1](borrow<u64>(&vector<u64>, u64): &u64)
        32: ReadRef
        33: StLoc[6](b1: u64)
        34: LdU64(0)
        35: StLoc[18](j: u64)
B8:
        36: CopyLoc[18](j: u64)
        37: LdAddr[6](U64: [4, 0, 0, 0, 0, 0, 0, 0])
        38: Lt
        39: BrTrue(41)
B9:
        40: Branch(138)
B10:
        41: ImmBorrowLoc[0](a: U256)
        42: ImmBorrowField[0](U256.ret: vector<u64>)
        43: CopyLoc[18](j: u64)
        44: Call[1](borrow<u64>(&vector<u64>, u64): &u64)
        45: ReadRef
        46: StLoc[5](a1: u64)
        47: CopyLoc[5](a1: u64)
        48: LdU64(0)
        49: Neq
        50: BrTrue(52)
B11:
        51: Branch(55)
B12:
        52: LdTrue
        53: StLoc[2](%#2: bool)
        54: Branch(59)
B13:
        55: CopyLoc[7](carry: u64)
        56: LdU64(0)
        57: Neq
        58: StLoc[2](%#2: bool)
B14:
        59: MoveLoc[2](%#2: bool)
        60: BrTrue(62)
B15:
        61: Branch(133)
B16:
        62: MoveLoc[5](a1: u64)
        63: CastU128
        64: CopyLoc[6](b1: u64)
        65: CastU128
        66: Mul
        67: Call[8](split_u128(u128): u64 * u64)
        68: StLoc[19](low: u64)
        69: StLoc[11](hi: u64)
        70: MutBorrowLoc[25](ret: vector<u64>)
        71: CopyLoc[16](i#1: u64)
        72: CopyLoc[18](j: u64)
        73: Add
        74: Call[4](borrow_mut<u64>(&mut vector<u64>, u64): &mut u64)
        75: StLoc[9](existing_low: &mut u64)
        76: MoveLoc[19](low: u64)
        77: CopyLoc[9](existing_low: &mut u64)
        78: ReadRef
        79: Call[6](overflowing_add(u64, u64): u64 * bool)
        80: StLoc[21](o: bool)
        81: StLoc[20](low#3: u64)
        82: MoveLoc[20](low#3: u64)
        83: MoveLoc[9](existing_low: &mut u64)
        84: WriteRef
        85: MoveLoc[21](o: bool)
        86: BrTrue(88)
B17:
        87: Branch(91)
B18:
        88: LdU64(1)
        89: StLoc[3](%#4: u64)
        90: Branch(93)
B19:
        91: LdU64(0)
        92: StLoc[3](%#4: u64)
B20:
        93: MoveLoc[3](%#4: u64)
        94: StLoc[24](overflow: u64)
        95: MutBorrowLoc[25](ret: vector<u64>)
        96: CopyLoc[16](i#1: u64)
        97: CopyLoc[18](j: u64)
        98: Add
        99: LdU64(1)
        100: Add
        101: Call[4](borrow_mut<u64>(&mut vector<u64>, u64): &mut u64)
        102: StLoc[8](existing_hi: &mut u64)
        103: MoveLoc[11](hi: u64)
        104: MoveLoc[24](overflow: u64)
        105: Add
        106: StLoc[12](hi#5: u64)
        107: MoveLoc[12](hi#5: u64)
        108: MoveLoc[7](carry: u64)
        109: Call[6](overflowing_add(u64, u64): u64 * bool)
        110: StLoc[22](o0: bool)
        111: StLoc[13](hi#6: u64)
        112: MoveLoc[13](hi#6: u64)
        113: CopyLoc[8](existing_hi: &mut u64)
        114: ReadRef
        115: Call[6](overflowing_add(u64, u64): u64 * bool)
        116: StLoc[23](o1: bool)
        117: StLoc[14](hi#7: u64)
        118: MoveLoc[14](hi#7: u64)
        119: MoveLoc[8](existing_hi: &mut u64)
        120: WriteRef
        121: MoveLoc[22](o0: bool)
        122: MoveLoc[23](o1: bool)
        123: Or
        124: BrTrue(126)
B21:
        125: Branch(129)
B22:
        126: LdU64(1)
        127: StLoc[4](%#8: u64)
        128: Branch(131)
B23:
        129: LdU64(0)
        130: StLoc[4](%#8: u64)
B24:
        131: MoveLoc[4](%#8: u64)
        132: StLoc[7](carry: u64)
B25:
        133: MoveLoc[18](j: u64)
        134: LdU64(1)
        135: Add
        136: StLoc[18](j: u64)
        137: Branch(36)
B26:
        138: MoveLoc[16](i#1: u64)
        139: LdU64(1)
        140: Add
        141: StLoc[16](i#1: u64)
        142: Branch(21)
B27:
        143: Call[0](empty<u64>(): vector<u64>)
        144: StLoc[10](final: vector<u64>)
        145: LdU64(0)
        146: StLoc[17](i#9: u64)
B28:
        147: CopyLoc[17](i#9: u64)
        148: LdAddr[6](U64: [4, 0, 0, 0, 0, 0, 0, 0])
        149: Lt
        150: BrTrue(152)
B29:
        151: Branch(163)
B30:
        152: MutBorrowLoc[10](final: vector<u64>)
        153: ImmBorrowLoc[25](ret: vector<u64>)
        154: CopyLoc[17](i#9: u64)
        155: Call[1](borrow<u64>(&vector<u64>, u64): &u64)
        156: ReadRef
        157: Call[2](push_back<u64>(&mut vector<u64>, u64))
        158: MoveLoc[17](i#9: u64)
        159: LdU64(1)
        160: Add
        161: StLoc[17](i#9: u64)
        162: Branch(147)
B31:
        163: MoveLoc[10](final: vector<u64>)
        164: Pack[0](U256)
        165: Ret
}
overflowing_add(a: u64, b: u64): u64 * bool {
L0:     a128: u128
L1:     b128: u128
L2:     overflow: u128
L3:     r: u128
B0:
        0: MoveLoc[0](a: u64)
        1: CastU128
        2: StLoc[4](a128: u128)
        3: MoveLoc[1](b: u64)
        4: CastU128
        5: StLoc[5](b128: u128)
        6: CopyLoc[4](a128: u128)
        7: CopyLoc[5](b128: u128)
        8: Add
        9: StLoc[7](r: u128)
        10: CopyLoc[7](r: u128)
        11: LdAddr[5](U128: [255, 255, 255, 255, 255, 255, 255, 255, 0, 0, 0, 0, 0, 0, 0, 0])
        12: Gt
        13: BrTrue(15)
B1:
        14: Branch(27)
B2:
        15: MoveLoc[7](r: u128)
        16: LdAddr[5](U128: [255, 255, 255, 255, 255, 255, 255, 255, 0, 0, 0, 0, 0, 0, 0, 0])
        17: Sub
        18: LdU128(1)
        19: Sub
        20: StLoc[6](overflow: u128)
        21: MoveLoc[6](overflow: u128)
        22: CastU64
        23: LdTrue
        24: StLoc[3](%#2: bool)
        25: StLoc[2](%#1: u64)
        26: Branch(34)
B3:
        27: MoveLoc[4](a128: u128)
        28: MoveLoc[5](b128: u128)
        29: Add
        30: CastU64
        31: LdFalse
        32: StLoc[3](%#2: bool)
        33: StLoc[2](%#1: u64)
B4:
        34: MoveLoc[2](%#1: u64)
        35: MoveLoc[3](%#2: bool)
        36: Ret
}
overflowing_sub(a: u64, b: u64): u64 * bool {
L0:     r: u64
B0:
        0: CopyLoc[0](a: u64)
        1: CopyLoc[1](b: u64)
        2: Lt
        3: BrTrue(5)
B1:
        4: Branch(19)
B2:
        5: MoveLoc[1](b: u64)
        6: MoveLoc[0](a: u64)
        7: Sub
        8: StLoc[4](r: u64)
        9: LdAddr[5](U128: [255, 255, 255, 255, 255, 255, 255, 255, 0, 0, 0, 0, 0, 0, 0, 0])
        10: CastU64
        11: MoveLoc[4](r: u64)
        12: Sub
        13: LdU64(1)
        14: Add
        15: LdTrue
        16: StLoc[3](%#2: bool)
        17: StLoc[2](%#1: u64)
        18: Branch(25)
B3:
        19: MoveLoc[0](a: u64)
        20: MoveLoc[1](b: u64)
        21: Sub
        22: LdFalse
        23: StLoc[3](%#2: bool)
        24: StLoc[2](%#1: u64)
B4:
        25: MoveLoc[2](%#1: u64)
        26: MoveLoc[3](%#2: bool)
        27: Ret
}
split_u128(a: u128): u64 * u64 {
L0:     a2: u64
B0:
        0: CopyLoc[0](a: u128)
        1: LdU8(64)
        2: Shr
        3: CastU64
        4: StLoc[1](a1: u64)
        5: MoveLoc[0](a: u128)
        6: LdU128(18446744073709551615)
        7: BitAnd
        8: CastU64
        9: StLoc[2](a2: u64)
        10: MoveLoc[1](a1: u64)
        11: MoveLoc[2](a2: u64)
        12: Ret
}
public sub(a: U256, b: U256): U256 {
L0:     carry: u64
L1:     i: u64
L2:     is_overflow: bool
L3:     is_overflow1: bool
L4:     is_overflow2: bool
L5:     res: u64
L6:     res1: u64
L7:     res2: u64
L8:     ret: vector<u64>
B0:
        0: Call[0](empty<u64>(): vector<u64>)
        1: StLoc[12](ret: vector<u64>)
        2: LdU64(0)
        3: StLoc[4](carry: u64)
        4: LdU64(0)
        5: StLoc[5](i: u64)
B1:
        6: CopyLoc[5](i: u64)
        7: LdAddr[6](U64: [4, 0, 0, 0, 0, 0, 0, 0])
        8: Lt
        9: BrTrue(11)
B2:
        10: Branch(78)
B3:
        11: ImmBorrowLoc[0](a: U256)
        12: ImmBorrowField[0](U256.ret: vector<u64>)
        13: CopyLoc[5](i: u64)
        14: Call[1](borrow<u64>(&vector<u64>, u64): &u64)
        15: ReadRef
        16: StLoc[2](a1: u64)
        17: ImmBorrowLoc[1](b: U256)
        18: ImmBorrowField[0](U256.ret: vector<u64>)
        19: CopyLoc[5](i: u64)
        20: Call[1](borrow<u64>(&vector<u64>, u64): &u64)
        21: ReadRef
        22: StLoc[3](b1: u64)
        23: CopyLoc[4](carry: u64)
        24: LdU64(0)
        25: Neq
        26: BrTrue(28)
B4:
        27: Branch(58)
B5:
        28: MoveLoc[2](a1: u64)
        29: MoveLoc[3](b1: u64)
        30: Call[7](overflowing_sub(u64, u64): u64 * bool)
        31: StLoc[7](is_overflow1: bool)
        32: StLoc[10](res1: u64)
        33: MoveLoc[10](res1: u64)
        34: MoveLoc[4](carry: u64)
        35: Call[7](overflowing_sub(u64, u64): u64 * bool)
        36: StLoc[8](is_overflow2: bool)
        37: StLoc[11](res2: u64)
        38: MutBorrowLoc[12](ret: vector<u64>)
        39: MoveLoc[11](res2: u64)
        40: Call[2](push_back<u64>(&mut vector<u64>, u64))
        41: LdU64(0)
        42: StLoc[4](carry: u64)
        43: MoveLoc[7](is_overflow1: bool)
        44: BrTrue(46)
B6:
        45: Branch(50)
B7:
        46: MoveLoc[4](carry: u64)
        47: LdU64(1)
        48: Add
        49: StLoc[4](carry: u64)
B8:
        50: MoveLoc[8](is_overflow2: bool)
        51: BrTrue(53)
B9:
        52: Branch(57)
B10:
        53: MoveLoc[4](carry: u64)
        54: LdU64(1)
        55: Add
        56: StLoc[4](carry: u64)
B11:
        57: Branch(73)
B12:
        58: MoveLoc[2](a1: u64)
        59: MoveLoc[3](b1: u64)
        60: Call[7](overflowing_sub(u64, u64): u64 * bool)
        61: StLoc[6](is_overflow: bool)
        62: StLoc[9](res: u64)
        63: MutBorrowLoc[12](ret: vector<u64>)
        64: MoveLoc[9](res: u64)
        65: Call[2](push_back<u64>(&mut vector<u64>, u64))
        66: LdU64(0)
        67: StLoc[4](carry: u64)
        68: MoveLoc[6](is_overflow: bool)
        69: BrTrue(71)
B13:
        70: Branch(73)
B14:
        71: LdU64(1)
        72: StLoc[4](carry: u64)
B15:
        73: MoveLoc[5](i: u64)
        74: LdU64(1)
        75: Add
        76: StLoc[5](i: u64)
        77: Branch(6)
B16:
        78: MoveLoc[12](ret: vector<u64>)
        79: Pack[0](U256)
        80: Ret
}
public zero(): U256 {
L0:     ret: vector<u64>
B0:
        0: Call[0](empty<u64>(): vector<u64>)
        1: StLoc[0](ret: vector<u64>)
        2: MutBorrowLoc[0](ret: vector<u64>)
        3: LdU64(0)
        4: Call[2](push_back<u64>(&mut vector<u64>, u64))
        5: MutBorrowLoc[0](ret: vector<u64>)
        6: LdU64(0)
        7: Call[2](push_back<u64>(&mut vector<u64>, u64))
        8: MutBorrowLoc[0](ret: vector<u64>)
        9: LdU64(0)
        10: Call[2](push_back<u64>(&mut vector<u64>, u64))
        11: MutBorrowLoc[0](ret: vector<u64>)
        12: LdU64(0)
        13: Call[2](push_back<u64>(&mut vector<u64>, u64))
        14: MoveLoc[0](ret: vector<u64>)
        15: Pack[0](U256)
        16: Ret
}
}

*/
