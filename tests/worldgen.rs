use std::collections::HashSet;

use crafter_rs::{Env, EnvConfig, Material};

#[test]
fn worldgen_is_seeded_and_diverse() {
    let config = EnvConfig {
        seed: 7,
        ..EnvConfig::default()
    };
    let mut env_a = Env::new(config.clone());
    let mut env_b = Env::new(config);

    env_a.reset();
    env_b.reset();

    let materials_a: Vec<_> = env_a.world().iter_materials().collect();
    let materials_b: Vec<_> = env_b.world().iter_materials().collect();
    assert_eq!(materials_a, materials_b);

    let unique: HashSet<_> = materials_a.into_iter().flatten().collect();
    assert!(unique.contains(&Material::Grass));
    assert!(unique.contains(&Material::Water));
    assert!(unique.contains(&Material::Stone));
    assert!(unique.len() >= 5);

    let center = env_a.player_position();
    assert_eq!(env_a.world().material(center), Some(Material::Grass));
    for x in (center[0] - 2)..=(center[0] + 2) {
        for y in (center[1] - 2)..=(center[1] + 2) {
            assert!(env_a.world().material([x, y]).unwrap().is_walkable());
        }
    }
}
