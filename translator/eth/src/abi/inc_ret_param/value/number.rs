use crate::abi::inc_ret_param::value::{AsParamValue, ParamValue};
use evm_core::utils::I256;
use primitive_types::U256;

// =================================================================================================
// INT
// =================================================================================================

impl AsParamValue for i8 {
    fn to_param(self) -> ParamValue
    where
        Self: Sized,
    {
        let value = if self >= 0 {
            I256::from(U256::from(self as u8))
        } else {
            I256::from(U256::from(self.unsigned_abs())) / I256::minus_one()
        };
        ParamValue::Int { size: 8, value }
    }

    fn try_to_param_int(self) -> anyhow::Result<ParamValue>
    where
        Self: Sized,
    {
        Ok(self.to_param())
    }
}

impl AsParamValue for i16 {
    fn to_param(self) -> ParamValue {
        let value = if self >= 0 {
            I256::from(U256::from(self as u16))
        } else {
            I256::from(U256::from(self.unsigned_abs())) / I256::minus_one()
        };
        ParamValue::Int { size: 16, value }
    }

    fn try_to_param_int(self) -> anyhow::Result<ParamValue>
    where
        Self: Sized,
    {
        Ok(self.to_param())
    }
}

impl AsParamValue for i32 {
    fn to_param(self) -> ParamValue {
        let value = if self >= 0 {
            I256::from(U256::from(self as u32))
        } else {
            I256::from(U256::from(self.unsigned_abs())) / I256::minus_one()
        };
        ParamValue::Int { size: 32, value }
    }

    fn try_to_param_int(self) -> anyhow::Result<ParamValue>
    where
        Self: Sized,
    {
        Ok(self.to_param())
    }
}

impl AsParamValue for i64 {
    fn to_param(self) -> ParamValue {
        let value = if self >= 0 {
            I256::from(U256::from(self as u64))
        } else {
            I256::from(U256::from(self.unsigned_abs())) / I256::minus_one()
        };
        ParamValue::Int { size: 64, value }
    }

    fn try_to_param_int(self) -> anyhow::Result<ParamValue>
    where
        Self: Sized,
    {
        Ok(self.to_param())
    }
}

impl AsParamValue for i128 {
    fn to_param(self) -> ParamValue {
        let value = if self >= 0 {
            I256::from(U256::from(self as u128))
        } else {
            I256::from(U256::from(self.unsigned_abs())) / I256::minus_one()
        };
        ParamValue::Int { size: 128, value }
    }

    fn try_to_param_int(self) -> anyhow::Result<ParamValue>
    where
        Self: Sized,
    {
        Ok(self.to_param())
    }
}

impl AsParamValue for isize {
    fn to_param(self) -> ParamValue {
        let value = if self >= 0 {
            I256::from(U256::from(self as usize))
        } else {
            I256::from(U256::from(self.unsigned_abs())) / I256::minus_one()
        };
        ParamValue::Int { size: 128, value }
    }

    fn try_to_param_int(self) -> anyhow::Result<ParamValue>
    where
        Self: Sized,
    {
        Ok(self.to_param())
    }
}

pub trait Int: AsParamValue {
    fn to_param_int(self) -> ParamValue
    where
        Self: Sized,
    {
        self.to_param()
    }
}

impl Int for i8 {}
impl Int for i16 {}
impl Int for i32 {}
impl Int for i64 {}
impl Int for i128 {}
impl Int for isize {}

pub trait I256Param: Int
where
    Self: Sized,
{
    fn to_param_i256(self) -> ParamValue {
        let mut result = self.to_param_int();
        if let ParamValue::Int { ref mut size, .. } = result {
            *size = 256
        };
        result
    }
}

impl<T: Int> I256Param for T {}

// =================================================================================================
// UINT
// =================================================================================================

impl AsParamValue for u8 {
    fn to_param(self) -> ParamValue {
        ParamValue::UInt {
            size: 8,
            value: U256::from(self),
        }
    }

    fn try_to_param_uint(self) -> anyhow::Result<ParamValue>
    where
        Self: Sized,
    {
        Ok(self.to_param())
    }
}

impl AsParamValue for u16 {
    fn to_param(self) -> ParamValue {
        ParamValue::UInt {
            size: 16,
            value: U256::from(self),
        }
    }

    fn try_to_param_uint(self) -> anyhow::Result<ParamValue>
    where
        Self: Sized,
    {
        Ok(self.to_param())
    }
}

impl AsParamValue for u32 {
    fn to_param(self) -> ParamValue {
        ParamValue::UInt {
            size: 32,
            value: U256::from(self),
        }
    }

    fn try_to_param_uint(self) -> anyhow::Result<ParamValue>
    where
        Self: Sized,
    {
        Ok(self.to_param())
    }
}

impl AsParamValue for u64 {
    fn to_param(self) -> ParamValue {
        ParamValue::UInt {
            size: 64,
            value: U256::from(self),
        }
    }

    fn try_to_param_uint(self) -> anyhow::Result<ParamValue>
    where
        Self: Sized,
    {
        Ok(self.to_param())
    }
}

impl AsParamValue for u128 {
    fn to_param(self) -> ParamValue {
        ParamValue::UInt {
            size: 128,
            value: U256::from(self),
        }
    }

    fn try_to_param_uint(self) -> anyhow::Result<ParamValue>
    where
        Self: Sized,
    {
        Ok(self.to_param())
    }
}

