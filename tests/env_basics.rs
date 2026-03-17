use crafter_rs::{Action, Env, ItemKind, Material};

#[test]
fn env_reset_initializes_default_spaces() {
    let mut env = Env::default();
    let obs = env.reset();

    assert_eq!(env.action_names().len(), 17);
    assert_eq!(obs.width(), 64);
    assert_eq!(obs.height(), 64);
    assert_eq!(obs.channels(), 3);

    let player = env.player();
    assert_eq!(player.item(ItemKind::Health), 9);
    assert_eq!(player.item(ItemKind::Food), 9);
    assert_eq!(player.item(ItemKind::Drink), 9);
    assert_eq!(player.item(ItemKind::Energy), 9);
    assert_eq!(player.item(ItemKind::Wood), 0);
    assert_eq!(player.item(ItemKind::Stone), 0);
    assert_eq!(player.item(ItemKind::WoodPickaxe), 0);
}

#[test]
fn env_reset_builds_world_state() {
    let mut env = Env::default();
    env.reset();

    assert_eq!(env.player_position(), [32, 32]);
    assert_eq!(env.world().area(), [64, 64]);
    assert_eq!(env.world().material([32, 32]), Some(Material::Grass));
    assert!(
        env.world()
            .iter_materials()
            .all(|material| material.is_some())
    );

    let semantic = env.semantic_view();
    assert_eq!(semantic.width(), 64);
    assert_eq!(semantic.height(), 64);
    assert_eq!(semantic.cells().len(), 64 * 64);
}

#[test]
fn env_accepts_action_indices() {
    let mut env = Env::default();
    env.reset();
    env.world_mut().fill(Material::Grass);

    let origin = env.player_position();
    let result = env.step_index(Action::MoveRight as usize);
    assert!(result.is_some());
    assert_eq!(env.player_position(), [origin[0] + 1, origin[1]]);
}
