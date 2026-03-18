use std::collections::HashMap;

use serde::Deserialize;

use crate::registry::{EntityTypeId, ItemDef, ItemId, MaterialDef, MaterialId, Registry};
use crate::Direction;

const DEFAULT_YAML: &str = include_str!("../data/config.yaml");

// ---------------------------------------------------------------------------
// Runtime rule types (pre-resolved IDs, no string lookups during gameplay)
// ---------------------------------------------------------------------------

#[derive(Clone, Debug)]
pub struct WellKnownItems {
    pub health: ItemId,
    pub food: ItemId,
    pub drink: ItemId,
    pub energy: ItemId,
}

#[derive(Clone, Debug)]
pub struct GameRules {
    pub registry: Registry,
    pub item_initial: Vec<i32>,
    pub item_max: Vec<i32>,
    pub collect: HashMap<MaterialId, CollectRule>,
    pub place: HashMap<String, PlaceRule>,
    pub make: HashMap<ItemId, MakeRule>,
    pub walkable: Vec<MaterialId>,
    pub player_walkable: Vec<MaterialId>,
    pub arrow_walkable: Vec<MaterialId>,
    pub actions: Vec<ActionDef>,
    pub achievement_names: Vec<String>,
    pub achievement_by_name: HashMap<String, usize>,
    pub balance: BalanceConfig,
    pub entity_defs: Vec<EntityDef>,
    pub entity_by_name: HashMap<String, EntityTypeId>,
    pub well_known: WellKnownItems,
    pub sleep_action_index: usize,
}

impl GameRules {
    pub fn action_count(&self) -> usize {
        self.actions.len()
    }

    pub fn achievement_count(&self) -> usize {
        self.achievement_names.len()
    }

    pub fn achievement_index(&self, name: &str) -> usize {
        self.achievement_by_name[name]
    }

    pub fn entity_def(&self, type_id: EntityTypeId) -> &EntityDef {
        &self.entity_defs[type_id.0 as usize]
    }

    pub fn entity_type_id(&self, name: &str) -> Option<EntityTypeId> {
        self.entity_by_name.get(name).copied()
    }

    pub fn action_index(&self, name: &str) -> Option<usize> {
        self.actions.iter().position(|a| a.name == name)
    }
}

#[derive(Clone, Debug)]
pub enum ActionKind {
    Noop,
    Move(Direction),
    Do,
    Sleep,
    Place(String),
    Make(ItemId),
}

#[derive(Clone, Debug)]
pub struct ActionDef {
    pub name: String,
    pub kind: ActionKind,
}

#[derive(Clone, Debug)]
pub struct CollectRule {
    pub require: Vec<(ItemId, i32)>,
    pub receive: Vec<(ItemId, i32)>,
    pub leaves: MaterialId,
    pub probability: f32,
    pub achievement: usize,
}

#[derive(Clone, Debug)]
pub enum PlaceType {
    Material,
    Object,
}

#[derive(Clone, Debug)]
pub struct PlaceRule {
    pub uses: Vec<(ItemId, i32)>,
    pub where_materials: Vec<MaterialId>,
    pub place_type: PlaceType,
    pub placed_material: Option<MaterialId>,
    pub achievement: usize,
}

#[derive(Clone, Debug)]
pub struct MakeRule {
    pub uses: Vec<(ItemId, i32)>,
    pub nearby: Vec<MaterialId>,
    pub gives: i32,
    pub achievement: usize,
}

// ---------------------------------------------------------------------------
// Entity definitions (data-driven, post-resolution)
// ---------------------------------------------------------------------------

#[derive(Clone, Debug)]
pub enum EntityBehavior {
    Passive { move_prob: f32 },
    Melee { chase_dist: i32, chase_prob: f32, direct_chase: f32, damage: i32, sleeping_damage: i32, cooldown: i32 },
    Ranged { flee_dist: i32, shoot_dist: i32, chase_dist: i32, reload: i32, flee_prob: f32, shoot_prob: f32, chase_prob: f32, wander_prob: f32, projectile: EntityTypeId },
    Projectile { damage: i32 },
    Growing { ripen_time: i32 },
    Static,
}

#[derive(Clone, Debug)]
pub struct EntityDef {
    pub name: String,
    pub type_id: EntityTypeId,
    pub health: i32,
    pub behavior: EntityBehavior,
    pub drops: Vec<(ItemId, i32)>,
    pub on_kill: Option<usize>,  // achievement index
    pub semantic_id: u16,
    pub spawning: Option<SpawnConfig>,
    pub worldgen: Option<WorldgenEntityConfig>,
}

