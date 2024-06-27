use std::f32::consts::PI;

use bevy::app::{App, Plugin, Update};
use bevy::input::ButtonInput;
use bevy::input::mouse::MouseMotion;
use bevy::math::{Quat, Vec3};
use bevy::prelude::{Component, EventReader, KeyCode, MouseButton, Query, Res, Time, Transform, Window, With};
use bevy::window::CursorGrabMode;

use crate::physics::Velocity;

#[derive(Debug, Component)]
pub struct Player;

#[derive(Default, Debug, Component)]
pub struct CameraRotation {
    pub pitch: f32,
    pub yaw: f32,
}

pub struct PlayerControllerPlugin;

impl Plugin for PlayerControllerPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, (grab_mouse, rotate_camera, move_player));
    }
}

fn rotate_camera(
    mut windows: Query<&mut Window>,
    mut mouse_motion: EventReader<MouseMotion>,
    mut rotation: Query<&mut CameraRotation, With<Player>>,
    mut transform: Query<&mut Transform, With<Player>>,
) {
    let window = windows.single_mut();
    if window.cursor.visible {
        return;
    }

    let mut rotation = rotation.single_mut();
    let mut transform = transform.single_mut();
    for motion in mouse_motion.read() {
        const SENSITIVITY: f32 = 4.0;
        let yaw = motion.delta.x * 0.002 * SENSITIVITY;
        let pitch = motion.delta.y * 0.002 * SENSITIVITY;
        rotation.yaw -= yaw;
        rotation.pitch -= pitch;
    }

    rotation.pitch = num::clamp(rotation.pitch, -PI / 2.0, PI / 2.0);

    // Order of rotations is important, see <https://gamedev.stackexchange.com/a/136175/103059s
    transform.rotation = Quat::default();
    transform.rotate_y(rotation.yaw);
    transform.rotate_local_x(rotation.pitch);
}

fn grab_mouse(
    mut windows: Query<&mut Window>,
    mouse: Res<ButtonInput<MouseButton>>,
    key: Res<ButtonInput<KeyCode>>,
) {
    let mut window = windows.single_mut();

    if mouse.just_pressed(MouseButton::Left) {
        window.cursor.visible = false;
        window.cursor.grab_mode = CursorGrabMode::Locked;
    }

    if key.just_pressed(KeyCode::Escape) {
        window.cursor.visible = true;
        window.cursor.grab_mode = CursorGrabMode::None;
    }
}

fn move_player(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
    mut camera_transform: Query<(&CameraRotation, &mut Velocity), With<Player>>,
) {
    let mut dx: f32 = 0.0;
    let mut dz: f32 = 0.0;

    if keyboard_input.pressed(KeyCode::KeyW) {
        dz -= 1.0;
    }
    if keyboard_input.pressed(KeyCode::KeyS) {
        dz += 1.0;
    }
    if keyboard_input.pressed(KeyCode::KeyA) {
        dx -= 1.0;
    }
    if keyboard_input.pressed(KeyCode::KeyD) {
        dx += 1.0;
    }

    let (rotation, mut velocity) = camera_transform.single_mut();
    let delta = time.delta().as_millis() as f32 / 1_000.0;
    if keyboard_input.just_pressed(KeyCode::Space) {
        velocity.0.y = 4.0 * delta;
    }

    let quat = Quat::from_rotation_y(rotation.yaw);
    velocity.0 += quat.mul_vec3(Vec3::new(dx, 0.0, dz).normalize_or_zero() * 4.0 * delta);
}