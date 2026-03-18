use std::collections::BTreeMap;
use std::fs;
use std::path::PathBuf;
use std::sync::OnceLock;

use crafter_rs::{
    ACTION_NAMES, Action, Direction, Env, EnvConfig, Frame, ITEM_ORDER, ItemKind,
    Material, SemanticGrid,
};
use serde::Deserialize;
use sha2::{Digest, Sha256};

#[derive(Debug, Deserialize)]
struct FixtureData {
    action_names: Vec<String>,
    daylight_timeline: Vec<DaylightPoint>,
    render_scenarios: Vec<RenderScenario>,
    night_render_scenarios: Vec<NightRenderScenario>,
    step_scenarios: Vec<StepScenario>,
}

#[derive(Debug, Deserialize)]
struct DaylightPoint {
    label: Option<String>,
    action: Option<String>,
    daylight: f64,
}

#[derive(Debug, Deserialize)]
struct RenderScenario {
    name: String,
    setup: ScenarioSetup,
    snapshot: SnapshotFixture,
}

#[derive(Debug, Deserialize)]
struct NightRenderScenario {
    name: String,
    setup: ScenarioSetup,
    metrics: NightRenderMetrics,
}

#[derive(Debug, Deserialize)]
struct NightRenderMetrics {
    unit: usize,
    edge_tile: [usize; 2],
    inner_tile: [usize; 2],
    edge_luma: f64,
    inner_luma: f64,
    frames_differ: bool,
}

#[derive(Debug, Deserialize)]
struct StepScenario {
    name: String,
    setup: ScenarioSetup,
    actions: Vec<String>,
    snapshots: Vec<SnapshotFixture>,
}

#[derive(Debug, Deserialize)]
struct ScenarioSetup {
    config: ConfigFixture,
    #[serde(default)]
    render_size: Option<[usize; 2]>,
    world_fill: String,
    #[serde(default)]
    materials: Vec<MaterialPatch>,
    #[serde(default)]
    objects: Vec<ObjectSpec>,
    #[serde(default)]
    player: PlayerSetup,
    #[serde(default)]
    daylight: Option<f64>,
}

#[derive(Debug, Deserialize)]
struct ConfigFixture {
    area: [usize; 2],
    view: [usize; 2],
    size: [usize; 2],
    reward: bool,
    length: u32,
    seed: u64,
}

#[derive(Debug, Default, Deserialize)]
struct PlayerSetup {
    #[serde(default)]
    pos: Option<[usize; 2]>,
    #[serde(default)]
    facing: Option<String>,
    #[serde(default)]
    sleeping: Option<bool>,
    #[serde(default)]
    inventory: BTreeMap<String, i32>,
    #[serde(default)]
    hunger: Option<f64>,
    #[serde(default)]
    thirst: Option<f64>,
    #[serde(default)]
    fatigue: Option<i32>,
    #[serde(default)]
    recover: Option<f64>,
    #[serde(default)]
    last_health: Option<i32>,
}

#[derive(Debug, Deserialize)]
struct MaterialPatch {
    pos: [usize; 2],
    material: String,
}

#[derive(Debug, Deserialize)]
struct ObjectSpec {
    kind: String,
    pos: [usize; 2],
    #[serde(default)]
    facing: Option<String>,
}

#[derive(Debug, Deserialize)]
struct SnapshotFixture {
    frame: FrameFixture,
    semantic: SemanticFixture,
    player_pos: [usize; 2],
    sleeping: bool,
    daylight: f64,
    inventory: BTreeMap<String, i32>,
    achievements: BTreeMap<String, u32>,
    #[serde(default)]
    reward: Option<f64>,
    #[serde(default)]
    done: Option<bool>,
    #[serde(default)]
    discount: Option<f64>,
}

#[derive(Debug, Deserialize)]
struct FrameFixture {
    width: usize,
    height: usize,
    channels: usize,
    sha256: String,
}

#[derive(Debug, Deserialize)]
struct SemanticFixture {
    width: usize,
    height: usize,
    cells: Vec<u16>,
}

