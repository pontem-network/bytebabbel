use crate::abi::inc_ret_param::value::{AsParamValue, ParamValue};

// =================================================================================================
// INT
// =================================================================================================

impl AsParamValue for i8 {
    fn as_param(self) -> ParamValue
    where
        Self: Sized,
    {
        ParamValue::Int {
            size: 8,
            value: self as isize,
        }
    }

    fn try_as_param_int(self) -> anyhow::Result<ParamValue>
    where
        Self: Sized,
    {
        Ok(self.as_param())
    }
}

impl AsParamValue for i16 {
    fn as_param(self) -> ParamValue {
        ParamValue::Int {
            size: 16,
            value: self as isize,
        }
    }

    fn try_as_param_int(self) -> anyhow::Result<ParamValue>
    where
        Self: Sized,
    {
        Ok(self.as_param())
    }
}

impl AsParamValue for i32 {
    fn as_param(self) -> ParamValue {
        ParamValue::Int {
            size: 32,
            value: self as isize,
        }
    }

    fn try_as_param_int(self) -> anyhow::Result<ParamValue>
    where
        Self: Sized,
    {
        Ok(self.as_param())
    }
}

impl AsParamValue for i64 {
    fn as_param(self) -> ParamValue {
        ParamValue::Int {
            size: 64,
            value: self as isize,
        }
    }

    fn try_as_param_int(self) -> anyhow::Result<ParamValue>
    where
        Self: Sized,
    {
        Ok(self.as_param())
    }
}

impl AsParamValue for i128 {
    fn as_param(self) -> ParamValue {
        ParamValue::Int {
            size: 128,
            value: self as isize,
        }
    }

    fn try_as_param_int(self) -> anyhow::Result<ParamValue>
    where
        Self: Sized,
    {
        Ok(self.as_param())
    }
}

impl AsParamValue for isize {
    fn as_param(self) -> ParamValue {
        ParamValue::Int {
            size: 128,
            value: self as isize,
        }
    }

    fn try_as_param_int(self) -> anyhow::Result<ParamValue>
    where
        Self: Sized,
    {
        Ok(self.as_param())
    }
}

pub trait Int: AsParamValue {
    fn to_param_int(self) -> ParamValue
    where
        Self: Sized,
    {
        self.as_param()
    }
}

impl Int for i8 {}
impl Int for i16 {}
impl Int for i32 {}
impl Int for i64 {}
impl Int for i128 {}
impl Int for isize {}

pub trait I256: Int
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

impl<T: Int> I256 for T {}

// =================================================================================================
// UINT
// =================================================================================================

impl AsParamValue for u8 {
    fn as_param(self) -> ParamValue {
        ParamValue::UInt {
            size: 8,
            value: self as usize,
        }
    }

    fn try_as_param_uint(self) -> anyhow::Result<ParamValue>
    where
        Self: Sized,
    {
        Ok(self.as_param())
    }
}

impl AsParamValue for u16 {
    fn as_param(self) -> ParamValue {
        ParamValue::UInt {
            size: 16,
            value: self as usize,
        }
    }

    fn try_as_param_uint(self) -> anyhow::Result<ParamValue>
    where
        Self: Sized,
    {
        Ok(self.as_param())
    }
}

impl AsParamValue for u32 {
    fn as_param(self) -> ParamValue {
        ParamValue::UInt {
            size: 32,
            value: self as usize,
        }
    }

    fn try_as_param_uint(self) -> anyhow::Result<ParamValue>
    where
        Self: Sized,
    {
        Ok(self.as_param())
    }
}

impl AsParamValue for u64 {
    fn as_param(self) -> ParamValue {
        ParamValue::UInt {
            size: 64,
            value: self as usize,
        }
    }

    fn try_as_param_uint(self) -> anyhow::Result<ParamValue>
    where
        Self: Sized,
    {
        Ok(self.as_param())
    }
}

impl AsParamValue for u128 {
    fn as_param(self) -> ParamValue {
        ParamValue::UInt {
            size: 128,
            value: self as usize,
        }
    }

    fn try_as_param_uint(self) -> anyhow::Result<ParamValue>
    where
        Self: Sized,
    {
        Ok(self.as_param())
    }
}

