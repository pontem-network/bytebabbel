use crate::abi::inc_ret_param::types::ParamType;
use crate::abi::inc_ret_param::value::{AsParamValue, ParamValue};
use anyhow::{anyhow, bail, ensure, Error, Result};

// =================================================================================================
// Array
// =================================================================================================
impl<T> AsParamValue for Vec<T>
where
    T: AsParamValue,
{
    fn as_param(self) -> ParamValue {
        ParamValue::Array(self.into_iter().map(|item| item.as_param()).collect())
    }

    /// Transformation together with checking the length of vectors
    fn try_as_param_array(self) -> Result<ParamValue>
    where
        Self: Sized,
    {
        let param = self.as_param();
        param.to_type()?;
        Ok(param)
    }

    fn try_as_array_by_type(self, tp: &ParamType) -> Result<ParamValue>
    where
        Self: Sized,
    {
        let (size, child_type) = match tp {
            ParamType::Array {
                size,
                tp: child_type,
            } => (size, child_type),
            _ => bail!("Expected {tp:?} type"),
        };

        let value_size = self.len();
        if let Some(size) = size {
            ensure!(
                value_size == *size as usize,
                "An array of length [{size};N] was expected"
            )
        }

        let value = self
            .into_iter()
            .map(|value| child_type.set_value(value))
            .collect::<Result<Vec<ParamValue>>>()?;

        Ok(ParamValue::Array(value))
    }

    fn try_vec_u8(self) -> Result<Vec<u8>>
    where
        Self: Sized,
    {
        let v = if let ParamValue::Array(v) = self.as_param() {
            v
        } else {
            bail!("Expected [u8;N]");
        };
        v.into_iter()
            .map(|v| match v {
                ParamValue::Int { value, .. } => Ok(value as u8),
                ParamValue::UInt { value, .. } => Ok(value as u8),
                _ => Err(anyhow!("Expected [u8;N]")),
            })
            .collect::<Result<Vec<u8>>>()
    }
}

impl<T> AsParamValue for &[T]
where
    T: AsParamValue + Clone,
{
    fn as_param(self) -> ParamValue {
        self.to_vec().as_param()
    }

    fn try_as_param_array(self) -> Result<ParamValue>
    where
        Self: Sized,
    {
        self.to_vec().try_as_param_array()
    }

    fn try_as_array_by_type(self, tp: &ParamType) -> Result<ParamValue>
    where
        Self: Sized,
    {
        self.to_vec().try_as_array_by_type(tp)
    }

    fn try_vec_u8(self) -> Result<Vec<u8>>
    where
        Self: Sized,
    {
        self.to_vec().try_vec_u8()
    }
}

impl<T, const N: usize> AsParamValue for [T; N]
where
    T: AsParamValue + Clone,
{
    fn as_param(self) -> ParamValue {
        self.to_vec().as_param()
    }

    fn try_as_param_array(self) -> Result<ParamValue>
    where
        Self: Sized,
    {
        self.to_vec().try_as_param_array()
    }

    fn try_as_array_by_type(self, tp: &ParamType) -> Result<ParamValue>
    where
        Self: Sized,
    {
        self.to_vec().try_as_array_by_type(tp)
    }

    fn try_vec_u8(self) -> Result<Vec<u8>>
    where
        Self: Sized,
    {
        self.to_vec().try_vec_u8()
    }
}

// =================================================================================================
// Bytes
// =================================================================================================
pub trait TryParamBytes {
    fn lenght(&self) -> usize;

    fn to_vector(self) -> Vec<u8>;

    fn try_as_param_bytes(self) -> ParamValue
    where
        Self: Sized,
    {
        let vec = self.to_vector();
        ParamValue::Bytes(vec)
    }

    fn try_as_param_bytes_with_size(self, size: usize) -> Result<ParamValue>
    where
        Self: Sized,
    {
        if size < 1 || size > 32 {
            bail!("The length of the array must be >= 1 and <= 32");
        }

        let len = self.lenght();
        if size != len {
            bail!("An array of length {size} was expected. An array of length {len} was passed")
        }
        Ok(ParamValue::Byte(self.to_vector()))
    }

    fn try_as_param_bytes1(self) -> Result<ParamValue>
    where
        Self: Sized,
    {
        self.try_as_param_bytes_with_size(1)
    }

    fn try_as_param_bytes2(self) -> Result<ParamValue>
    where
        Self: Sized,
    {
        self.try_as_param_bytes_with_size(2)
    }

    fn try_as_param_bytes3(self) -> Result<ParamValue>
    where
        Self: Sized,
    {
        self.try_as_param_bytes_with_size(3)
    }

    fn try_as_param_bytes4(self) -> Result<ParamValue>
    where
        Self: Sized,
    {
        self.try_as_param_bytes_with_size(4)
    }

    fn try_as_param_bytes5(self) -> Result<ParamValue>
    where
        Self: Sized,
    {
        self.try_as_param_bytes_with_size(5)
    }

    fn try_as_param_bytes6(self) -> Result<ParamValue>
    where
        Self: Sized,
    {
        self.try_as_param_bytes_with_size(6)
    }

