use crafter_rs::{Action, Direction, Env, EnvConfig, Material};

#[test]
fn achievements_affect_reward_once_and_length_can_end_episode() {
    let mut env = Env::new(EnvConfig {
        length: Some(2),
        ..EnvConfig::default()
    });
    env.reset();
    env.world_mut().fill(Material::Grass);

    env.player_mut().set_facing(Direction::Right);
    let front = [env.player_position()[0] + 1, env.player_position()[1]];
    env.world_mut().set_material(front, Material::Tree);
    let first = env.step(Action::Do);
    assert!(first.reward >= 1.0);
    assert!(first.info.achievements.unlocked("collect_wood"));

    env.player_mut().set_facing(Direction::Right);
    env.world_mut().set_material(front, Material::Tree);
    let second = env.step(Action::Do);
    assert!(second.reward < 1.0);
    assert!(second.done);
}

#[test]
fn zombie_and_skeleton_updates_affect_player_and_world() {
    let mut env = Env::default();
    env.reset();
    env.world_mut().fill(Material::Grass);
    env.world_mut().clear_objects();

    let player = env.player_position();
    env.world_mut().spawn_zombie([player[0] + 1, player[1]]);
    let health_before = env.player().health();
    let zombie_step = env.step(Action::Noop);
    assert!(env.player().health() < health_before);
    assert!(zombie_step.reward < 0.0);

    env.world_mut().clear_objects();
    env.world_mut().spawn_skeleton([player[0] + 4, player[1]]);
    env.step(Action::Noop);
    assert!(env.world().arrow_count() > 0);
}

#[test]
fn lethal_damage_marks_done_and_zero_discount() {
    let mut env = Env::default();
    env.reset();
    env.world_mut().fill(Material::Grass);
    env.world_mut().clear_objects();

    let player = env.player_position();
    env.player_mut().set_health(1);
    env.world_mut().spawn_zombie([player[0] + 1, player[1]]);
    let result = env.step(Action::Noop);
    assert!(result.done);
    assert_eq!(result.info.discount, 0.0);
}
