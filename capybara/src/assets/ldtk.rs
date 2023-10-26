use crate::utils::color::RgbIntoVec4;
use anyhow::anyhow;
use anyhow::bail;
use anyhow::Error;
use anyhow::Result;
use colors_transform::Rgb;
use glam::Vec2;
use glam::Vec4;
use std::collections::HashMap;
use std::path::Path;
use tinyjson::InnerAsRef;
use tinyjson::JsonValue;

#[derive(Default)]
pub struct LdtkWorld {
    pub name: String,
    pub path: String,
    pub tilemaps: Vec<LdtkTilemap>,
    pub levels: Vec<LdtkLevel>,
}

#[derive(Default)]
pub struct LdtkTilemap {
    pub id: usize,
    pub name: String,
    pub path: String,
    pub tile_size: Vec2,
    pub custom: HashMap<usize, String>,
}

#[derive(Default)]
pub struct LdtkLevel {
    pub id: usize,
    pub name: String,
    pub size: Vec2,
    pub background: Vec4,
    pub tiles: Vec<LdtkTile>,
    pub entities: Vec<LdtkEntity>,
}

#[derive(Default)]
pub struct LdtkTile {
    pub id: usize,
    pub position: Vec2,
    pub source: Vec2,
    pub tilemap_id: usize,
}

#[derive(Default)]
pub struct LdtkEntity {
    pub name: String,
    pub position: Vec2,
    pub size: Vec2,
    pub source: Vec2,
    pub tilemap_id: usize,
    pub fields: HashMap<String, LdtkEntityField>,
}

pub enum LdtkEntityField {
    Bool(bool),
    Int(i32),
    Float(f32),
    String(String),
    Color(Vec4),
    BoolArray(Vec<bool>),
    IntArray(Vec<i32>),
    FloatArray(Vec<f32>),
    StringArray(Vec<String>),
    ColorArray(Vec<Vec4>),
}

pub fn load_world(name: &str, path: &str, data: &HashMap<String, JsonValue>) -> Result<LdtkWorld> {
    let mut world = LdtkWorld::default();
    let definitions = read_object(data, "defs")?;
    let tilemap_definitions = read_array(definitions, "tilesets")?;
    let layer_definitions = read_array(definitions, "layers")?;
    let entity_definitions = read_array(definitions, "entities")?;
    let levels = read_array(data, "levels")?;

    world.name = name.to_string();
    world.path = path.to_string();

    for tilemap in tilemap_definitions {
        world.tilemaps.push(load_tilemap(tilemap)?);
    }

    for level in levels {
        world.levels.push(load_level(level, &world.tilemaps, &layer_definitions, &entity_definitions)?);
    }

    Ok(world)
}

fn load_tilemap(data: &HashMap<String, JsonValue>) -> Result<LdtkTilemap> {
    let path = read_value::<String>(data, "relPath")?;
    let tile_size = read_value::<f64>(data, "tileGridSize")? as f32;
    let custom_data = read_array(data, "customData")?;

    let name = match Path::new(&path).file_stem().and_then(|p| p.to_str()) {
        Some(name) => name.to_string(),
        None => bail!("Failed to get name from path {:?}", path),
    };

    let mut tilemap = LdtkTilemap {
        id: read_value::<f64>(data, "uid")? as usize,
        name,
        path: read_value::<String>(data, "relPath")?,
        tile_size: Vec2::new(tile_size, tile_size),
        custom: Default::default(),
    };

    for data in custom_data {
        let tile_id = read_value::<f64>(data, "tileId")? as usize;
        let data = read_value::<String>(data, "data")?;

        tilemap.custom.insert(tile_id, data);
    }

    Ok(tilemap)
}