impl AsParamValue for usize {
    fn to_param(self) -> ParamValue {
        ParamValue::UInt {
            size: 128,
            value: U256::from(self),
        }
    }

    fn try_to_param_uint(self) -> anyhow::Result<ParamValue>
    where
        Self: Sized,
    {
        Ok(self.to_param())
    }
}

pub trait UInt: AsParamValue {
    fn to_param_uint(self) -> ParamValue
    where
        Self: Sized,
    {
        self.to_param()
    }
}

impl UInt for u8 {}
impl UInt for u16 {}
impl UInt for u32 {}
impl UInt for u64 {}
impl UInt for u128 {}
impl UInt for usize {}

pub trait U256Param: UInt
where
    Self: Sized,
{
    fn to_param_u256(self) -> ParamValue {
        let mut result = self.to_param_uint();
        if let ParamValue::UInt { ref mut size, .. } = result {
            *size = 256
        };
        result
    }
}

impl<T: UInt> U256Param for T {}

#[cfg(test)]
mod test {
    use crate::abi::inc_ret_param::value::number::{I256Param, U256Param};
    use crate::abi::inc_ret_param::value::{AsParamValue, ParamValue};
    use evm_core::utils::I256;
    use primitive_types::U256;

    #[test]
    fn test_to_param_int() {
        // i8
        assert_eq!(
            ParamValue::from(2i8),
            ParamValue::Int {
                size: 8,
                value: I256::from(U256::from(2))
            }
        );
        assert_eq!(
            3i8.to_param(),
            ParamValue::Int {
                size: 8,
                value: I256::from(U256::from(3))
            }
        );
        // i16
        assert_eq!(
            ParamValue::from(4i16),
            ParamValue::Int {
                size: 16,
                value: I256::from(U256::from(4))
            }
        );
        assert_eq!(
            5i16.to_param(),
            ParamValue::Int {
                size: 16,
                value: I256::from(U256::from(5))
            }
        );
        // i32
        assert_eq!(
            ParamValue::from(6i32),
            ParamValue::Int {
                size: 32,
                value: I256::from(U256::from(6))
            }
        );
        assert_eq!(
            7i32.to_param(),
            ParamValue::Int {
                size: 32,
                value: I256::from(U256::from(7))
            }
        );
        // i64
        assert_eq!(
            ParamValue::from(8i64),
            ParamValue::Int {
                size: 64,
                value: I256::from(U256::from(8))
            }
        );
        assert_eq!(
            9i64.to_param(),
            ParamValue::Int {
                size: 64,
                value: I256::from(U256::from(9))
            }
        );

        // i128
        assert_eq!(
            ParamValue::from(10i128),
            ParamValue::Int {
                size: 128,
                value: I256::from(U256::from(10))
            }
        );
        assert_eq!(
            11i128.to_param(),
            ParamValue::Int {
                size: 128,
                value: I256::from(U256::from(11))
            }
        );
        // isize
        assert_eq!(
            ParamValue::from(12isize),
            ParamValue::Int {
                size: 128,
                value: I256::from(U256::from(12))
            }
        );
        assert_eq!(
            13isize.to_param(),
            ParamValue::Int {
                size: 128,
                value: I256::from(U256::from(13))
            }
        );
        // i256
        assert_eq!(
            14i8.to_param_i256(),
            ParamValue::Int {
                size: 256,
                value: I256::from(U256::from(14))
            }
        );
    }

    #[test]
    fn test_to_param_uint() {
        // u8
        assert_eq!(
            ParamValue::from(2u8),
            ParamValue::UInt {
                size: 8,
                value: U256::from(2)
            }
        );
        assert_eq!(
            3u8.to_param(),
            ParamValue::UInt {
                size: 8,
                value: U256::from(3)
            }
        );
        // u16
        assert_eq!(
            ParamValue::from(4u16),
            ParamValue::UInt {
                size: 16,
                value: U256::from(4)
            }
        );
        assert_eq!(
            5u16.to_param(),
            ParamValue::UInt {
                size: 16,
                value: U256::from(5)
            }
        );
        // u32
        assert_eq!(
            ParamValue::from(6u32),
            ParamValue::UInt {
                size: 32,
                value: U256::from(6)
            }
        );
        assert_eq!(
            7u32.to_param(),
            ParamValue::UInt {
                size: 32,
                value: U256::from(7)
            }
        );
        // u64
        assert_eq!(
            ParamValue::from(8u64),
            ParamValue::UInt {
                size: 64,
                value: U256::from(8)
            }
        );
        assert_eq!(
            9u64.to_param(),
            ParamValue::UInt {
                size: 64,
                value: U256::from(9)
            }
        );

        // u128
        assert_eq!(
            ParamValue::from(10u128),
            ParamValue::UInt {
                size: 128,
                value: U256::from(10)
            }
        );
        assert_eq!(
            11u128.to_param(),
            ParamValue::UInt {
                size: 128,
                value: U256::from(11)
            }
        );
        // usize
        assert_eq!(
            ParamValue::from(12usize),
            ParamValue::UInt {
                size: 128,
                value: U256::from(12)
            }
        );
        assert_eq!(
            13usize.to_param(),
            ParamValue::UInt {
                size: 128,
                value: U256::from(13)
            }
        );
        // u256
        assert_eq!(
            14usize.to_param_u256(),
            ParamValue::UInt {
                size: 256,
                value: U256::from(14),
            }
        );
    }
}
