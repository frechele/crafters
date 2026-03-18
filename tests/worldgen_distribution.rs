use crafter_rs::{Env, EnvConfig, Material};

#[test]
fn worldgen_has_meaningful_path_and_iron_density_on_reference_seed() {
    let mut env = Env::new(EnvConfig {
        seed: 0,
        ..EnvConfig::default()
    });
    env.reset();

    let mut path = 0usize;
    let mut iron = 0usize;
    let area = env.world().area();
    let path_id = Material::Path.id();
    let iron_id = Material::Iron.id();
    for x in 0..area[0] {
        for y in 0..area[1] {
            if let Some(mid) = env.world().material([x, y]) {
                if mid == path_id {
                    path += 1;
                } else if mid == iron_id {
                    iron += 1;
                }
            }
        }
    }

    assert!(
        path >= 200,
        "reference seed should produce enough path tiles for crafter parity, got {path}",
    );
    assert!(
        iron >= 8,
        "reference seed should produce enough iron tiles for crafter parity, got {iron}",
    );
}

#[test]
fn worldgen_seed_two_matches_python_reference_summary() {
    let mut env = Env::new(EnvConfig {
        seed: 2,
        ..EnvConfig::default()
    });
    env.reset();

    let semantic = env.semantic_view();
    let path = semantic.cells().iter().filter(|&&cell| cell == 4).count();
    let stone = semantic.cells().iter().filter(|&&cell| cell == 3).count();
    let coal = semantic.cells().iter().filter(|&&cell| cell == 8).count();
    let iron = semantic.cells().iter().filter(|&&cell| cell == 9).count();
    let skeletons = semantic.cells().iter().filter(|&&cell| cell == 16).count();
    let zombies = semantic.cells().iter().filter(|&&cell| cell == 15).count();
    let cows = semantic.cells().iter().filter(|&&cell| cell == 14).count();

    assert_eq!(path, 239);
    assert_eq!(stone, 476);
    assert_eq!(coal, 44);
    assert_eq!(iron, 17);
    assert_eq!(skeletons, 9);
    assert_eq!(zombies, 13);
    assert_eq!(cows, 24);
}