fn load_level(
    data: &HashMap<String, JsonValue>,
    tilemaps: &Vec<LdtkTilemap>,
    layer_definitions: &Vec<&HashMap<String, JsonValue>>,
    entity_definitions: &Vec<&HashMap<String, JsonValue>>,
) -> Result<LdtkLevel> {
    let mut level = LdtkLevel {
        id: read_value::<f64>(data, "uid")? as usize,
        name: read_value::<String>(data, "identifier")?,
        size: Vec2::new(read_value::<f64>(data, "pxWid")? as f32, read_value::<f64>(data, "pxHei")? as f32),
        background: read_color(data, "bgColor")?,
        tiles: Vec::new(),
        entities: Vec::new(),
    };

    let layers = read_array(data, "layerInstances")?;

    for data in layers {
        let id = read_value::<f64>(data, "layerDefUid")? as usize;
        let layer_definition = match layer_definitions.iter().find(|p| read_value::<f64>(p, "uid").unwrap_or(0.0) as usize == id) {
            Some(definition) => definition,
            None => bail!("Failed to find definition for layer {}", id),
        };
        let layer_definition_tilemap = read_value_nullable::<f64>(layer_definition, "tilesetDefUid")?;
        let layer_tilemap = read_value_nullable::<f64>(data, "overrideTilesetUid")?;

        if let Some(tilemap_id) = layer_tilemap.or(layer_definition_tilemap) {
            let tilemap = match tilemaps.iter().find(|p| p.id == tilemap_id as usize) {
                Some(tilemap) => tilemap,
                None => bail!("Failed to find tilemap {}", tilemap_id),
            };
            let tiles = read_array(data, "gridTiles")?;

            for data in tiles {
                let position = read_position(data, "px")?;
                level.tiles.push(LdtkTile {
                    id: read_value::<f64>(data, "t")? as usize,
                    position: Vec2::new(position.x, level.size.y - position.y - tilemap.tile_size.y),
                    source: read_position(data, "src")?,
                    tilemap_id: tilemap_id as usize,
                });
            }
        }

        let entities = read_array(data, "entityInstances")?;
        for data in entities {
            let entity_definition_id = read_value::<f64>(data, "defUid")? as usize;
            let entity_definition =
                match entity_definitions.iter().find(|p| read_value::<f64>(p, "uid").unwrap_or(0.0) as usize == entity_definition_id) {
                    Some(definition) => definition,
                    None => bail!("Failed to find definition for entity {}", entity_definition_id),
                };
            let field_definitions = read_array(entity_definition, "fieldDefs")?;
            let tile_data = read_object(entity_definition, "tileRect")?;

            let entity_definition_tilemap = read_value_nullable::<f64>(entity_definition, "tilesetId")?;
            if let Some(tilemap_id) = layer_tilemap.or(entity_definition_tilemap) {
                let position = read_position(data, "px")?;
                let tilemap = match tilemaps.iter().find(|p| p.id == tilemap_id as usize) {
                    Some(tilemap) => tilemap,
                    None => bail!("Failed to find tilemap {}", tilemap_id),
                };

                let mut fields = HashMap::new();
                let field_instances = read_array(data, "fieldInstances")?;

                for data in field_instances {
                    let field_definition_id = read_value::<f64>(data, "defUid")? as usize;
                    let field_definition =
                        match field_definitions.iter().find(|p| read_value::<f64>(p, "uid").unwrap_or(0.0) as usize == field_definition_id) {
                            Some(definition) => definition,
                            None => bail!("Failed to find definition for field {}", field_definition_id),
                        };

                    let field_name = read_value::<String>(field_definition, "identifier")?;
                    let field_type = read_value::<String>(field_definition, "type")?;
                    let field_is_array = read_value::<bool>(field_definition, "isArray")?;
                    let default_value_object = read_object_nullable(field_definition, "defaultOverride")?;
                    let default_value = if let Some(default_value_object) = default_value_object {
                        read_array_values(default_value_object, "params")?.get(0).cloned()
                    } else {
                        None
                    };

                    let mut values = Vec::new();
                    let field_value_array = read_array_raw(data, "realEditorValues")?;
                    for field_value in field_value_array {
                        match field_value {
                            JsonValue::Object(data) => {
                                let value = read_array_values(data, "params")?;
                                values.push(value[0].clone());
                            }
                            JsonValue::Null => match &default_value {
                                Some(default_value) => values.push(default_value.clone()),
                                None => bail!("No default value"),
                            },
                            _ => {}
                        }
                    }

                    if values.is_empty() {
                        if let Some(default_value) = default_value {
                            values.push(default_value);
                        }
                    }

                    let value = if !field_is_array {
                        match field_type.as_str() {
                            "F_Bool" => LdtkEntityField::Bool(values.get(0).unwrap_or(&"false".to_string()).parse()?),
                            "F_Int" => LdtkEntityField::Int(values.get(0).unwrap_or(&"0".to_string()).parse()?),
                            "F_Float" => LdtkEntityField::Float(values.get(0).unwrap_or(&"0.0".to_string()).parse()?),
                            "F_String" => LdtkEntityField::String(values.get(0).unwrap_or(&"".to_string()).to_string()),
                            "F_Text" => LdtkEntityField::String(values.get(0).unwrap_or(&"".to_string()).to_string()),
                            "F_Color" => {
                                let hex = format!("{:x}", values.get(0).unwrap_or(&"0".to_string()).to_string().parse::<u32>()?);
                                let color = Rgb::from_hex_str(&hex).map_err(|_| anyhow!("Failed to parse color"))?.into_vec4();
                                LdtkEntityField::Color(color)
                            }
                            _ => bail!("Invalid field type"),
                        }
                    } else {
                        match field_type.as_str() {
                            "F_Bool" => LdtkEntityField::BoolArray(values.iter().map(|p| p.parse()).collect::<Result<Vec<bool>, _>>()?),
                            "F_Int" => LdtkEntityField::IntArray(values.iter().map(|p| p.parse()).collect::<Result<Vec<i32>, _>>()?),
                            "F_Float" => LdtkEntityField::FloatArray(values.iter().map(|p| p.parse()).collect::<Result<Vec<f32>, _>>()?),
                            "F_String" => LdtkEntityField::StringArray(values.iter().map(|p| p.to_string()).collect()),
                            "F_Text" => LdtkEntityField::StringArray(values.iter().map(|p| p.to_string()).collect()),
                            "F_Color" => LdtkEntityField::ColorArray(
                                values
                                    .iter()
                                    .map(|p| {
                                        let hex = format!("{:x}", p.parse::<u32>()?);
                                        let color = Rgb::from_hex_str(&hex).map_err(|_| anyhow!("Failed to parse color"))?.into_vec4();

                                        Ok(color)
                                    })
                                    .collect::<Result<Vec<Vec4>, Error>>()?,
                            ),
                            _ => bail!("Invalid field type"),
                        }
                    };

                    fields.insert(field_name, value);
                }

                level.entities.push(LdtkEntity {
                    name: read_value::<String>(entity_definition, "identifier")?,
                    position: Vec2::new(position.x, level.size.y - position.y - tilemap.tile_size.y),
                    size: Vec2::new(read_value::<f64>(data, "width")? as f32, read_value::<f64>(data, "height")? as f32),
                    source: Vec2::new(read_value::<f64>(tile_data, "x")? as f32, read_value::<f64>(tile_data, "y")? as f32),
                    tilemap_id: tilemap_id as usize,
                    fields,
                });
            }
        }
    }

    Ok(level)
}

