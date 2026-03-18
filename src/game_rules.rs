use std::collections::HashMap;

use serde::Deserialize;

use crate::{Achievement, ItemKind, Material, ITEM_COUNT};

const DEFAULT_YAML: &str = include_str!("../data/config.yaml");

// ---------------------------------------------------------------------------
// Runtime rule types (pre-resolved enums, no string lookups during gameplay)
// ---------------------------------------------------------------------------

#[derive(Clone, Debug)]
pub struct GameRules {
    pub item_initial: [i32; ITEM_COUNT],
    pub item_max: [i32; ITEM_COUNT],
    pub collect: HashMap<Material, CollectRule>,
    pub place: HashMap<String, PlaceRule>,
    pub make: HashMap<ItemKind, MakeRule>,
    pub walkable: Vec<Material>,
}

#[derive(Clone, Debug)]
pub struct CollectRule {
    pub require: Vec<(ItemKind, i32)>,
    pub receive: Vec<(ItemKind, i32)>,
    pub leaves: Material,
    pub probability: f32,
    pub achievement: Achievement,
}

#[derive(Clone, Debug)]
pub enum PlaceType {
    Material,
    Object,
}

#[derive(Clone, Debug)]
pub struct PlaceRule {
    pub uses: Vec<(ItemKind, i32)>,
    pub where_materials: Vec<Material>,
    pub place_type: PlaceType,
    pub placed_material: Option<Material>,
    pub achievement: Achievement,
}

#[derive(Clone, Debug)]
pub struct MakeRule {
    pub uses: Vec<(ItemKind, i32)>,
    pub nearby: Vec<Material>,
    pub gives: i32,
    pub achievement: Achievement,
}

// ---------------------------------------------------------------------------
// Intermediate serde types for YAML parsing
// ---------------------------------------------------------------------------

#[derive(Deserialize)]
struct YamlConfig {
    walkable: Vec<String>,
    items: HashMap<String, YamlItemDef>,
    collect: HashMap<String, YamlCollectDef>,
    place: HashMap<String, YamlPlaceDef>,
    make: HashMap<String, YamlMakeDef>,
}

#[derive(Deserialize)]
struct YamlItemDef {
    max: i32,
    initial: i32,
}

#[derive(Deserialize)]
struct YamlCollectDef {
    #[serde(default)]
    require: HashMap<String, i32>,
    receive: HashMap<String, i32>,
    leaves: String,
    #[serde(default = "default_probability")]
    probability: f32,
}

fn default_probability() -> f32 {
    1.0
}

#[derive(Deserialize)]
struct YamlPlaceDef {
    uses: HashMap<String, i32>,
    #[serde(rename = "where")]
    where_: Vec<String>,
    #[serde(rename = "type")]
    type_: String,
}

#[derive(Deserialize)]
struct YamlMakeDef {
    uses: HashMap<String, i32>,
    nearby: Vec<String>,
    gives: i32,
}

// ---------------------------------------------------------------------------
// Resolution: YAML strings → enum-indexed runtime types
// ---------------------------------------------------------------------------

impl GameRules {
    pub fn from_yaml(yaml: &str) -> Result<Self, String> {
        let cfg: YamlConfig =
            serde_yaml::from_str(yaml).map_err(|e| format!("YAML parse error: {e}"))?;
        Self::resolve(cfg)
    }