impl AsParamValue for usize {
    fn as_param(self) -> ParamValue {
        ParamValue::UInt {
            size: 128,
            value: self as usize,
        }
    }

    fn try_as_param_uint(self) -> anyhow::Result<ParamValue>
    where
        Self: Sized,
    {
        Ok(self.as_param())
    }
}

pub trait UInt: AsParamValue {
    fn to_param_uint(self) -> ParamValue
    where
        Self: Sized,
    {
        self.as_param()
    }
}

impl UInt for u8 {}
impl UInt for u16 {}
impl UInt for u32 {}
impl UInt for u64 {}
impl UInt for u128 {}
impl UInt for usize {}

pub trait U256: UInt
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

impl<T: UInt> U256 for T {}

#[cfg(test)]
mod test {
    use crate::abi::inc_ret_param::value::number::{I256, U256};
    use crate::abi::inc_ret_param::value::{AsParamValue, ParamValue};

    #[test]
    fn test_to_param_int() {
        // i8
        assert_eq!(ParamValue::from(2i8), ParamValue::Int { size: 8, value: 2 });
        assert_eq!(3i8.as_param(), ParamValue::Int { size: 8, value: 3 });
        // i16
        assert_eq!(
            ParamValue::from(4i16),
            ParamValue::Int { size: 16, value: 4 }
        );
        assert_eq!(5i16.as_param(), ParamValue::Int { size: 16, value: 5 });
        // i32
        assert_eq!(
            ParamValue::from(6i32),
            ParamValue::Int { size: 32, value: 6 }
        );
        assert_eq!(7i32.as_param(), ParamValue::Int { size: 32, value: 7 });
        // i64
        assert_eq!(
            ParamValue::from(8i64),
            ParamValue::Int { size: 64, value: 8 }
        );
        assert_eq!(9i64.as_param(), ParamValue::Int { size: 64, value: 9 });

        // i128
        assert_eq!(
            ParamValue::from(10i128),
            ParamValue::Int {
                size: 128,
                value: 10
            }
        );
        assert_eq!(
            11i128.as_param(),
            ParamValue::Int {
                size: 128,
                value: 11
            }
        );
        // isize
        assert_eq!(
            ParamValue::from(12isize),
            ParamValue::Int {
                size: 128,
                value: 12
            }
        );
        assert_eq!(
            13isize.as_param(),
            ParamValue::Int {
                size: 128,
                value: 13
            }
        );
        // i256
        assert_eq!(
            14i8.to_param_i256(),
            ParamValue::Int {
                size: 256,
                value: 14
            }
        );
    }

    #[test]
    fn test_to_param_uint() {
        // u8
        assert_eq!(
            ParamValue::from(2u8),
            ParamValue::UInt { size: 8, value: 2 }
        );
        assert_eq!(3u8.as_param(), ParamValue::UInt { size: 8, value: 3 });
        // u16
        assert_eq!(
            ParamValue::from(4u16),
            ParamValue::UInt { size: 16, value: 4 }
        );
        assert_eq!(5u16.as_param(), ParamValue::UInt { size: 16, value: 5 });
        // u32
        assert_eq!(
            ParamValue::from(6u32),
            ParamValue::UInt { size: 32, value: 6 }
        );
        assert_eq!(7u32.as_param(), ParamValue::UInt { size: 32, value: 7 });
        // u64
        assert_eq!(
            ParamValue::from(8u64),
            ParamValue::UInt { size: 64, value: 8 }
        );
        assert_eq!(9u64.as_param(), ParamValue::UInt { size: 64, value: 9 });

        // u128
        assert_eq!(
            ParamValue::from(10u128),
            ParamValue::UInt {
                size: 128,
                value: 10
            }
        );
        assert_eq!(
            11u128.as_param(),
            ParamValue::UInt {
                size: 128,
                value: 11
            }
        );
        // usize
        assert_eq!(
            ParamValue::from(12usize),
            ParamValue::UInt {
                size: 128,
                value: 12
            }
        );
        assert_eq!(
            13usize.as_param(),
            ParamValue::UInt {
                size: 128,
                value: 13
            }
        );
        // u256
        assert_eq!(
            14usize.to_param_u256(),
            ParamValue::UInt {
                size: 256,
                value: 14
            }
        );
    }
}