    fn try_as_param_bytes7(self) -> Result<ParamValue>
    where
        Self: Sized,
    {
        self.try_as_param_bytes_with_size(7)
    }

    fn try_as_param_bytes8(self) -> Result<ParamValue>
    where
        Self: Sized,
    {
        self.try_as_param_bytes_with_size(8)
    }

    fn try_as_param_bytes9(self) -> Result<ParamValue>
    where
        Self: Sized,
    {
        self.try_as_param_bytes_with_size(9)
    }

    fn try_as_param_bytes10(self) -> Result<ParamValue>
    where
        Self: Sized,
    {
        self.try_as_param_bytes_with_size(10)
    }

    fn try_as_param_bytes11(self) -> Result<ParamValue>
    where
        Self: Sized,
    {
        self.try_as_param_bytes_with_size(11)
    }

    fn try_as_param_bytes12(self) -> Result<ParamValue>
    where
        Self: Sized,
    {
        self.try_as_param_bytes_with_size(12)
    }

    fn try_as_param_bytes13(self) -> Result<ParamValue>
    where
        Self: Sized,
    {
        self.try_as_param_bytes_with_size(13)
    }

    fn try_as_param_bytes14(self) -> Result<ParamValue>
    where
        Self: Sized,
    {
        self.try_as_param_bytes_with_size(14)
    }

    fn try_as_param_bytes15(self) -> Result<ParamValue>
    where
        Self: Sized,
    {
        self.try_as_param_bytes_with_size(15)
    }

    fn try_as_param_bytes16(self) -> Result<ParamValue>
    where
        Self: Sized,
    {
        self.try_as_param_bytes_with_size(16)
    }

    fn try_as_param_bytes17(self) -> Result<ParamValue>
    where
        Self: Sized,
    {
        self.try_as_param_bytes_with_size(17)
    }

    fn try_as_param_bytes18(self) -> Result<ParamValue>
    where
        Self: Sized,
    {
        self.try_as_param_bytes_with_size(18)
    }

    fn try_as_param_bytes19(self) -> Result<ParamValue>
    where
        Self: Sized,
    {
        self.try_as_param_bytes_with_size(19)
    }

    fn try_as_param_bytes20(self) -> Result<ParamValue>
    where
        Self: Sized,
    {
        self.try_as_param_bytes_with_size(20)
    }

    fn try_as_param_bytes21(self) -> Result<ParamValue>
    where
        Self: Sized,
    {
        self.try_as_param_bytes_with_size(21)
    }

    fn try_as_param_bytes22(self) -> Result<ParamValue>
    where
        Self: Sized,
    {
        self.try_as_param_bytes_with_size(22)
    }

    fn try_as_param_bytes23(self) -> Result<ParamValue>
    where
        Self: Sized,
    {
        self.try_as_param_bytes_with_size(23)
    }

    fn try_as_param_bytes24(self) -> Result<ParamValue>
    where
        Self: Sized,
    {
        self.try_as_param_bytes_with_size(24)
    }

    fn try_as_param_bytes25(self) -> Result<ParamValue>
    where
        Self: Sized,
    {
        self.try_as_param_bytes_with_size(25)
    }

    fn try_as_param_bytes26(self) -> Result<ParamValue>
    where
        Self: Sized,
    {
        self.try_as_param_bytes_with_size(26)
    }

    fn try_as_param_bytes27(self) -> Result<ParamValue>
    where
        Self: Sized,
    {
        self.try_as_param_bytes_with_size(27)
    }

    fn try_as_param_bytes28(self) -> Result<ParamValue>
    where
        Self: Sized,
    {
        self.try_as_param_bytes_with_size(28)
    }

    fn try_as_param_bytes29(self) -> Result<ParamValue>
    where
        Self: Sized,
    {
        self.try_as_param_bytes_with_size(29)
    }

    fn try_as_param_bytes30(self) -> Result<ParamValue>
    where
        Self: Sized,
    {
        self.try_as_param_bytes_with_size(30)
    }

    fn try_as_param_bytes31(self) -> Result<ParamValue>
    where
        Self: Sized,
    {
        self.try_as_param_bytes_with_size(31)
    }

    fn try_as_param_bytes32(self) -> Result<ParamValue>
    where
        Self: Sized,
    {
        self.try_as_param_bytes_with_size(32)
    }
}

impl TryParamBytes for Vec<u8> {
    fn lenght(&self) -> usize {
        self.len()
    }

    fn to_vector(self) -> Vec<u8> {
        self
    }
}

impl<const N: usize> TryParamBytes for [u8; N] {
    fn lenght(&self) -> usize {
        N
    }

    fn to_vector(self) -> Vec<u8> {
        self.to_vec()
    }
}

impl TryParamBytes for &[u8] {
    fn lenght(&self) -> usize {
        self.len()
    }

    fn to_vector(self) -> Vec<u8> {
        self.to_vec()
    }
}

// =================================================================================================
// Address
// =================================================================================================
pub trait TryParamAddress {
    fn try_as_param_address(self) -> Result<ParamValue>
    where
        Self: Sized;
}

