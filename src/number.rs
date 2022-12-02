use crate::LunifyError;

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) enum Number {
    /// Lua non-integral number type.
    Float(f64),
    /// Lua integral number type.
    Integer(i64),
}

impl Number {
    pub(crate) fn as_integer(self) -> Result<i64, LunifyError> {
        match self {
            Number::Integer(value) => Ok(value),
            Number::Float(value) => match value == value.round() {
                true => Ok(value as i64),
                false => Err(LunifyError::FloatPrecisionLoss),
            },
        }
    }

    pub(crate) fn as_float(self) -> Result<f64, LunifyError> {
        match self {
            Number::Float(value) => Ok(value),
            Number::Integer(value) => match value < f64::MAX as i64 {
                true => Ok(value as f64),
                false => Err(LunifyError::IntegerOverflow),
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::Number;
    use crate::LunifyError;

    #[test]
    fn integer_as_integer() {
        let number = Number::Integer(10);
        assert_eq!(number.as_integer(), Ok(10))
    }

    #[test]
    fn integer_as_float() {
        let number = Number::Integer(10);
        assert_eq!(number.as_float(), Ok(10.0))
    }

    #[test]
    fn float_as_float() {
        let number = Number::Float(10.0);
        assert_eq!(number.as_float(), Ok(10.0))
    }

    #[test]
    fn float_as_integer() {
        let number = Number::Float(10.0);
        assert_eq!(number.as_integer(), Ok(10))
    }

    #[test]
    fn fraction_as_integer() {
        let number = Number::Float(10.5);
        assert_eq!(number.as_integer(), Err(LunifyError::FloatPrecisionLoss))
    }

    #[test]
    fn i64_max_as_float() {
        let number = Number::Integer(i64::MAX);
        assert_eq!(number.as_float(), Err(LunifyError::IntegerOverflow))
    }
}
