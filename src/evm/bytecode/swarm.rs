const SWARM_HASH_LENGTH: usize = 43;

const SWARM_HASH_PROGRAM_TRAILER: &[u8] = &[0x00, 0x29];
const SWARM_HASH_HEADER: &[u8] = &[0xa1, 0x65];

pub fn remove_swarm_hash(bytecode: &mut Vec<u8>) {
    let len = bytecode.len();
    if bytecode[len - 1] == SWARM_HASH_PROGRAM_TRAILER[1]
        && bytecode[len - 2] == SWARM_HASH_PROGRAM_TRAILER[0]
        && bytecode[len - 43] == SWARM_HASH_HEADER[0]
        && bytecode[len - 42] == SWARM_HASH_HEADER[1]
    {
        bytecode.resize(len - SWARM_HASH_LENGTH, 0);
    }
}
