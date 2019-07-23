use std::convert::TryFrom;

use crate::error::RadError;
use crate::types::{
    array::RadonArray, bytes::RadonBytes, float::RadonFloat, map::RadonMap, RadonType,
};

pub fn to_float(input: RadonBytes) -> Result<RadonFloat, RadError> {
    RadonFloat::try_from(input.value())
}

pub fn to_map(input: RadonBytes) -> Result<RadonMap, RadError> {
    RadonMap::try_from(input.value())
}
pub fn to_array(input: RadonBytes) -> Result<RadonArray, RadError> {
    RadonArray::try_from(input.value())
}

#[test]
fn test_as_float() {
    use serde_cbor::value::Value;

    let radon_float = RadonFloat::from(std::f64::consts::PI);
    let radon_bytes_error =
        RadonBytes::from(Value::try_from(String::from("Hello world!")).unwrap());
    let radon_bytes = RadonBytes::from(Value::try_from(std::f64::consts::PI).unwrap());

    assert_eq!(to_float(radon_bytes).unwrap(), radon_float);
    assert_eq!(
        &to_float(radon_bytes_error).unwrap_err().to_string(),
        "Failed to convert string to float with error message: invalid float literal"
    );
}