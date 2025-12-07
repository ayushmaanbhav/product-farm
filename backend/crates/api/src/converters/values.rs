//! Value type conversions between proto and core types

use product_farm_core::Value as CoreValue;

use crate::grpc::proto;

/// Convert proto Value to core Value (fallible version)
/// Returns an error if decimal parsing fails or other validation fails
pub fn try_proto_to_core_value(v: &proto::Value) -> Result<CoreValue, tonic::Status> {
    match &v.value {
        Some(proto::value::Value::NullValue(_)) => Ok(CoreValue::Null),
        Some(proto::value::Value::BoolValue(b)) => Ok(CoreValue::Bool(*b)),
        Some(proto::value::Value::IntValue(i)) => Ok(CoreValue::Int(*i)),
        Some(proto::value::Value::FloatValue(f)) => Ok(CoreValue::Float(*f)),
        Some(proto::value::Value::StringValue(s)) => Ok(CoreValue::String(s.clone())),
        Some(proto::value::Value::DecimalValue(s)) => s
            .parse()
            .map(CoreValue::Decimal)
            .map_err(|_| tonic::Status::invalid_argument(format!("Invalid decimal value: '{}'", s))),
        Some(proto::value::Value::ArrayValue(arr)) => {
            let values: Result<Vec<_>, _> = arr.values.iter().map(try_proto_to_core_value).collect();
            Ok(CoreValue::Array(values?))
        }
        Some(proto::value::Value::ObjectValue(obj)) => {
            let fields: Result<Vec<_>, _> = obj
                .fields
                .iter()
                .map(|(k, v)| try_proto_to_core_value(v).map(|cv| (k.clone(), cv)))
                .collect();
            Ok(CoreValue::Object(fields?.into_iter().collect()))
        }
        None => Ok(CoreValue::Null),
    }
}

/// Convert proto Value to core Value
/// This is a convenience wrapper for contexts where errors cannot be returned.
/// For gRPC handlers, prefer try_proto_to_core_value for proper error handling.
pub fn proto_to_core_value(v: &proto::Value) -> CoreValue {
    // Use unwrap_or_default for backwards compatibility, but log warnings
    try_proto_to_core_value(v).unwrap_or_else(|e| {
        tracing::warn!("Failed to convert proto value: {}", e);
        CoreValue::Null
    })
}

/// Convert core Value to proto Value
pub fn core_to_proto_value(v: &CoreValue) -> proto::Value {
    let value = match v {
        CoreValue::Null => proto::value::Value::NullValue(true),
        CoreValue::Bool(b) => proto::value::Value::BoolValue(*b),
        CoreValue::Int(i) => proto::value::Value::IntValue(*i),
        CoreValue::Float(f) => proto::value::Value::FloatValue(*f),
        CoreValue::String(s) => proto::value::Value::StringValue(s.clone()),
        CoreValue::Decimal(d) => proto::value::Value::DecimalValue(d.to_string()),
        CoreValue::Array(arr) => proto::value::Value::ArrayValue(proto::ArrayValue {
            values: arr.iter().map(core_to_proto_value).collect(),
        }),
        CoreValue::Object(obj) => proto::value::Value::ObjectValue(proto::ObjectValue {
            fields: obj
                .iter()
                .map(|(k, v)| (k.clone(), core_to_proto_value(v)))
                .collect(),
        }),
    };
    proto::Value { value: Some(value) }
}