#[derive(Clone, Debug)]
pub struct WorldgenEntityConfig {
    pub min_dist: f64,
    pub threshold: f64,
    pub material: Option<MaterialId>,
    pub tunnel_only: bool,
}

// ---------------------------------------------------------------------------
// Balance configuration (runtime, post-resolution)
// ---------------------------------------------------------------------------

#[derive(Clone, Debug)]
pub struct BalanceConfig {
    pub daylight_cycle: f32,
    pub player: PlayerBalance,
    pub combat: CombatBalance,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(default)]
pub struct PlayerBalance {
    pub hunger_threshold: f32,
    pub thirst_threshold: f32,
    pub fatigue_min: i32,
    pub fatigue_max: i32,
    pub recover_gain: f32,
    pub recover_loss: f32,
    pub sleep_consumption_factor: f32,
    pub sleep_recovery_factor: f32,
    pub sleep_degeneration_factor: f32,
}

impl Default for PlayerBalance {
    fn default() -> Self {
        Self {
            hunger_threshold: 25.0,
            thirst_threshold: 20.0,
            fatigue_min: -10,
            fatigue_max: 30,
            recover_gain: 25.0,
            recover_loss: -15.0,
            sleep_consumption_factor: 0.5,
            sleep_recovery_factor: 2.0,
            sleep_degeneration_factor: 0.5,
        }
    }
}

#[derive(Clone, Debug)]
pub struct CombatBalance {
    pub base_damage: i32,
    pub swords: Vec<(ItemId, i32)>,
}

#[derive(Clone, Debug)]
pub struct SpawnConfig {
    pub material: MaterialId,
    pub spawn_dist: i32,
    pub despawn_dist: i32,
    pub spawn_prob: f32,
    pub despawn_prob: f32,
    pub min_space: usize,
    pub base_target: f32,
    pub max_target: Option<f32>,
    pub light_factor: f32,
}

/// Compute (min_target, max_target) for entity spawning balance.
pub fn spawn_targets(cfg: &SpawnConfig, light: f32, space: usize) -> (f32, f32) {
    let has_explicit_max = cfg.max_target.is_some();
    let max_base = cfg.max_target.unwrap_or(cfg.base_target);

    let min = if space < cfg.min_space {
        0.0
    } else if has_explicit_max {
        cfg.base_target
    } else {
        cfg.base_target + cfg.light_factor * light
    };

    let max = max_base + cfg.light_factor * light;
    (min, max)
}

// ---------------------------------------------------------------------------
// Intermediate serde types for YAML parsing
// ---------------------------------------------------------------------------

#[derive(Deserialize)]
struct YamlConfig {
    #[serde(default = "default_materials")]
    materials: Vec<YamlMaterialDef>,
    items: Vec<YamlItemDef>,
    collect: HashMap<String, YamlCollectDef>,
    place: HashMap<String, YamlPlaceDef>,
    make: HashMap<String, YamlMakeDef>,
    #[serde(default)]
    balance: YamlBalance,
    #[serde(default)]
    entities: indexmap::IndexMap<String, YamlEntityDef>,
}

#[derive(Deserialize)]
struct YamlMaterialDef {
    name: String,
    #[serde(default)]
    walkable: bool,
    #[serde(default)]
    player_walkable: bool,
    #[serde(default)]
    arrow_walkable: bool,
}

fn default_materials() -> Vec<YamlMaterialDef> {
    vec![
        YamlMaterialDef { name: "water".into(), walkable: false, player_walkable: false, arrow_walkable: true },
        YamlMaterialDef { name: "grass".into(), walkable: true, player_walkable: true, arrow_walkable: true },
        YamlMaterialDef { name: "stone".into(), walkable: false, player_walkable: false, arrow_walkable: false },
        YamlMaterialDef { name: "path".into(), walkable: true, player_walkable: true, arrow_walkable: true },
        YamlMaterialDef { name: "sand".into(), walkable: true, player_walkable: true, arrow_walkable: true },
        YamlMaterialDef { name: "tree".into(), walkable: false, player_walkable: false, arrow_walkable: false },
        YamlMaterialDef { name: "lava".into(), walkable: false, player_walkable: true, arrow_walkable: true },
        YamlMaterialDef { name: "coal".into(), walkable: false, player_walkable: false, arrow_walkable: false },
        YamlMaterialDef { name: "iron".into(), walkable: false, player_walkable: false, arrow_walkable: false },
        YamlMaterialDef { name: "diamond".into(), walkable: false, player_walkable: false, arrow_walkable: false },
        YamlMaterialDef { name: "table".into(), walkable: false, player_walkable: false, arrow_walkable: false },
        YamlMaterialDef { name: "furnace".into(), walkable: false, player_walkable: false, arrow_walkable: false },
    ]
}