#[test]
fn reference_action_names_and_daylight_timeline_match() {
    let fixture = fixture();
    let mut env = Env::default();
    env.reset();

    let action_names: Vec<_> = ACTION_NAMES.into_iter().map(str::to_string).collect();
    assert_eq!(action_names, fixture.action_names);

    assert_eq!(fixture.daylight_timeline.len(), 4);
    assert_eq!(fixture.daylight_timeline[0].label.as_deref(), Some("reset"));
    assert_daylight_close(
        env.world().daylight(),
        fixture.daylight_timeline[0].daylight,
        "reset daylight",
    );

    for point in fixture.daylight_timeline.iter().skip(1) {
        let action = action_by_name(point.action.as_deref().expect("missing action"));
        env.step(action);
        assert_daylight_close(
            env.world().daylight(),
            point.daylight,
            point.action.as_deref().unwrap_or("timeline action"),
        );
    }
}

#[test]
fn reference_render_scenarios_match() {
    let fixture = fixture();
    for scenario in &fixture.render_scenarios {
        let env = env_from_setup(&scenario.setup);
        let render_size = scenario
            .setup
            .render_size
            .expect("render scenario missing render_size");
        let frame = env.render(Some(render_size));
        let semantic = env.semantic_view();
        compare_snapshot(
            &scenario.name,
            &env,
            &frame,
            &semantic,
            &scenario.snapshot,
            None,
            None,
            None,
        );
    }
}

#[test]
fn reference_night_render_metrics_match() {
    let fixture = fixture();
    for scenario in &fixture.night_render_scenarios {
        let env = env_from_setup(&scenario.setup);
        let render_size = scenario
            .setup
            .render_size
            .expect("night render scenario missing render_size");
        let first = env.render(Some(render_size));
        let second = env.render(Some(render_size));
        let metrics = &scenario.metrics;
        let edge = average_tile_luma(
            &first,
            metrics.unit,
            metrics.edge_tile[0],
            metrics.edge_tile[1],
        );
        let inner = average_tile_luma(
            &first,
            metrics.unit,
            metrics.inner_tile[0],
            metrics.inner_tile[1],
        );

        assert!(
            (edge - metrics.edge_luma as f32).abs() <= 3.0,
            "{} edge luma mismatch: expected {:.2}, got {:.2}",
            scenario.name,
            metrics.edge_luma,
            edge
        );
        assert!(
            (inner - metrics.inner_luma as f32).abs() <= 3.0,
            "{} inner luma mismatch: expected {:.2}, got {:.2}",
            scenario.name,
            metrics.inner_luma,
            inner
        );
        assert_eq!(
            sha256_frame(&first) != sha256_frame(&second),
            metrics.frames_differ,
            "{} frame-to-frame noise mismatch",
            scenario.name
        );
    }
}

#[test]
fn reference_step_scenarios_match() {
    let fixture = fixture();
    for scenario in &fixture.step_scenarios {
        let mut env = env_from_setup(&scenario.setup);
        assert_eq!(
            scenario.actions.len(),
            scenario.snapshots.len(),
            "{} has mismatched action/snapshot count",
            scenario.name
        );
        for (index, (action_name, expected)) in scenario
            .actions
            .iter()
            .zip(scenario.snapshots.iter())
            .enumerate()
        {
            let result = env.step(action_by_name(action_name));
            compare_snapshot(
                &format!("{} step {}", scenario.name, index + 1),
                &env,
                &result.observation,
                &result.info.semantic,
                expected,
                Some(result.reward),
                Some(result.done),
                Some(result.info.discount),
            );
        }
    }
}

fn fixture() -> &'static FixtureData {
    static FIXTURE: OnceLock<FixtureData> = OnceLock::new();
    FIXTURE.get_or_init(|| {
        let data = fs::read_to_string(fixture_path()).expect("failed to read parity fixture");
        serde_json::from_str(&data).expect("failed to parse parity fixture")
    })
}

fn fixture_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
        .join("crafter_reference.json")
}