fn read_object<'a>(data: &'a HashMap<String, JsonValue>, name: &str) -> Result<&'a HashMap<String, JsonValue>> {
    match data.get(name) {
        Some(JsonValue::Object(object)) => Ok(object),
        _ => bail!("Failed to read object {}", name),
    }
}

fn read_object_nullable<'a>(data: &'a HashMap<String, JsonValue>, name: &str) -> Result<Option<&'a HashMap<String, JsonValue>>> {
    match data.get(name) {
        Some(JsonValue::Object(object)) => Ok(Some(object)),
        Some(JsonValue::Null) => Ok(None),
        _ => bail!("Failed to read object {}", name),
    }
}

fn read_array<'a>(data: &'a HashMap<String, JsonValue>, name: &str) -> Result<Vec<&'a HashMap<String, JsonValue>>> {
    match data.get(name) {
        Some(JsonValue::Array(array)) => Ok(array.iter().map(|p| p.get().unwrap()).collect()),
        _ => bail!("Failed to read array {}", name),
    }
}

fn read_array_raw<'a>(data: &'a HashMap<String, JsonValue>, name: &str) -> Result<&'a Vec<JsonValue>> {
    match data.get(name) {
        Some(JsonValue::Array(array)) => Ok(array),
        _ => bail!("Failed to read array {}", name),
    }
}

fn read_array_values<'a>(data: &'a HashMap<String, JsonValue>, name: &str) -> Result<Vec<String>> {
    match data.get(name) {
        Some(JsonValue::Array(array)) => Ok(array.iter().map(|p| p.stringify().unwrap()).collect()),
        _ => bail!("Failed to read array {}", name),
    }
}

fn read_value<T>(data: &HashMap<String, JsonValue>, name: &str) -> Result<T>
where
    T: Clone + Default + InnerAsRef,
{
    let value = data.get(name).ok_or_else(|| anyhow!("Failed to read {}", name))?;
    if value.is_null() {
        return Ok(Default::default());
    }

    Ok(value.get::<T>().ok_or_else(|| anyhow!("Failed to parse {}", name))?.clone())
}

fn read_value_nullable<T>(data: &HashMap<String, JsonValue>, name: &str) -> Result<Option<T>>
where
    T: Clone + Default + InnerAsRef,
{
    let value = data.get(name).ok_or_else(|| anyhow!("Failed to read {}", name))?;
    if value.is_null() {
        return Ok(None);
    }

    Ok(Some(value.get::<T>().ok_or_else(|| anyhow!("Failed to parse {}", name))?.clone()))
}

fn read_color(data: &HashMap<String, JsonValue>, name: &str) -> Result<Vec4> {
    let value = data.get(name).ok_or_else(|| anyhow!("Failed to read {}", name))?;
    if value.is_null() {
        return Ok(Vec4::new(0.0, 0.0, 0.0, 1.0));
    }
    let parsed = value.get::<String>().ok_or_else(|| anyhow!("Failed to parse {}", name))?.clone();

    Ok(Rgb::from_hex_str(&parsed).map_err(|_| anyhow!("Failed to parse {} into RGB", name))?.into_vec4())
}

fn read_position(data: &HashMap<String, JsonValue>, name: &str) -> Result<Vec2> {
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