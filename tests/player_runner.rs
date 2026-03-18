use crafter_rs::{Frame, RunnerKey, runner_action_from_keys, runner_frame_to_buffer};

#[test]
fn runner_maps_keys_to_single_action() {
    assert_eq!(runner_action_from_keys(&[]), "noop");
    assert_eq!(
        runner_action_from_keys(&[RunnerKey::Left]),
        "move_left"
    );
    assert_eq!(
        runner_action_from_keys(&[RunnerKey::Right]),
        "move_right"
    );
    assert_eq!(
        runner_action_from_keys(&[RunnerKey::Left, RunnerKey::Do]),
        "do"
    );
    assert_eq!(
        runner_action_from_keys(&[RunnerKey::Down, RunnerKey::PlaceTable]),
        "place_table"
    );
}

#[test]
fn runner_converts_rgb_frames_to_window_buffer() {
    let frame = Frame::new(2, 1, 3, vec![255, 0, 0, 0, 128, 255]);
    let buffer = runner_frame_to_buffer(&frame);
    assert_eq!(buffer, vec![0x00FF0000, 0x000080FF]);
}
