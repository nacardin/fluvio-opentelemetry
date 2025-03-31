use std::time::{Duration, SystemTime, UNIX_EPOCH};

pub(crate) fn to_nanos(time: SystemTime) -> u64 {
    time.duration_since(UNIX_EPOCH)
        .unwrap_or_else(|_| Duration::from_secs(0))
        .as_nanos() as u64
}

use opentelemetry::{Array, Value};
use opentelemetry_types::common::v1::{
    any_value, AnyValue, ArrayValue, InstrumentationScope, KeyValue,
};
use std::borrow::Cow;

#[derive(Debug, Default)]
pub struct ResourceAttributesWithSchema {
    pub attributes: Attributes,
    pub schema_url: Option<String>,
}

impl From<&opentelemetry_sdk::Resource> for ResourceAttributesWithSchema {
    fn from(resource: &opentelemetry_sdk::Resource) -> Self {
        ResourceAttributesWithSchema {
            attributes: resource_attributes(resource),
            schema_url: resource.schema_url().map(ToString::to_string),
        }
    }
}

use opentelemetry_sdk::Resource;

pub fn convert_instrumentation_scope(
    data: (
        &opentelemetry::InstrumentationScope,
        Option<Cow<'static, str>>,
    ),
) -> InstrumentationScope {
    let (library, target) = data;
    if let Some(t) = target {
        InstrumentationScope {
            name: t.to_string(),
            version: String::new(),
            attributes: vec![],
            ..Default::default()
        }
    } else {
        InstrumentationScope {
            name: library.name().to_owned(),
            version: library.version().map(ToOwned::to_owned).unwrap_or_default(),
            attributes: attributes_from_kv(library.attributes().cloned()).0,
            ..Default::default()
        }
    }
}

/// Wrapper type for Vec<`KeyValue`>
#[derive(Default, Debug)]
pub struct Attributes(pub ::std::vec::Vec<opentelemetry_types::common::v1::KeyValue>);

pub fn attributes_from_kv<I: IntoIterator<Item = opentelemetry::KeyValue>>(kvs: I) -> Attributes {
    Attributes(
        kvs.into_iter()
            .map(|api_kv| KeyValue {
                key: api_kv.key.as_str().to_string(),
                value: Some(convert_value(api_kv.value)),
            })
            .collect(),
    )
}

impl<K: Into<String>, V: Into<AnyValue>> FromIterator<(K, V)> for Attributes {
    fn from_iter<T: IntoIterator<Item = (K, V)>>(iter: T) -> Self {
        Attributes(
            iter.into_iter()
                .map(|(k, v)| KeyValue {
                    key: k.into(),
                    value: Some(v.into()),
                })
                .collect(),
        )
    }
}

pub fn convert_value(value: Value) -> AnyValue {
    AnyValue {
        value: match value {
            Value::Bool(val) => Some(any_value::Value::BoolValue(val)),
            Value::I64(val) => Some(any_value::Value::IntValue(val)),
            Value::F64(val) => Some(any_value::Value::DoubleValue(val)),
            Value::String(val) => Some(any_value::Value::StringValue(val.to_string())),
            Value::Array(array) => Some(any_value::Value::ArrayValue(match array {
                Array::Bool(vals) => array_into_proto(vals),
                Array::I64(vals) => array_into_proto(vals),
                Array::F64(vals) => array_into_proto(vals),
                Array::String(vals) => array_into_proto(vals),
                _ => unreachable!("Nonexistent array type"), // Needs to be updated when new array types are added
            })),
            _ => unreachable!("Nonexistent value type"), // Needs to be updated when new value types are added
        },
    }
}

pub fn array_into_proto<T>(vals: Vec<T>) -> ArrayValue
where
    Value: From<T>,
{
    let values = vals
        .into_iter()
        .map(|val| convert_value(Value::from(val)))
        .collect();

    ArrayValue { values }
}

pub(crate) fn resource_attributes(resource: &Resource) -> Attributes {
    let kv_iter = resource
        .iter()
        .map(|(k, v)| opentelemetry::KeyValue::new(k.clone(), v.clone()));

    attributes_from_kv(kv_iter)
}
