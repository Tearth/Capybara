use super::color::RgbUtils;
use anyhow::anyhow;
use anyhow::bail;
use anyhow::Result;
use colors_transform::Rgb;
use glam::Vec2;
use glam::Vec4;
use std::collections::HashMap;
use tinyjson::InnerAsRef;
use tinyjson::JsonValue;

pub fn read_object<'a>(data: &'a HashMap<String, JsonValue>, name: &str) -> Result<&'a HashMap<String, JsonValue>> {
    match data.get(name) {
        Some(JsonValue::Object(object)) => Ok(object),
        _ => bail!("Failed to read object {}", name),
    }
}

pub fn read_object_nullable<'a>(data: &'a HashMap<String, JsonValue>, name: &str) -> Result<Option<&'a HashMap<String, JsonValue>>> {
    match data.get(name) {
        Some(JsonValue::Object(object)) => Ok(Some(object)),
        Some(JsonValue::Null) => Ok(None),
        _ => bail!("Failed to read object {}", name),
    }
}

pub fn read_array<'a>(data: &'a HashMap<String, JsonValue>, name: &str) -> Result<Vec<&'a HashMap<String, JsonValue>>> {
    match data.get(name) {
        Some(JsonValue::Array(array)) => Ok(array.iter().map(|p| p.get().unwrap()).collect()),
        _ => bail!("Failed to read array {}", name),
    }
}

pub fn read_array_raw<'a>(data: &'a HashMap<String, JsonValue>, name: &str) -> Result<&'a Vec<JsonValue>> {
    match data.get(name) {
        Some(JsonValue::Array(array)) => Ok(array),
        _ => bail!("Failed to read array {}", name),
    }
}

pub fn read_array_values(data: &HashMap<String, JsonValue>, name: &str) -> Result<Vec<String>> {
    match data.get(name) {
        Some(JsonValue::Array(array)) => Ok(array.iter().map(|p| p.stringify().unwrap()).collect()),
        _ => bail!("Failed to read array {}", name),
    }
}

pub fn read_value<T>(data: &HashMap<String, JsonValue>, name: &str) -> Result<T>
where
    T: Clone + Default + InnerAsRef,
{
    let value = data.get(name).ok_or_else(|| anyhow!("Failed to read {}", name))?;
    if value.is_null() {
        return Ok(Default::default());
    }

    Ok(value.get::<T>().ok_or_else(|| anyhow!("Failed to parse {}", name))?.clone())
}

pub fn read_value_nullable<T>(data: &HashMap<String, JsonValue>, name: &str) -> Result<Option<T>>
where
    T: Clone + Default + InnerAsRef,
{
    let value = data.get(name).ok_or_else(|| anyhow!("Failed to read {}", name))?;
    if value.is_null() {
        return Ok(None);
    }

    Ok(Some(value.get::<T>().ok_or_else(|| anyhow!("Failed to parse {}", name))?.clone()))
}

pub fn read_color(data: &HashMap<String, JsonValue>, name: &str) -> Result<Vec4> {
    let value = data.get(name).ok_or_else(|| anyhow!("Failed to read {}", name))?;
    if value.is_null() {
        return Ok(Vec4::new(0.0, 0.0, 0.0, 1.0));
    }
    let parsed = value.get::<String>().ok_or_else(|| anyhow!("Failed to parse {}", name))?.clone();

    Ok(Rgb::from_hex_str(&parsed).map_err(|_| anyhow!("Failed to parse {} into RGB", name))?.to_vec4())
}

pub fn read_position(data: &HashMap<String, JsonValue>, name: &str) -> Result<Vec2> {
    let position = match data.get(name) {
        Some(JsonValue::Array(array)) => array,
        _ => bail!("Failed to read position"),
    };

    let x = match position.get(0) {
        Some(JsonValue::Number(value)) => *value as f32,
        _ => bail!("Failed to parse position"),
    };
    let y = match position.get(1) {
        Some(JsonValue::Number(value)) => *value as f32,
        _ => bail!("Failed to parse position"),
    };

    Ok(Vec2::new(x, y))
}