#[derive(Deserialize)]
struct YamlItemDef {
    name: String,
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

// ----- Balance YAML types -----

#[derive(Deserialize)]
#[serde(default)]
struct YamlBalance {
    daylight_cycle: f32,
    player: PlayerBalance,
    combat: YamlCombatBalance,
}

impl Default for YamlBalance {
    fn default() -> Self {
        Self {
            daylight_cycle: 300.0,
            player: PlayerBalance::default(),
            combat: YamlCombatBalance::default(),
        }
    }
}

#[derive(Deserialize)]
#[serde(default)]
struct YamlCombatBalance {
    base_damage: i32,
    swords: HashMap<String, i32>,
}

impl Default for YamlCombatBalance {
    fn default() -> Self {
        let mut swords = HashMap::new();
        swords.insert("wood_sword".into(), 2);
        swords.insert("stone_sword".into(), 3);
        swords.insert("iron_sword".into(), 5);
        Self {
            base_damage: 1,
            swords,
        }
    }
}

// ----- Entity YAML types -----

#[derive(Deserialize, Default)]
#[serde(default)]
struct YamlEntityDef {
    #[serde(default)]
    health: Option<i32>,
    behavior: YamlEntityBehavior,
    #[serde(default)]
    drops: HashMap<String, i32>,
    #[serde(default)]
    on_kill: Option<String>,
    #[serde(default)]
    spawning: Option<YamlSpawnConfig>,
    #[serde(default)]
    worldgen: Option<YamlWorldgenEntityConfig>,
}

#[derive(Deserialize)]
#[serde(tag = "type")]
#[serde(rename_all = "snake_case")]
enum YamlEntityBehavior {
    Passive {
        #[serde(default = "default_move_prob")]
        move_prob: f32,
    },
    Melee {
        #[serde(default = "default_chase_dist")]
        chase_dist: i32,
        #[serde(default = "default_chase_prob")]
        chase_prob: f32,
        #[serde(default = "default_direct_chase")]
        direct_chase: f32,
        #[serde(default = "default_melee_damage")]
        damage: i32,
        #[serde(default = "default_sleeping_damage")]
        sleeping_damage: i32,
        #[serde(default = "default_cooldown")]
        cooldown: i32,
    },
    Ranged {
        #[serde(default = "default_flee_dist")]
        flee_dist: i32,
        #[serde(default = "default_shoot_dist")]
        shoot_dist: i32,
        #[serde(default = "default_chase_dist")]
        chase_dist: i32,
        #[serde(default = "default_reload")]
        reload: i32,
        #[serde(default = "default_flee_prob")]
        flee_prob: f32,
        #[serde(default = "default_shoot_prob")]
        shoot_prob: f32,
        #[serde(default = "default_ranged_chase_prob")]
        chase_prob: f32,
        #[serde(default = "default_wander_prob")]
        wander_prob: f32,
        projectile: String,
    },
    Projectile {
        #[serde(default = "default_arrow_damage")]
        damage: i32,
    },
    Growing {
        #[serde(default = "default_ripen_time")]
        ripen_time: i32,
    },
    Static,
}

impl Default for YamlEntityBehavior {
    fn default() -> Self {
        YamlEntityBehavior::Static
    }
}

fn default_move_prob() -> f32 { 0.5 }
fn default_chase_dist() -> i32 { 8 }
fn default_chase_prob() -> f32 { 0.9 }
fn default_direct_chase() -> f32 { 0.8 }
fn default_melee_damage() -> i32 { 2 }
fn default_sleeping_damage() -> i32 { 7 }
fn default_cooldown() -> i32 { 5 }
fn default_flee_dist() -> i32 { 3 }
fn default_shoot_dist() -> i32 { 5 }
fn default_reload() -> i32 { 4 }
fn default_flee_prob() -> f32 { 0.6 }
fn default_shoot_prob() -> f32 { 0.5 }
fn default_ranged_chase_prob() -> f32 { 0.3 }
fn default_wander_prob() -> f32 { 0.2 }
fn default_arrow_damage() -> i32 { 2 }
fn default_ripen_time() -> i32 { 300 }

#[derive(Deserialize)]
#[serde(default)]
struct YamlSpawnConfig {
    material: String,
    spawn_dist: i32,
    despawn_dist: i32,
    spawn_prob: f32,
    despawn_prob: f32,
    min_space: usize,
    base_target: f32,
    max_target: Option<f32>,
    light_factor: f32,
}

impl Default for YamlSpawnConfig {
    fn default() -> Self {
        Self {
            material: "grass".into(),
            spawn_dist: 6,
            despawn_dist: 0,
            spawn_prob: 0.3,
            despawn_prob: 0.4,
            min_space: 50,
            base_target: 3.5,
            max_target: None,
            light_factor: 0.0,
        }
    }
}

#[derive(Deserialize)]
#[serde(default)]
struct YamlWorldgenEntityConfig {
    #[serde(default)]
    min_dist: f64,
    threshold: f64,
    #[serde(default)]
    material: Option<String>,
    #[serde(default)]
    tunnel_only: bool,
}

impl Default for YamlWorldgenEntityConfig {
    fn default() -> Self {
        Self {
            min_dist: 0.0,
            threshold: 1.0,
            material: None,
            tunnel_only: false,
        }
    }
}

// ---------------------------------------------------------------------------
// Resolution: YAML strings → ID-indexed runtime types
// ---------------------------------------------------------------------------

impl GameRules {
    pub fn from_yaml(yaml: &str) -> Result<Self, String> {
        let cfg: YamlConfig =
            serde_yaml::from_str(yaml).map_err(|e| format!("YAML parse error: {e}"))?;
        Self::resolve(cfg)
    }

