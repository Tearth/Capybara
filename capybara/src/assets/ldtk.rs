use crate::utils::color::RgbIntoVec4;
use anyhow::anyhow;
use anyhow::bail;
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
}

#[derive(Default)]
pub struct LdtkTile {
    pub id: usize,
    pub position: Vec2,
    pub source: Vec2,
    pub tilemap_id: usize,
}

pub fn load_world(name: &str, path: &str, data: &HashMap<String, JsonValue>) -> Result<LdtkWorld> {
    let mut world = LdtkWorld::default();
    let definitions = read_object(data, "defs")?;
    let tilemap_definitions = read_array(definitions, "tilesets")?;
    let layer_definitions = read_array(definitions, "layers")?;
    let levels = read_array(data, "levels")?;

    world.name = name.to_string();
    world.path = path.to_string();

    for tilemap in tilemap_definitions {
        world.tilemaps.push(load_tilemap(tilemap)?);
    }

    for level in levels {
        world.levels.push(load_level(level, &world.tilemaps, &layer_definitions)?);
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
) -> Result<LdtkLevel> {
    let mut level = LdtkLevel {
        id: read_value::<f64>(data, "uid")? as usize,
        name: read_value::<String>(data, "identifier")?,
        size: Vec2::new(read_value::<f64>(data, "pxWid")? as f32, read_value::<f64>(data, "pxHei")? as f32),
        background: read_color(data, "bgColor")?,
        tiles: Vec::new(),
    };

    let layers = read_array(data, "layerInstances")?;

    for data in layers {
        let id = read_value::<f64>(data, "layerDefUid")? as usize;
        let definition = match layer_definitions.iter().find(|p| read_value::<f64>(p, "uid").unwrap_or(0.0) as usize == id) {
            Some(definition) => definition,
            None => bail!("Failed to find definition for layer {}", id),
        };
        let definition_tilemap = read_value_nullable::<f64>(definition, "tilesetDefUid")?;
        let layer_tilemap = read_value_nullable::<f64>(data, "overrideTilesetUid")?;

        if let Some(tilemap_id) = definition_tilemap.or(layer_tilemap) {
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
    }

    Ok(level)
}

fn read_object<'a>(data: &'a HashMap<String, JsonValue>, name: &str) -> Result<&'a HashMap<String, JsonValue>> {
    match data.get(name) {
        Some(JsonValue::Object(object)) => Ok(object),
        _ => bail!("Failed to read object {}", name),
    }
}

fn read_array<'a>(data: &'a HashMap<String, JsonValue>, name: &str) -> Result<Vec<&'a HashMap<String, JsonValue>>> {
    match data.get(name) {
        Some(JsonValue::Array(array)) => Ok(array.iter().map(|p| p.get().unwrap()).collect()),
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