fn env_from_setup(setup: &ScenarioSetup) -> Env {
    let mut env = Env::new(EnvConfig {
        area: setup.config.area,
        view: setup.config.view,
        size: setup.config.size,
        reward: setup.config.reward,
        length: Some(setup.config.length),
        seed: setup.config.seed,
    });
    env.reset();
    env.world_mut().fill(material_by_name(&setup.world_fill).id());
    env.world_mut().clear_objects();

    if let Some(pos) = setup.player.pos {
        env.player_mut().set_pos(pos);
    }
    if let Some(facing) = setup.player.facing.as_deref() {
        env.player_mut().set_facing(direction_by_name(facing));
    }
    if let Some(sleeping) = setup.player.sleeping {
        env.player_mut().set_sleeping(sleeping);
    }
    if let Some(hunger) = setup.player.hunger {
        *env.player_mut().hunger_mut() = hunger as f32;
    }
    if let Some(thirst) = setup.player.thirst {
        *env.player_mut().thirst_mut() = thirst as f32;
    }
    if let Some(fatigue) = setup.player.fatigue {
        *env.player_mut().fatigue_mut() = fatigue;
    }
    if let Some(recover) = setup.player.recover {
        *env.player_mut().recover_mut() = recover as f32;
    }
    if let Some(last_health) = setup.player.last_health {
        env.player_mut().set_last_health(last_health);
    }
    for (name, value) in &setup.player.inventory {
        env.player_mut().set_item(item_by_name(name).id(), *value);
    }

    for patch in &setup.materials {
        env.world_mut()
            .set_material(patch.pos, material_by_name(&patch.material).id());
    }

    if let Some(daylight) = setup.daylight {
        env.world_mut().set_daylight(daylight as f32);
    }

    for object in &setup.objects {
        spawn_object(&mut env, object);
    }

    env
}

fn spawn_object(env: &mut Env, object: &ObjectSpec) {
    let type_id = env.rules().entity_type_id(&object.kind)
        .unwrap_or_else(|| panic!("unsupported object kind in fixture: {}", object.kind));
    let def = env.rules().entity_def(type_id);
    let health = def.health;
    match object.kind.as_str() {
        "cow" => env.world_mut().spawn_cow(object.pos, health),
        "zombie" => env.world_mut().spawn_zombie(object.pos, health),
        "skeleton" => env.world_mut().spawn_skeleton(object.pos, health),
        "plant" => env.world_mut().spawn_plant(object.pos, health, 300),
        "arrow" => env.world_mut().spawn_arrow(
            object.pos,
            direction_by_name(object.facing.as_deref().expect("arrow missing facing")),
        ),
        "fence" => env.world_mut().spawn_fence(object.pos),
        kind => panic!("unsupported object kind in fixture: {kind}"),
    }
}

fn compare_snapshot(
    context: &str,
    env: &Env,
    frame: &Frame,
    semantic: &SemanticGrid,
    expected: &SnapshotFixture,
    reward: Option<f32>,
    done: Option<bool>,
    discount: Option<f32>,
) {
    assert_eq!(frame.width(), expected.frame.width, "{context} frame width");
    assert_eq!(
        frame.height(),
        expected.frame.height,
        "{context} frame height"
    );
    assert_eq!(
        frame.channels(),
        expected.frame.channels,
        "{context} frame channels"
    );
    assert_eq!(
        sha256_frame(frame),
        expected.frame.sha256,
        "{context} frame hash"
    );

    assert_eq!(
        semantic.width(),
        expected.semantic.width,
        "{context} semantic width"
    );
    assert_eq!(
        semantic.height(),
        expected.semantic.height,
        "{context} semantic height"
    );
    assert_eq!(
        semantic.cells(),
        expected.semantic.cells,
        "{context} semantic cells"
    );

    assert_eq!(
        env.player_position(),
        expected.player_pos,
        "{context} player position"
    );
    assert_eq!(
        env.player().sleeping(),
        expected.sleeping,
        "{context} sleeping"
    );
    assert_daylight_close(env.world().daylight(), expected.daylight, context);
    assert_eq!(
        inventory_map(env),
        expected.inventory,
        "{context} inventory mismatch"
    );
    assert_eq!(
        achievement_map(env),
        expected.achievements,
        "{context} achievement mismatch"
    );

    if let Some(expected_reward) = expected.reward {
        assert!(
            (reward.expect("missing reward") as f64 - expected_reward).abs() <= 1e-6,
            "{context} reward mismatch: expected {expected_reward}, got {}",
            reward.unwrap()
        );
    }
    if let Some(expected_done) = expected.done {
        assert_eq!(done.expect("missing done"), expected_done, "{context} done");
    }
    if let Some(expected_discount) = expected.discount {
        assert!(
            (discount.expect("missing discount") as f64 - expected_discount).abs() <= 1e-6,
            "{context} discount mismatch: expected {expected_discount}, got {}",
            discount.unwrap()
        );
    }
}

fn inventory_map(env: &Env) -> BTreeMap<String, i32> {
    ITEM_ORDER
        .into_iter()
        .map(|kind| (kind.name().to_string(), env.player().item(kind.id())))
        .collect()
}

