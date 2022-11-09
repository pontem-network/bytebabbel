# Changelog

## [Unreleased]

## [0.0.4] - 2022-11-08

### Changed
- Update to aptos 1.0.1.
- Significantly reduced the size of the final mv file.

## [0.0.3] - 2022-10-18

### Added
- Support of 'do-while' loop.
- Support of `addmod`, `mulmod`
- Support of `SignExtend`
- Support <ACCOUNT>.balance, gasprice(), gaslimit(), block.number, block.timestamp, block.blockhash

### Fixed
- Verified exp
- Verified sar
- Verified byte

## [0.0.2] - 2022-10-04

### Added

- Support of 'for', 'while' loops.
- Added event decoder.
- Support of bool and uint parameters.
- Support of bool and uint literals.
- Support for branching operators 'IF'.
- Added changelog.md.

### Fixed

- Fixed u128 math.
- Verified smod, sdiv
- Updated tests for slt, sgt

### Changed

- Update to aptos 0.3.8.

### Removed
