use crafter_rs::{Action, Direction, Env, ItemKind, Material};

#[test]
fn player_can_move_collect_place_and_craft() {
    let mut env = Env::default();
    env.reset();

    env.world_mut().fill(Material::Grass);
    let origin = env.player_position();

    let step = env.step(Action::MoveRight);
    assert_eq!(env.player_position(), [origin[0] + 1, origin[1]]);
    assert_eq!(env.player().facing(), Direction::Right);
    assert!(!step.done);

    env.player_mut().set_facing(Direction::Right);
    let front = [env.player_position()[0] + 1, env.player_position()[1]];
    env.world_mut().set_material(front, Material::Tree);
    let wood_before = env.player().item(ItemKind::Wood);
    let collect_tree = env.step(Action::Do);
    assert_eq!(env.player().item(ItemKind::Wood), wood_before + 1);
    assert_eq!(env.world().material(front), Some(Material::Grass));
    assert!(collect_tree.info.achievements.unlocked("collect_wood"));

    env.player_mut().set_facing(Direction::Right);
    let front = [env.player_position()[0] + 1, env.player_position()[1]];
    env.world_mut().set_material(front, Material::Stone);
    env.step(Action::Do);
    assert_eq!(env.player().item(ItemKind::Stone), 0);

    env.player_mut().set_item(ItemKind::WoodPickaxe, 1);
    env.player_mut().set_facing(Direction::Right);
    env.step(Action::Do);
    assert_eq!(env.player().item(ItemKind::Stone), 1);
    assert_eq!(env.world().material(front), Some(Material::Path));

    env.player_mut().set_item(ItemKind::Wood, 3);
    env.player_mut().set_facing(Direction::Right);
    let front = [env.player_position()[0] + 1, env.player_position()[1]];
    env.world_mut().set_material(front, Material::Grass);
    env.step(Action::PlaceTable);
    assert_eq!(env.world().material(front), Some(Material::Table));
    assert_eq!(env.player().item(ItemKind::Wood), 1);

    env.player_mut().set_item(ItemKind::Wood, 2);
    env.step(Action::MakeWoodPickaxe);
    assert_eq!(env.player().item(ItemKind::WoodPickaxe), 2);
    assert_eq!(env.player().item(ItemKind::Wood), 1);
}

#[test]
fn survival_needs_can_reduce_and_restore_health() {
    let mut env = Env::default();
    env.reset();
    env.world_mut().fill(Material::Grass);

    env.player_mut().set_health(8);
    env.player_mut().set_item(ItemKind::Food, 0);
    env.player_mut().set_item(ItemKind::Drink, 0);
    env.player_mut().set_item(ItemKind::Energy, 0);
    for _ in 0..40 {
        env.step(Action::Noop);
    }
    assert!(env.player().health() < 8);

    env.player_mut().set_health(5);
    env.player_mut().set_item(ItemKind::Food, 9);
    env.player_mut().set_item(ItemKind::Drink, 9);
    env.player_mut().set_item(ItemKind::Energy, 1);
    for _ in 0..40 {
        env.step(Action::Sleep);
    }
    assert!(env.player().health() > 5);
}