fn achievement_map(env: &Env) -> BTreeMap<String, u32> {
    env.rules()
        .achievement_names
        .iter()
        .enumerate()
        .map(|(i, name)| (name.clone(), env.player().achievements().count(i)))
        .collect()
}

fn sha256_frame(frame: &Frame) -> String {
    let mut hasher = Sha256::new();
    hasher.update(frame.pixels());
    format!("{:x}", hasher.finalize())
}

fn assert_daylight_close(actual: f32, expected: f64, context: &str) {
    assert!(
        (actual as f64 - expected).abs() <= 1e-6,
        "{context} daylight mismatch: expected {expected:.8}, got {actual:.8}"
    );
}

fn average_tile_luma(frame: &Frame, unit: usize, tile_x: usize, tile_y: usize) -> f32 {
    let pixels = tile_pixels(frame, unit, tile_x, tile_y);
    let mut total = 0.0;
    let mut count = 0.0;
    for pixel in pixels.chunks_exact(3) {
        total += 0.299 * pixel[0] as f32 + 0.587 * pixel[1] as f32 + 0.114 * pixel[2] as f32;
        count += 1.0;
    }
    total / count
}

fn tile_pixels(frame: &Frame, unit: usize, tile_x: usize, tile_y: usize) -> Vec<u8> {
    let start_x = tile_x * unit;
    let start_y = tile_y * unit;
    let mut out = Vec::with_capacity(unit * unit * frame.channels());
    for y in start_y..(start_y + unit) {
        for x in start_x..(start_x + unit) {
            let index = (y * frame.width() + x) * frame.channels();
            out.extend_from_slice(&frame.pixels()[index..index + frame.channels()]);
        }
    }
    out
}

fn action_by_name(name: &str) -> Action {
    match name {
        "noop" => Action::Noop,
        "move_left" => Action::MoveLeft,
        "move_right" => Action::MoveRight,
        "move_up" => Action::MoveUp,
        "move_down" => Action::MoveDown,
        "do" => Action::Do,
        "sleep" => Action::Sleep,
        "place_stone" => Action::PlaceStone,
        "place_table" => Action::PlaceTable,
        "place_furnace" => Action::PlaceFurnace,
        "place_plant" => Action::PlacePlant,
        "make_wood_pickaxe" => Action::MakeWoodPickaxe,
        "make_stone_pickaxe" => Action::MakeStonePickaxe,
        "make_iron_pickaxe" => Action::MakeIronPickaxe,
        "make_wood_sword" => Action::MakeWoodSword,
        "make_stone_sword" => Action::MakeStoneSword,
        "make_iron_sword" => Action::MakeIronSword,
        other => panic!("unknown action in fixture: {other}"),
    }
}

fn direction_by_name(name: &str) -> Direction {
    match name {
        "left" => Direction::Left,
        "right" => Direction::Right,
        "up" => Direction::Up,
        "down" => Direction::Down,
        other => panic!("unknown direction in fixture: {other}"),
    }
}

fn material_by_name(name: &str) -> Material {
    match name {
        "water" => Material::Water,
        "grass" => Material::Grass,
        "stone" => Material::Stone,
        "path" => Material::Path,
        "sand" => Material::Sand,
        "tree" => Material::Tree,
        "lava" => Material::Lava,
        "coal" => Material::Coal,
        "iron" => Material::Iron,
        "diamond" => Material::Diamond,
        "table" => Material::Table,
        "furnace" => Material::Furnace,
        other => panic!("unknown material in fixture: {other}"),
    }
}

fn item_by_name(name: &str) -> ItemKind {
    match name {
        "health" => ItemKind::Health,
        "food" => ItemKind::Food,
        "drink" => ItemKind::Drink,
        "energy" => ItemKind::Energy,
        "sapling" => ItemKind::Sapling,
        "wood" => ItemKind::Wood,
        "stone" => ItemKind::Stone,
        "coal" => ItemKind::Coal,
        "iron" => ItemKind::Iron,
        "diamond" => ItemKind::Diamond,
        "wood_pickaxe" => ItemKind::WoodPickaxe,
        "stone_pickaxe" => ItemKind::StonePickaxe,
        "iron_pickaxe" => ItemKind::IronPickaxe,
        "wood_sword" => ItemKind::WoodSword,
        "stone_sword" => ItemKind::StoneSword,
        "iron_sword" => ItemKind::IronSword,
        other => panic!("unknown item in fixture: {other}"),
    }
}
