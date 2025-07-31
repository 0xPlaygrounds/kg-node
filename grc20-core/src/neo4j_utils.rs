use neo4rs::BoltType;
use serde::Deserialize;

pub fn serde_value_to_bolt(value: serde_json::Value) -> BoltType {
    match value {
        serde_json::Value::Null => BoltType::Null(neo4rs::BoltNull),
        serde_json::Value::Bool(value) => BoltType::Boolean(neo4rs::BoltBoolean { value }),
        serde_json::Value::Number(number) if number.is_i64() => {
            BoltType::Integer(neo4rs::BoltInteger {
                value: number.as_i64().expect("number should be an i64"),
            })
        }
        serde_json::Value::Number(number) if number.is_u64() => {
            BoltType::Integer(neo4rs::BoltInteger {
                value: number.as_u64().expect("number should be a u64") as i64,
            })
        }
        serde_json::Value::Number(number) => BoltType::Float(neo4rs::BoltFloat {
            value: number.as_f64().expect("number should be an f64"),
        }),
        serde_json::Value::String(value) => BoltType::String(neo4rs::BoltString { value }),
        serde_json::Value::Array(vec) => {
            let values = vec.into_iter().map(serde_value_to_bolt).collect();
            BoltType::List(neo4rs::BoltList { value: values })
        }
        serde_json::Value::Object(map) => {
            let properties = map
                .into_iter()
                .filter(|(key, _)| key != "$type")
                .map(|(key, value)| {
                    (
                        neo4rs::BoltString { value: key },
                        serde_value_to_bolt(value),
                    )
                })
                .collect();
            BoltType::Map(neo4rs::BoltMap { value: properties })
        }
    }
}

pub fn deserialize_uuid<'de, D>(deserializer: D) -> Result<uuid::Uuid, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;
    uuid::Uuid::parse_str(&s).map_err(serde::de::Error::custom)
}
