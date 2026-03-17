use crafter_rs::{Action, Frame, RunnerKey, runner_action_from_keys, runner_frame_to_buffer};

#[test]
fn runner_maps_keys_to_single_action() {
    assert_eq!(runner_action_from_keys(&[]), Action::Noop);
    assert_eq!(
        runner_action_from_keys(&[RunnerKey::Left]),
        Action::MoveLeft
    );
    assert_eq!(
        runner_action_from_keys(&[RunnerKey::Right]),
        Action::MoveRight
    );
    assert_eq!(
        runner_action_from_keys(&[RunnerKey::Left, RunnerKey::Do]),
        Action::Do
    );
    assert_eq!(
        runner_action_from_keys(&[RunnerKey::Down, RunnerKey::PlaceTable]),
        Action::PlaceTable
    );
}

#[test]
fn runner_converts_rgb_frames_to_window_buffer() {
    let frame = Frame::new(2, 1, 3, vec![255, 0, 0, 0, 128, 255]);
    let buffer = runner_frame_to_buffer(&frame);
    assert_eq!(buffer, vec![0x00FF0000, 0x000080FF]);
}
