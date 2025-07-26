use std::f32::consts::{PI, TAU};

use bevy::{
    input::mouse::{MouseMotion, MouseScrollUnit, MouseWheel},
    prelude::*,
};

pub struct CameraPlugin;

#[derive(Component, Debug, Clone, Copy, PartialEq)]
pub struct ManagedCamera {
    pub radius: f32,
    pub pitch: f32,
    pub yaw: f32,
}

impl Default for ManagedCamera {
    fn default() -> Self {
        Self {
            radius: 10.0,
            pitch: 0.0,
            yaw: 0.0,
        }
    }
}

pub fn camera_move(
    cameras: Single<(&mut ManagedCamera, &mut Transform)>,
    buttons: Res<ButtonInput<MouseButton>>,
    mut evr_scroll: EventReader<MouseWheel>,
    mut evr_motion: EventReader<MouseMotion>,
) {
    let (mut camera, mut transform) = cameras.into_inner();

    let zoom: f32 = evr_scroll
        .read()
        .map(|ev| match ev.unit {
            MouseScrollUnit::Line => ev.y * 0.1,
            MouseScrollUnit::Pixel => ev.y * 0.01,
        })
        .sum();

    camera.radius *= (-zoom).exp();

    if buttons.pressed(MouseButton::Right) {
        let scroll: Vec2 = evr_motion.read().map(|ev| ev.delta).sum();
        camera.yaw += scroll.x * 0.01;
        camera.pitch += scroll.y * 0.01;

        if camera.yaw > PI {
            camera.yaw -= TAU;
        }
        if camera.yaw < -PI {
            camera.yaw += TAU;
        }

        camera.pitch = camera.pitch.clamp(-PI / 2.2, PI / 2.2);
    }

    transform.translation = Vec3::new(
        camera.yaw.cos() * camera.pitch.cos(),
        camera.pitch.sin(),
        camera.yaw.sin() * camera.pitch.cos(),
    ) * camera.radius;

    transform.look_at(Vec3::ZERO, Vec3::Y);
}

impl Plugin for CameraPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, camera_move);
    }
}
