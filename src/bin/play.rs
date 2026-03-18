use crafter_rs::{
    Env, EnvConfig, RunnerKey, runner_action_from_keys, runner_frame_to_buffer,
};
use minifb::{Key, KeyRepeat, Scale, Window, WindowOptions};

const WINDOW_SIZE: [usize; 2] = [576, 576];

fn main() -> Result<(), Box<dyn std::error::Error>> {
    print_controls();

    let mut env = Env::new(EnvConfig {
        size: WINDOW_SIZE,
        ..EnvConfig::default()
    });
    let mut frame = env.reset();
    let mut buffer = runner_frame_to_buffer(&frame);
    let mut done = false;

    let mut window = Window::new(
        "Crafter RS",
        frame.width(),
        frame.height(),
        WindowOptions {
            scale: Scale::X1,
            resize: false,
            ..WindowOptions::default()
        },
    )?;
    window.set_target_fps(60);

    while window.is_open() && !window.is_key_down(Key::Escape) {
        update_title(&mut window, &env, done);

        if window.is_key_pressed(Key::R, KeyRepeat::No) {
            frame = env.reset();
            buffer = runner_frame_to_buffer(&frame);
            done = false;
        } else if let Some(keys) = pressed_runner_keys(&window) {
            if done {
                frame = env.reset();
                done = false;
            } else {
                let action_name = runner_action_from_keys(&keys);
                let result = env.step_by_name(action_name).expect("unknown action");
                frame = result.observation;
                done = result.done;
            }
            buffer = runner_frame_to_buffer(&frame);
        }

        window.update_with_buffer(&buffer, frame.width(), frame.height())?;
    }

    Ok(())
}

fn pressed_runner_keys(window: &Window) -> Option<Vec<RunnerKey>> {
    let keys = window.get_keys_pressed(KeyRepeat::Yes);
    if keys.is_empty() {
        return None;
    }

    let mut mapped = Vec::new();
    for key in keys {
        if let Some(runner_key) = map_key(key) {
            mapped.push(runner_key);
        }
    }

    if mapped.is_empty() {
        if window.is_key_pressed(Key::N, KeyRepeat::Yes) {
            Some(Vec::new())
        } else {
            None
        }
    } else {
        Some(mapped)
    }
}

fn map_key(key: Key) -> Option<RunnerKey> {
    match key {
        Key::Left => Some(RunnerKey::Left),
        Key::Right => Some(RunnerKey::Right),
        Key::Up => Some(RunnerKey::Up),
        Key::Down => Some(RunnerKey::Down),
        Key::Space | Key::Enter => Some(RunnerKey::Do),
        Key::E => Some(RunnerKey::Sleep),
        Key::Key1 => Some(RunnerKey::PlaceStone),
        Key::Key2 => Some(RunnerKey::PlaceTable),
        Key::Key3 => Some(RunnerKey::PlaceFurnace),
        Key::Key4 => Some(RunnerKey::PlacePlant),
        Key::Z => Some(RunnerKey::MakeWoodPickaxe),
        Key::X => Some(RunnerKey::MakeStonePickaxe),
        Key::C => Some(RunnerKey::MakeIronPickaxe),
        Key::A => Some(RunnerKey::MakeWoodSword),
        Key::S => Some(RunnerKey::MakeStoneSword),
        Key::D => Some(RunnerKey::MakeIronSword),
        _ => None,
    }
}

fn update_title(window: &mut Window, env: &Env, done: bool) {
    let wk = &env.rules().well_known;
    let title = if done {
        format!(
            "Crafter RS | HP {} Food {} Drink {} Energy {} | episode over: press R or any action to reset",
            env.player().health(),
            env.player().item(wk.food),
            env.player().item(wk.drink),
            env.player().item(wk.energy),
        )
    } else {
        format!(
            "Crafter RS | HP {} Food {} Drink {} Energy {} | arrows move, space interact, E sleep, 1-4 place, Z/X/C pickaxe, A/S/D sword, N wait, R reset",
            env.player().health(),
            env.player().item(wk.food),
            env.player().item(wk.drink),
            env.player().item(wk.energy),
        )
    };
    window.set_title(&title);
}

fn print_controls() {
    println!("Crafter RS controls:");
    println!("  Arrow keys: move");
    println!("  Space or Enter: interact / mine / attack");
    println!("  E: sleep");
    println!("  1: place stone");
    println!("  2: place table");
    println!("  3: place furnace");
    println!("  4: place plant");
    println!("  Z/X/C: craft wood/stone/iron pickaxe");
    println!("  A/S/D: craft wood/stone/iron sword");
    println!("  N: wait one step");
    println!("  R: reset episode");
    println!("  Esc: quit");
}
