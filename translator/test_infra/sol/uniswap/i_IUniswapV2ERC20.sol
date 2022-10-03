// SPDX-License-Identifier: Apache-2.0


pragma solidity >=0.5.0;

interface IUniswapV2ERC20 {
    event Approval(address indexed owner, address indexed spender, uint128 value);
    event Transfer(address indexed from, address indexed to, uint128 value);

    function name() external pure returns (string memory);

    function symbol() external pure returns (string memory);

    function decimals() external pure returns (uint8);

    function totalSupply() external view returns (uint128);

    function balanceOf(address owner) external view returns (uint128);

    function allowance(address owner, address spender) external view returns (uint128);

    function approve(address spender, uint128 value) external returns (bool);

    function transfer(address to, uint128 value) external returns (bool);

    function transferFrom(address from, address to, uint128 value) external returns (bool);

    function DOMAIN_SEPARATOR() external view returns (bytes32);

    function PERMIT_TYPEHASH() external pure returns (bytes32);

    function nonces(address owner) external view returns (uint128);

    function permit(address owner, address spender, uint128 value, uint128 deadline, uint8 v, bytes32 r, bytes32 s) external;
}