    fn resolve(cfg: YamlConfig) -> Result<Self, String> {
        // Items
        let mut item_initial = [0_i32; ITEM_COUNT];
        let mut item_max = [9_i32; ITEM_COUNT];
        for (name, def) in &cfg.items {
            let kind = ItemKind::from_name(name)
                .ok_or_else(|| format!("unknown item: {name}"))?;
            item_initial[kind as usize] = def.initial;
            item_max[kind as usize] = def.max;
        }

        // Walkable
        let walkable = cfg
            .walkable
            .iter()
            .map(|name| {
                Material::from_name(name).ok_or_else(|| format!("unknown material: {name}"))
            })
            .collect::<Result<Vec<_>, _>>()?;

        // Collect
        let mut collect = HashMap::new();
        for (name, def) in &cfg.collect {
            let material = Material::from_name(name)
                .ok_or_else(|| format!("unknown collect material: {name}"))?;
            let require = resolve_item_map(&def.require)?;
            let receive = resolve_item_map(&def.receive)?;
            let leaves = Material::from_name(&def.leaves)
                .ok_or_else(|| format!("unknown leaves material: {}", def.leaves))?;

            // Achievement: collect_{first_received_item_name}
            let first_receive_name = def
                .receive
                .keys()
                .next()
                .ok_or_else(|| format!("collect rule '{name}' has no receive items"))?;
            let achievement_name = format!("collect_{first_receive_name}");
            let achievement = Achievement::from_name(&achievement_name)
                .ok_or_else(|| format!("unknown achievement: {achievement_name}"))?;

            collect.insert(
                material,
                CollectRule {
                    require,
                    receive,
                    leaves,
                    probability: def.probability,
                    achievement,
                },
            );
        }

        // Place
        let mut place = HashMap::new();
        for (name, def) in &cfg.place {
            let uses = resolve_item_map(&def.uses)?;
            let where_materials = def
                .where_
                .iter()
                .map(|m| {
                    Material::from_name(m).ok_or_else(|| format!("unknown material: {m}"))
                })
                .collect::<Result<Vec<_>, _>>()?;
            let place_type = match def.type_.as_str() {
                "material" => PlaceType::Material,
                "object" => PlaceType::Object,
                other => return Err(format!("unknown place type: {other}")),
            };
            let placed_material = Material::from_name(name);
            let achievement_name = format!("place_{name}");
            let achievement = Achievement::from_name(&achievement_name)
                .ok_or_else(|| format!("unknown achievement: {achievement_name}"))?;

            place.insert(
                name.clone(),
                PlaceRule {
                    uses,
                    where_materials,
                    place_type,
                    placed_material,
                    achievement,
                },
            );
        }

        // Make
        let mut make = HashMap::new();
        for (name, def) in &cfg.make {
            let item = ItemKind::from_name(name)
                .ok_or_else(|| format!("unknown make item: {name}"))?;
            let uses = resolve_item_map(&def.uses)?;
            let nearby = def
                .nearby
                .iter()
                .map(|m| {
                    Material::from_name(m).ok_or_else(|| format!("unknown material: {m}"))
                })
                .collect::<Result<Vec<_>, _>>()?;
            let achievement_name = format!("make_{name}");
            let achievement = Achievement::from_name(&achievement_name)
                .ok_or_else(|| format!("unknown achievement: {achievement_name}"))?;

            make.insert(
                item,
                MakeRule {
                    uses,
                    nearby,
                    gives: def.gives,
                    achievement,
                },
            );
        }

        Ok(GameRules {
            item_initial,
            item_max,
            collect,
            place,
            make,
            walkable,
        })
    }
}

impl Default for GameRules {
    fn default() -> Self {
        Self::from_yaml(DEFAULT_YAML).expect("embedded config.yaml must be valid")
    }
}

fn resolve_item_map(map: &HashMap<String, i32>) -> Result<Vec<(ItemKind, i32)>, String> {
    map.iter()
        .map(|(name, amount)| {
            let kind = ItemKind::from_name(name)
                .ok_or_else(|| format!("unknown item: {name}"))?;
            Ok((kind, *amount))
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_rules_load_successfully() {
        let rules = GameRules::default();
        assert_eq!(rules.item_initial[ItemKind::Health as usize], 9);
        assert_eq!(rules.item_initial[ItemKind::Wood as usize], 0);
        assert_eq!(rules.item_max[ItemKind::Health as usize], 9);
        assert_eq!(rules.collect.len(), 7);
        assert_eq!(rules.place.len(), 4);
        assert_eq!(rules.make.len(), 6);
        assert_eq!(rules.walkable.len(), 3);
    }

    #[test]
    fn collect_rules_resolve_correctly() {
        let rules = GameRules::default();
        let tree_rule = rules.collect.get(&Material::Tree).unwrap();
        assert!(tree_rule.require.is_empty());
        assert_eq!(tree_rule.receive, vec![(ItemKind::Wood, 1)]);
        assert_eq!(tree_rule.leaves, Material::Grass);
        assert_eq!(tree_rule.probability, 1.0);
        assert_eq!(tree_rule.achievement, Achievement::CollectWood);
    }

    #[test]
    fn make_rules_resolve_correctly() {
        let rules = GameRules::default();
        let iron_pickaxe = rules.make.get(&ItemKind::IronPickaxe).unwrap();
        assert_eq!(iron_pickaxe.gives, 1);
        assert!(iron_pickaxe.nearby.contains(&Material::Table));
        assert!(iron_pickaxe.nearby.contains(&Material::Furnace));
        assert_eq!(iron_pickaxe.achievement, Achievement::MakeIronPickaxe);
    }
}
