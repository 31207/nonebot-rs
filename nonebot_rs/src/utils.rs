/// 去除字符串前方多余空格
#[allow(dead_code)]
pub fn remove_space(s: &str) -> String {
    let mut rstring = String::from(s);
    let mut chars = s.chars();
    while chars.next() == Some(' ') {
        rstring.remove(0);
    }
    rstring
}

use chrono::Local;

#[allow(dead_code)]
pub fn timestamp() -> i64 {
    let time = Local::now();
    time.timestamp()
}

use serde::Deserializer;

struct JsonIdVisitor;

impl<'de> serde::de::Visitor<'de> for JsonIdVisitor {
    type Value = String;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("a i64 or str containing json data")
    }

    fn visit_i64<E>(self, v: i64) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(v.to_string())
    }

    fn visit_u64<E>(self, v: u64) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(v.to_string())
    }

    fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(v.to_string())
    }
}

pub fn id_deserializer<'de, D>(d: D) -> Result<String, D::Error>
where
    D: Deserializer<'de>,
{
    d.deserialize_any(JsonIdVisitor)
}

struct OptionJsonIdVisitor;

impl<'de> serde::de::Visitor<'de> for OptionJsonIdVisitor {
    type Value = Option<String>;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("a i64 or str containing json data")
    }

    fn visit_some<D>(self, d: D) -> Result<Option<String>, D::Error>
    where
        D: serde::de::Deserializer<'de>,
    {
        Ok(Some(d.deserialize_any(JsonIdVisitor)?))
    }
}

pub fn option_id_deserializer<'de, D>(d: D) -> Result<Option<String>, D::Error>
where
    D: Deserializer<'de>,
{
    d.deserialize_option(OptionJsonIdVisitor)
}

struct ScalarStringVisitor;

impl<'de> serde::de::Visitor<'de> for ScalarStringVisitor {
    type Value = String;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("a scalar containing json data")
    }

    fn visit_i64<E>(self, v: i64) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(v.to_string())
    }

    fn visit_u64<E>(self, v: u64) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(v.to_string())
    }

    fn visit_f64<E>(self, v: f64) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(v.to_string())
    }

    fn visit_bool<E>(self, v: bool) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(v.to_string())
    }

    fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(v.to_string())
    }
}

/// 将任意标量（数字/字符串/布尔）反序列化为 `String`，兼容不同实现
pub fn scalar_string_deserializer<'de, D>(d: D) -> Result<String, D::Error>
where
    D: Deserializer<'de>,
{
    d.deserialize_any(ScalarStringVisitor)
}

struct OptionScalarStringVisitor;

impl<'de> serde::de::Visitor<'de> for OptionScalarStringVisitor {
    type Value = Option<String>;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("an optional scalar containing json data")
    }

    fn visit_some<D>(self, d: D) -> Result<Option<String>, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        Ok(Some(d.deserialize_any(ScalarStringVisitor)?))
    }

    fn visit_none<E>(self) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(None)
    }

    fn visit_unit<E>(self) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(None)
    }
}

/// 将任意可选标量（数字/字符串/布尔/null）反序列化为 `Option<String>`
pub fn option_scalar_string_deserializer<'de, D>(d: D) -> Result<Option<String>, D::Error>
where
    D: Deserializer<'de>,
{
    d.deserialize_option(OptionScalarStringVisitor)
}

struct LenientI64Visitor;

impl<'de> serde::de::Visitor<'de> for LenientI64Visitor {
    type Value = i64;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("an i64 or a string containing an i64")
    }

    fn visit_i64<E>(self, v: i64) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(v)
    }

    fn visit_u64<E>(self, v: u64) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        i64::try_from(v).map_err(E::custom)
    }

    fn visit_f64<E>(self, v: f64) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(v as i64)
    }

    fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        v.parse::<i64>().map_err(E::custom)
    }
}

/// 将 `i64`/`u64`/`f64`/字符串 统一反序列化为 `i64`
pub fn lenient_i64_deserializer<'de, D>(d: D) -> Result<i64, D::Error>
where
    D: Deserializer<'de>,
{
    d.deserialize_any(LenientI64Visitor)
}

struct LenientI32Visitor;

impl<'de> serde::de::Visitor<'de> for LenientI32Visitor {
    type Value = i32;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("an i32 or a string containing an i32")
    }

    fn visit_i64<E>(self, v: i64) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        i32::try_from(v).map_err(E::custom)
    }

    fn visit_u64<E>(self, v: u64) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        i32::try_from(v).map_err(E::custom)
    }

    fn visit_f64<E>(self, v: f64) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(v as i32)
    }

    fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        v.parse::<i32>().map_err(E::custom)
    }
}

/// 将 `i32`/`i64`/`u64`/`f64`/字符串 统一反序列化为 `i32`
pub fn lenient_i32_deserializer<'de, D>(d: D) -> Result<i32, D::Error>
where
    D: Deserializer<'de>,
{
    d.deserialize_any(LenientI32Visitor)
}

/// 将任意 JSON 值反序列化为 `String`：字符串原样返回，其他标量/对象转为 JSON 文本
pub fn flexible_string_deserializer<'de, D>(d: D) -> Result<String, D::Error>
where
    D: Deserializer<'de>,
{
    let value = <serde_json::Value as serde::Deserialize>::deserialize(d)?;
    Ok(match value {
        serde_json::Value::String(s) => s,
        serde_json::Value::Null => String::new(),
        other => other.to_string(),
    })
}
