use crate::utils::color::RgbUtils;
use crate::utils::json::*;
use anyhow::anyhow;
use anyhow::bail;
use anyhow::Error;
use anyhow::Result;
use colors_transform::Rgb;
use glam::Vec2;
use glam::Vec4;
use rustc_hash::FxHashMap;
use std::collections::HashMap;
use std::path::Path;
use tinyjson::JsonValue;

#[derive(Debug, Default)]
pub struct LdtkWorld {
    pub name: String,
    pub path: String,
    pub tilemaps: Vec<LdtkTilemap>,
    pub levels: Vec<LdtkLevel>,
}

#[derive(Debug, Default)]
pub struct LdtkTilemap {
    pub id: usize,
    pub name: String,
    pub path: String,
    pub tile_size: Vec2,
    pub custom: FxHashMap<usize, String>,
}

#[derive(Debug, Default)]
pub struct LdtkLevel {
    pub id: usize,
    pub name: String,
    pub size: Vec2,
    pub background: Vec4,
    pub layers: Vec<LdtkLayer>,
}

#[derive(Debug, Default)]
pub struct LdtkLayer {
    pub id: usize,
    pub name: String,
    pub grid_size: Vec2,
    pub tilemap_id: Option<usize>,
    pub tiles: Vec<LdtkTile>,
    pub entities: Vec<LdtkEntity>,
}

#[derive(Debug, Default)]
pub struct LdtkTile {
    pub id: usize,
    pub position: Vec2,
    pub source: Vec2,
}

#[derive(Debug, Default)]
pub struct LdtkEntity {
    pub name: String,
    pub position: Vec2,
    pub size: Vec2,
    pub pivot: Vec2,
    pub source: Vec2,
    pub tilemap_id: usize,
    pub fields: FxHashMap<String, LdtkEntityField>,
}

#[derive(Clone, Debug, PartialEq)]
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
    tilemaps: &[LdtkTilemap],
    layer_definitions: &[&HashMap<String, JsonValue>],
    entity_definitions: &[&HashMap<String, JsonValue>],
) -> Result<LdtkLevel> {
    let mut level = LdtkLevel {
        id: read_value::<f64>(data, "uid")? as usize,
        name: read_value::<String>(data, "identifier")?,
        size: Vec2::new(read_value::<f64>(data, "pxWid")? as f32, read_value::<f64>(data, "pxHei")? as f32),
        background: read_color(data, "bgColor")?,
        layers: Vec::new(),
    };

    let layers = read_array(data, "layerInstances")?;

    for data in layers {
        let id = read_value::<f64>(data, "layerDefUid")? as usize;
        let layer_definition = match layer_definitions.iter().find(|p| read_value::<f64>(p, "uid").unwrap_or(0.0) as usize == id) {
            Some(definition) => definition,
            None => bail!("Failed to find definition for layer {}", id),
        };
        let layer_definition_tilemap = read_value_nullable::<f64>(layer_definition, "tilesetDefUid")?;
        let layer_name = read_value::<String>(layer_definition, "identifier")?;
        let layer_grid_size = read_value::<f64>(layer_definition, "gridSize")? as f32;
        let layer_tilemap = read_value_nullable::<f64>(data, "overrideTilesetUid")?;
        let tilemap_id = layer_tilemap.or(layer_definition_tilemap);

        let mut layer = LdtkLayer {
            id,
            name: layer_name,
            grid_size: Vec2::new(layer_grid_size, layer_grid_size),
            tilemap_id: tilemap_id.map(|p| p as usize),
            tiles: Vec::new(),
            entities: Vec::new(),
        };

        if let Some(tilemap_id) = tilemap_id {
            let tilemap = match tilemaps.iter().find(|p| p.id == tilemap_id as usize) {
                Some(tilemap) => tilemap,
                None => bail!("Failed to find tilemap {}", tilemap_id),
            };
            let tiles = read_array(data, "gridTiles")?;

            for data in tiles {
                let position = read_position(data, "px")?;
                layer.tiles.push(LdtkTile {
                    id: read_value::<f64>(data, "t")? as usize,
                    position: Vec2::new(position.x, level.size.y - position.y - tilemap.tile_size.y),
                    source: read_position(data, "src")?,
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
            let pivot = Vec2::new(read_value::<f64>(entity_definition, "pivotX")? as f32, read_value::<f64>(entity_definition, "pivotY")? as f32);

            let entity_definition_tilemap = read_value_nullable::<f64>(entity_definition, "tilesetId")?;
            if let Some(tilemap_id) = layer_tilemap.or(entity_definition_tilemap) {
                let position = read_position(data, "px")?;
                let tilemap = match tilemaps.iter().find(|p| p.id == tilemap_id as usize) {
                    Some(tilemap) => tilemap,
                    None => bail!("Failed to find tilemap {}", tilemap_id),
                };

                let mut fields = FxHashMap::default();
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
                                let color = Rgb::from_hex_str(&hex).map_err(|_| anyhow!("Failed to parse color"))?.to_vec4();
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
                                        let color = Rgb::from_hex_str(&hex).map_err(|_| anyhow!("Failed to parse color"))?.to_vec4();

                                        Ok(color)
                                    })
                                    .collect::<Result<Vec<Vec4>, Error>>()?,
                            ),
                            _ => bail!("Invalid field type"),
                        }
                    };

                    fields.insert(field_name, value);
                }

                layer.entities.push(LdtkEntity {
                    name: read_value::<String>(entity_definition, "identifier")?,
                    position: Vec2::new(position.x, level.size.y - position.y - tilemap.tile_size.y + pivot.y * tilemap.tile_size.y),
                    size: Vec2::new(read_value::<f64>(data, "width")? as f32, read_value::<f64>(data, "height")? as f32),
                    pivot,
                    source: Vec2::new(read_value::<f64>(tile_data, "x")? as f32, read_value::<f64>(tile_data, "y")? as f32),
                    tilemap_id: tilemap_id as usize,
                    fields,
                });
            }
        }

        level.layers.push(layer);
    }

    Ok(level)
}
