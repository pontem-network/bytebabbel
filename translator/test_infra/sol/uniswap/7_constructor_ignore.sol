// SPDX-License-Identifier: Apache-2.0

pragma solidity >=0.8.10;

contract TestConstruct {
    string public constant name = 'Uniswap V2';
    bytes32 public DOMAIN_SEPARATOR;

    constructor() {
        uint128 chainId = 0;
        DOMAIN_SEPARATOR = keccak256(
            abi.encode(
                keccak256('EIP712Domain(string name,string version,uint256 chainId,address verifyingContract)'),
                keccak256(bytes(name)),
                keccak256(bytes('1')),
                chainId,
                address(this)
            )
        );
    }

    // # get() 0x1
    function get() public view returns (bytes32){
        return DOMAIN_SEPARATOR;
    }

}