    fn resolve(cfg: YamlConfig) -> Result<Self, String> {
        // Build registry from materials + items config
        let mat_defs: Vec<MaterialDef> = cfg
            .materials
            .into_iter()
            .enumerate()
            .map(|(i, m)| MaterialDef {
                name: m.name,
                id: MaterialId((i + 1) as u16),
                walkable: m.walkable,
                player_walkable: m.player_walkable,
                arrow_walkable: m.arrow_walkable,
            })
            .collect();

        let item_defs: Vec<ItemDef> = cfg
            .items
            .into_iter()
            .enumerate()
            .map(|(i, it)| ItemDef {
                name: it.name,
                id: ItemId(i as u16),
                max: it.max,
                initial: it.initial,
            })
            .collect();

        let registry = Registry::new(mat_defs, item_defs);

        // Item initial/max from registry
        let item_initial: Vec<i32> = registry.items.iter().map(|i| i.initial).collect();
        let item_max: Vec<i32> = registry.items.iter().map(|i| i.max).collect();

        // Walkable lists from registry
        let walkable = registry.walkable_materials();
        let player_walkable = registry.player_walkable_materials();
        let arrow_walkable = registry.arrow_walkable_materials();

        // Auto-generate achievement list from rules
        // Collect achievements: collect_{received_item}
        let mut achievement_set: Vec<String> = Vec::new();
        for def in cfg.collect.values() {
            if let Some(first_name) = def.receive.keys().next() {
                achievement_set.push(format!("collect_{first_name}"));
            }
        }
        // Place achievements: place_{name}
        for name in cfg.place.keys() {
            achievement_set.push(format!("place_{name}"));
        }
        // Make achievements: make_{name}
        for name in cfg.make.keys() {
            achievement_set.push(format!("make_{name}"));
        }
        // Entity achievements from on_kill fields
        for entity_def in cfg.entities.values() {
            if let Some(ref name) = entity_def.on_kill {
                achievement_set.push(name.clone());
            }
        }
        // Fixed
        achievement_set.push("wake_up".into());
        // Sort alphabetically for parity with existing enum order
        achievement_set.sort();
        achievement_set.dedup();
        let achievement_by_name: HashMap<String, usize> = achievement_set
            .iter()
            .enumerate()
            .map(|(i, name)| (name.clone(), i))
            .collect();

        // Collect
        let mut collect = HashMap::new();
        for (name, def) in &cfg.collect {
            let material_id = registry
                .material_id(name)
                .ok_or_else(|| format!("unknown collect material: {name}"))?;
            let require = resolve_item_map(&registry, &def.require)?;
            let receive = resolve_item_map(&registry, &def.receive)?;
            let leaves = registry
                .material_id(&def.leaves)
                .ok_or_else(|| format!("unknown leaves material: {}", def.leaves))?;

            let first_receive_name = def
                .receive
                .keys()
                .next()
                .ok_or_else(|| format!("collect rule '{name}' has no receive items"))?;
            let achievement_name = format!("collect_{first_receive_name}");
            let achievement = *achievement_by_name
                .get(&achievement_name)
                .ok_or_else(|| format!("unknown achievement: {achievement_name}"))?;

            collect.insert(
                material_id,
                CollectRule {
                    require,
                    receive,
                    leaves,
                    probability: def.probability,
                    achievement,
                },
            );
        }

        // Place rules + actions (sorted for parity)
        let place_order = sorted_place_names(&cfg.place);
        let mut place = HashMap::new();
        for name in &place_order {
            let def = &cfg.place[name];
            let uses = resolve_item_map(&registry, &def.uses)?;
            let where_materials = resolve_material_list(&registry, &def.where_)?;
            let place_type = match def.type_.as_str() {
                "material" => PlaceType::Material,
                "object" => PlaceType::Object,
                other => return Err(format!("unknown place type: {other}")),
            };
            let placed_material = registry.material_id(name);
            let achievement_name = format!("place_{name}");
            let achievement = *achievement_by_name
                .get(&achievement_name)
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

        // Make rules + actions (sorted by ItemId for parity)
        let mut make_entries: Vec<(ItemId, String)> = Vec::new();
        let mut make = HashMap::new();
        for (name, def) in &cfg.make {
            let item_id = registry
                .item_id(name)
                .ok_or_else(|| format!("unknown make item: {name}"))?;
            let uses = resolve_item_map(&registry, &def.uses)?;
            let nearby = resolve_material_list(&registry, &def.nearby)?;
            let achievement_name = format!("make_{name}");
            let achievement = *achievement_by_name
                .get(&achievement_name)
                .ok_or_else(|| format!("unknown achievement: {achievement_name}"))?;

            make.insert(
                item_id,
                MakeRule {
                    uses,
                    nearby,
                    gives: def.gives,
                    achievement,
                },
            );
            make_entries.push((item_id, name.clone()));
        }
        make_entries.sort_by_key(|(id, _)| id.0);

        // Build action list
        let mut actions = vec![
            ActionDef {
                name: "noop".into(),
                kind: ActionKind::Noop,
            },
            ActionDef {
                name: "move_left".into(),
                kind: ActionKind::Move(Direction::Left),
            },
            ActionDef {
                name: "move_right".into(),
                kind: ActionKind::Move(Direction::Right),
            },
            ActionDef {
                name: "move_up".into(),
                kind: ActionKind::Move(Direction::Up),
            },
            ActionDef {
                name: "move_down".into(),
                kind: ActionKind::Move(Direction::Down),
            },
            ActionDef {
                name: "do".into(),
                kind: ActionKind::Do,
            },
            ActionDef {
                name: "sleep".into(),
                kind: ActionKind::Sleep,
            },
        ];
        for name in &place_order {
            actions.push(ActionDef {
                name: format!("place_{name}"),
                kind: ActionKind::Place(name.clone()),
            });
        }
        for (item_id, name) in &make_entries {
            actions.push(ActionDef {
                name: format!("make_{name}"),
                kind: ActionKind::Make(*item_id),
            });
        }

        // Balance
        let balance = resolve_balance(cfg.balance, &registry)?;

        // Entity definitions
        let (entity_defs, entity_by_name) = resolve_entities(
            cfg.entities,
            &registry,
            &achievement_by_name,
        )?;

        // Well-known items
        let well_known = WellKnownItems {
            health: registry.item_id("health")
                .ok_or_else(|| "missing well-known item: health".to_string())?,
            food: registry.item_id("food")
                .ok_or_else(|| "missing well-known item: food".to_string())?,
            drink: registry.item_id("drink")
                .ok_or_else(|| "missing well-known item: drink".to_string())?,
            energy: registry.item_id("energy")
                .ok_or_else(|| "missing well-known item: energy".to_string())?,
        };

        // Sleep action index
        let sleep_action_index = actions.iter()
            .position(|a| matches!(a.kind, ActionKind::Sleep))
            .ok_or_else(|| "missing sleep action".to_string())?;

        Ok(GameRules {
            registry,
            item_initial,
            item_max,
            collect,
            place,
            make,
            walkable,
            player_walkable,
            arrow_walkable,
            actions,
            achievement_names: achievement_set,
            achievement_by_name,
            balance,
            entity_defs,
            entity_by_name,
            well_known,
            sleep_action_index,
        })
    }
}

impl Default for GameRules {
    fn default() -> Self {
        Self::from_yaml(DEFAULT_YAML).expect("embedded config.yaml must be valid")
    }
}

fn resolve_balance(yaml: YamlBalance, registry: &Registry) -> Result<BalanceConfig, String> {
    let swords = resolve_item_map(registry, &yaml.combat.swords)?;

    Ok(BalanceConfig {
        daylight_cycle: yaml.daylight_cycle,
        player: yaml.player,
        combat: CombatBalance {
            base_damage: yaml.combat.base_damage,
            swords,
        },
    })
}

fn resolve_entities(
    entities: indexmap::IndexMap<String, YamlEntityDef>,
    registry: &Registry,
    achievement_by_name: &HashMap<String, usize>,
) -> Result<(Vec<EntityDef>, HashMap<String, EntityTypeId>), String> {
    // First pass: create a name-to-id map so we can resolve projectile references
    let mut name_to_id: HashMap<String, EntityTypeId> = HashMap::new();
    for (i, name) in entities.keys().enumerate() {
        name_to_id.insert(name.clone(), EntityTypeId(i as u16));
    }

    let mut defs = Vec::new();
    for (i, (name, yaml_def)) in entities.iter().enumerate() {
        let type_id = EntityTypeId(i as u16);
        let semantic_id = 14 + i as u16; // player=13, entities start at 14

        let behavior = match &yaml_def.behavior {
            YamlEntityBehavior::Passive { move_prob } => {
                EntityBehavior::Passive { move_prob: *move_prob }
            }
            YamlEntityBehavior::Melee { chase_dist, chase_prob, direct_chase, damage, sleeping_damage, cooldown } => {
                EntityBehavior::Melee {
                    chase_dist: *chase_dist,
                    chase_prob: *chase_prob,
                    direct_chase: *direct_chase,
                    damage: *damage,
                    sleeping_damage: *sleeping_damage,
                    cooldown: *cooldown,
                }
            }
            YamlEntityBehavior::Ranged { flee_dist, shoot_dist, chase_dist, reload, flee_prob, shoot_prob, chase_prob, wander_prob, projectile } => {
                let proj_id = name_to_id.get(projectile)
                    .copied()
                    .ok_or_else(|| format!("unknown projectile entity: {projectile}"))?;
                EntityBehavior::Ranged {
                    flee_dist: *flee_dist,
                    shoot_dist: *shoot_dist,
                    chase_dist: *chase_dist,
                    reload: *reload,
                    flee_prob: *flee_prob,
                    shoot_prob: *shoot_prob,
                    chase_prob: *chase_prob,
                    wander_prob: *wander_prob,
                    projectile: proj_id,
                }
            }
            YamlEntityBehavior::Projectile { damage } => {
                EntityBehavior::Projectile { damage: *damage }
            }
            YamlEntityBehavior::Growing { ripen_time } => {
                EntityBehavior::Growing { ripen_time: *ripen_time }
            }
            YamlEntityBehavior::Static => EntityBehavior::Static,
        };

        let drops = resolve_item_map(registry, &yaml_def.drops)?;

        let on_kill = if let Some(ref kill_name) = yaml_def.on_kill {
            Some(*achievement_by_name.get(kill_name)
                .ok_or_else(|| format!("unknown on_kill achievement: {kill_name}"))?)
        } else {
            None
        };

        let spawning = if let Some(ref yaml_spawn) = yaml_def.spawning {
            let mat = registry.material_id(&yaml_spawn.material)
                .ok_or_else(|| format!("unknown spawn material: {}", yaml_spawn.material))?;
            Some(resolve_spawn_config_from_yaml(yaml_spawn, mat)?)
        } else {
            None
        };

        let worldgen = if let Some(ref yaml_wg) = yaml_def.worldgen {
            let mat = if let Some(ref mat_name) = yaml_wg.material {
                Some(registry.material_id(mat_name)
                    .ok_or_else(|| format!("unknown worldgen material: {mat_name}"))?)
            } else {
                None
            };
            Some(WorldgenEntityConfig {
                min_dist: yaml_wg.min_dist,
                threshold: yaml_wg.threshold,
                material: mat,
                tunnel_only: yaml_wg.tunnel_only,
            })
        } else {
            None
        };

        defs.push(EntityDef {
            name: name.clone(),
            type_id,
            health: yaml_def.health.unwrap_or(0),
            behavior,
            drops,
            on_kill,
            semantic_id,
            spawning,
            worldgen,
        });
    }

    Ok((defs, name_to_id))
}

fn resolve_spawn_config_from_yaml(
    yaml: &YamlSpawnConfig,
    material: MaterialId,
) -> Result<SpawnConfig, String> {
    Ok(SpawnConfig {
        material,
        spawn_dist: yaml.spawn_dist,
        despawn_dist: yaml.despawn_dist,
        spawn_prob: yaml.spawn_prob,
        despawn_prob: yaml.despawn_prob,
        min_space: yaml.min_space,
        base_target: yaml.base_target,
        max_target: yaml.max_target,
        light_factor: yaml.light_factor,
    })
}

fn resolve_item_map(
    registry: &Registry,
    map: &HashMap<String, i32>,
) -> Result<Vec<(ItemId, i32)>, String> {
    map.iter()
        .map(|(name, amount)| {
            let id = registry
                .item_id(name)
                .ok_or_else(|| format!("unknown item: {name}"))?;
            Ok((id, *amount))
        })
        .collect()
}

fn sorted_place_names(place: &HashMap<String, YamlPlaceDef>) -> Vec<String> {
    // Sort known place names in their original enum order for parity
    const KNOWN: &[&str] = &["stone", "table", "furnace", "plant"];
    let mut result: Vec<String> = Vec::new();
    for name in KNOWN {
        if place.contains_key(*name) {
            result.push(name.to_string());
        }
    }
    let mut unknown: Vec<String> = place
        .keys()
        .filter(|k| !KNOWN.contains(&k.as_str()))
        .cloned()
        .collect();
    unknown.sort();
    result.extend(unknown);
    result
}

fn resolve_material_list(
    registry: &Registry,
    names: &[String],
) -> Result<Vec<MaterialId>, String> {
    names
        .iter()
        .map(|name| {
            registry
                .material_id(name)
                .ok_or_else(|| format!("unknown material: {name}"))
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Material;

    #[test]
    fn default_rules_load_successfully() {
        let rules = GameRules::default();
        assert!(rules.registry.item_count() >= 16);
        assert!(rules.registry.material_count() >= 12);
        assert!(rules.collect.len() >= 7);
        assert!(rules.place.len() >= 4);
        assert!(rules.make.len() >= 6);
        assert!(rules.walkable.len() >= 3);
        assert!(rules.player_walkable.len() >= 4);
        assert!(rules.arrow_walkable.len() >= 5);
    }

    #[test]
    fn registry_parity_materials() {
        let rules = GameRules::default();
        let r = &rules.registry;
        for mat in Material::ALL {
            let name = match mat {
                Material::Water => "water",
                Material::Grass => "grass",
                Material::Stone => "stone",
                Material::Path => "path",
                Material::Sand => "sand",
                Material::Tree => "tree",
                Material::Lava => "lava",
                Material::Coal => "coal",
                Material::Iron => "iron",
                Material::Diamond => "diamond",
                Material::Table => "table",
                Material::Furnace => "furnace",
            };
            let reg_id = r.material_id(name).unwrap();
            assert_eq!(
                reg_id,
                mat.id(),
                "material {name}: registry id {reg_id:?} != enum id {:?}",
                mat.id()
            );
        }
    }

    #[test]
    fn collect_rules_resolve_correctly() {
        let rules = GameRules::default();
        let r = &rules.registry;
        let tree_rule = rules.collect.get(&Material::Tree.id()).unwrap();
        assert!(tree_rule.require.is_empty());
        assert_eq!(tree_rule.receive, vec![(r.item_id("wood").unwrap(), 1)]);
        assert_eq!(tree_rule.leaves, Material::Grass.id());
        assert_eq!(tree_rule.probability, 1.0);
        assert_eq!(tree_rule.achievement, rules.achievement_index("collect_wood"));
    }

    #[test]
    fn make_rules_resolve_correctly() {
        let rules = GameRules::default();
        let r = &rules.registry;
        let iron_pickaxe = rules.make.get(&r.item_id("iron_pickaxe").unwrap()).unwrap();
        assert_eq!(iron_pickaxe.gives, 1);
        assert!(iron_pickaxe.nearby.contains(&Material::Table.id()));
        assert!(iron_pickaxe.nearby.contains(&Material::Furnace.id()));
        assert_eq!(iron_pickaxe.achievement, rules.achievement_index("make_iron_pickaxe"));
    }

    #[test]
    fn balance_defaults_match_hardcoded_values() {
        let rules = GameRules::default();
        let r = &rules.registry;
        let b = &rules.balance;
        assert_eq!(b.daylight_cycle, 300.0);
        assert_eq!(b.player.hunger_threshold, 25.0);
        assert_eq!(b.combat.base_damage, 1);
        assert!(b.combat.swords.contains(&(r.item_id("wood_sword").unwrap(), 2)));
        assert!(b.combat.swords.contains(&(r.item_id("stone_sword").unwrap(), 3)));
        assert!(b.combat.swords.contains(&(r.item_id("iron_sword").unwrap(), 5)));

        // Entity definitions
        assert_eq!(rules.entity_defs.len(), 6);
        let cow_def = rules.entity_def(rules.entity_type_id("cow").unwrap());
        assert_eq!(cow_def.health, 3);
        assert!(cow_def.drops.contains(&(rules.well_known.food, 6)));
        let zombie_def = rules.entity_def(rules.entity_type_id("zombie").unwrap());
        assert_eq!(zombie_def.health, 5);
        let skeleton_def = rules.entity_def(rules.entity_type_id("skeleton").unwrap());
        assert_eq!(skeleton_def.health, 3);
        let plant_def = rules.entity_def(rules.entity_type_id("plant").unwrap());
        assert_eq!(plant_def.health, 1);
        assert!(plant_def.drops.contains(&(rules.well_known.food, 4)));

        // Spawning
        let cow_spawn = cow_def.spawning.as_ref().unwrap();
        assert_eq!(cow_spawn.material, Material::Grass.id());
        let zombie_spawn = zombie_def.spawning.as_ref().unwrap();
        assert_eq!(zombie_spawn.material, Material::Grass.id());
        let skeleton_spawn = skeleton_def.spawning.as_ref().unwrap();
        assert_eq!(skeleton_spawn.material, Material::Path.id());

        // Worldgen
        let cow_wg = cow_def.worldgen.as_ref().unwrap();
        assert_eq!(cow_wg.min_dist, 3.0);
        let zombie_wg = zombie_def.worldgen.as_ref().unwrap();
        assert_eq!(zombie_wg.threshold, 0.993);
    }

    #[test]
    fn well_known_items_resolve() {
        let rules = GameRules::default();
        assert_eq!(rules.well_known.health, ItemId(0));
        assert_eq!(rules.well_known.food, ItemId(1));
        assert_eq!(rules.well_known.drink, ItemId(2));
        assert_eq!(rules.well_known.energy, ItemId(3));
    }

    #[test]
    fn sleep_action_index_resolves() {
        let rules = GameRules::default();
        assert_eq!(rules.actions[rules.sleep_action_index].name, "sleep");
    }

    #[test]
    fn action_index_lookup() {
        let rules = GameRules::default();
        assert_eq!(rules.action_index("noop"), Some(0));
        assert_eq!(rules.action_index("do"), Some(5));
        assert_eq!(rules.action_index("sleep"), Some(6));
        assert_eq!(rules.action_index("nonexistent"), None);
    }

    #[test]
    fn walkable_lists_derived_from_materials() {
        let rules = GameRules::default();
        assert_eq!(
            rules.walkable,
            vec![Material::Grass.id(), Material::Path.id(), Material::Sand.id()]
        );
        assert_eq!(
            rules.player_walkable,
            vec![
                Material::Grass.id(),
                Material::Path.id(),
                Material::Sand.id(),
                Material::Lava.id()
            ]
        );
        assert_eq!(
            rules.arrow_walkable,
            vec![
                Material::Water.id(),
                Material::Grass.id(),
                Material::Path.id(),
                Material::Sand.id(),
                Material::Lava.id()
            ]
        );
    }

    #[test]
    fn action_parity() {
        let rules = GameRules::default();
        let names: Vec<&str> = rules.actions.iter().map(|a| a.name.as_str()).collect();
        assert_eq!(
            names,
            vec![
                "noop", "move_left", "move_right", "move_up", "move_down",
                "do", "sleep",
                "place_stone", "place_table", "place_furnace", "place_plant",
                "make_wood_pickaxe", "make_stone_pickaxe", "make_iron_pickaxe",
                "make_wood_sword", "make_stone_sword", "make_iron_sword",
            ]
        );
    }

    #[test]
    fn achievement_parity() {
        let rules = GameRules::default();
        assert!(rules.achievement_names.len() >= 22);
        let expected = [
            "collect_coal", "collect_diamond", "collect_drink", "collect_iron",
            "collect_sapling", "collect_stone", "collect_wood",
            "defeat_skeleton", "defeat_zombie", "eat_cow", "eat_plant",
            "make_iron_pickaxe", "make_iron_sword", "make_stone_pickaxe",
            "make_stone_sword", "make_wood_pickaxe", "make_wood_sword",
            "place_furnace", "place_plant", "place_stone", "place_table",
            "wake_up",
        ];
        for name in &expected {
            assert!(
                rules.achievement_by_name.contains_key(*name),
                "missing achievement: {name}"
            );
        }
    }
}