impl TryParamAddress for &[u8] {
    fn try_as_param_address(self) -> Result<ParamValue>
    where
        Self: Sized,
    {
        let len = self.lenght();
        if len != 32 {
            bail!("The length of the address must be 32 bytes. An array of {len} bytes was passed");
        }
        let mut value = [0u8; 32];
        value.clone_from_slice(self);
        Ok(ParamValue::Address(value))
    }
}

impl<const N: usize> TryParamAddress for [u8; N] {
    fn try_as_param_address(self) -> Result<ParamValue>
    where
        Self: Sized,
    {
        self.as_slice().try_as_param_address()
    }
}

impl TryParamAddress for Vec<u8> {
    fn try_as_param_address(self) -> Result<ParamValue>
    where
        Self: Sized,
    {
        self.as_slice().try_as_param_address()
    }
}

#[cfg(test)]
mod test {
    use crate::abi::inc_ret_param::value::collection::{TryParamAddress, TryParamBytes};
    use crate::abi::inc_ret_param::value::{AsParamValue, ParamValue};

    #[test]
    fn test_to_param_bytes() {
        // bytes
        let bytes = vec![1, 2, 3, 4, 5];

        assert_eq!(
            ParamValue::from(bytes.clone().try_vec_u8().unwrap().try_as_param_bytes()),
            ParamValue::Bytes(bytes.clone())
        );
        assert_eq!(
            bytes.clone().try_as_param_bytes(),
            ParamValue::Bytes(bytes.clone())
        );

        assert_eq!(
            [1, 2, 3].try_as_param_bytes(),
            ParamValue::Bytes(vec![1, 2, 3])
        );
        assert_eq!(
            [1, 2, 3].as_slice().try_as_param_bytes(),
            ParamValue::Bytes(vec![1, 2, 3])
        );

        assert_eq!(
            vec![1].try_as_param_bytes1().unwrap(),
            ParamValue::Byte(vec![1])
        );
        assert_eq!(
            [1, 2].try_as_param_bytes2().unwrap(),
            ParamValue::Byte(vec![1, 2])
        );
        assert_eq!(
            [1, 2, 3].as_slice().try_as_param_bytes3().unwrap(),
            ParamValue::Byte(vec![1, 2, 3])
        );

        assert_eq!(
            vec![
                1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23,
                24, 25, 26, 27, 28, 29, 30, 31, 32
            ]
            .try_as_param_bytes32()
            .unwrap(),
            ParamValue::Byte(vec![
                1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23,
                24, 25, 26, 27, 28, 29, 30, 31, 32
            ])
        );

        assert!(vec![1, 2].try_as_param_bytes1().is_err());
    }

    #[test]
    fn try_as_param_address() {
        assert_eq!(
            [0; 32].try_as_param_address().unwrap(),
            ParamValue::Address([0; 32])
        );
        let arr = [0; 32];
        assert_eq!(
            { &arr }.try_as_param_address().unwrap(),
            ParamValue::Address([0; 32])
        );
        let v = (0..32).map(|_| 0).collect::<Vec<u8>>();
        assert_eq!(
            v.try_as_param_address().unwrap(),
            ParamValue::Address([0; 32])
        );

        assert!([0; 1].try_as_param_address().is_err());
    }

    #[test]
    fn try_as_param_array() {
        assert_eq!(
            vec![1u16].as_param(),
            ParamValue::Array(vec![ParamValue::UInt { size: 16, value: 1 }])
        );
        assert_eq!(
            [1i32].as_slice().as_param(),
            ParamValue::Array(vec![ParamValue::Int { size: 32, value: 1 }])
        );
        assert_eq!(
            [1u64].as_param(),
            ParamValue::Array(vec![ParamValue::UInt { size: 64, value: 1 }])
        );

        assert_eq!(
            [1u8, 2].as_param(),
            ParamValue::Array(vec![
                ParamValue::UInt { size: 8, value: 1 },
                ParamValue::UInt { size: 8, value: 2 }
            ])
        );
        assert_eq!(
            [[1i8, 2], [3, 4]].as_param(),
            ParamValue::Array(vec![
                ParamValue::Array(vec![
                    ParamValue::Int { size: 8, value: 1 },
                    ParamValue::Int { size: 8, value: 2 }
                ]),
                ParamValue::Array(vec![
                    ParamValue::Int { size: 8, value: 3 },
                    ParamValue::Int { size: 8, value: 4 }
                ])
            ])
        );

        assert_eq!(
            vec![vec![1u8], vec![2]].as_param(),
            ParamValue::Array(vec![
                ParamValue::Array(vec![ParamValue::UInt { size: 8, value: 1 },]),
                ParamValue::Array(vec![ParamValue::UInt { size: 8, value: 2 },])
            ])
        );
        assert!(vec![vec![1u8], vec![2, 3]].try_as_param_array().is_err());
        assert!([1].try_as_param_array().is_ok());

        assert_eq!(
            [[["1"]]].as_param(),
            ParamValue::Array(vec![ParamValue::Array(vec![ParamValue::Array(vec![
                ParamValue::String("1".as_bytes().to_vec(),)
            ])])])
        );
    }
}
