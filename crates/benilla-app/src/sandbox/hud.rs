//! The on-screen help line and a coordinates readout. The client's player UI is off in sandbox
//! mode, so this is plain Bevy UI in the client's font, under the loading cover.

use bevy::prelude::*;

use crate::player::Player;

const HELP: &str = "W/S move   A/D turn   Q/E strafe   Space jump   Right-drag steer   \
                    Left-drag orbit   Wheel zoom   X sit   Z sheathe   M mute   F1 hide";

#[derive(Component)]
pub(super) struct HudRoot;

#[derive(Component)]
pub(super) struct Readout;

fn line(font: &Handle<Font>, text: &str, alpha: f32) -> impl Bundle {
    (
        Text::new(text),
        TextFont {
            font: font.clone(),
            font_size: 13.0,
            ..default()
        },
        TextColor(Color::srgba(1.0, 1.0, 1.0, alpha)),
        TextShadow {
            offset: Vec2::splat(1.0),
            color: Color::BLACK,
        },
    )
}

pub(super) fn setup(mut commands: Commands, assets: Res<AssetServer>) {
    let font = crate::char_select::wow_font(&assets);
    commands
        .spawn((
            HudRoot,
            Node {
                position_type: PositionType::Absolute,
                left: Val::Px(10.0),
                top: Val::Px(8.0),
                flex_direction: FlexDirection::Column,
                row_gap: Val::Px(2.0),
                ..default()
            },
            // Under the loading screen's 1000.
            GlobalZIndex(400),
        ))
        .with_children(|root| {
            root.spawn(line(&font, HELP, 0.8));
            root.spawn((Readout, line(&font, "", 0.8)));
        });
}

/// `F1` hides or shows the lines; the readout refreshes four times a second.
pub(super) fn update(
    keys: Res<ButtonInput<KeyCode>>,
    time: Res<Time<Real>>,
    player: Res<Player>,
    map: Option<Res<benilla_world::world_map::CurrentMap>>,
    mut root: Query<&mut Visibility, With<HudRoot>>,
    mut readout: Query<&mut Text, With<Readout>>,
    mut fps: Local<f32>,
    mut since: Local<f32>,
) {
    if keys.just_pressed(KeyCode::F1) {
        for mut v in &mut root {
            *v = match *v {
                Visibility::Hidden => Visibility::Inherited,
                _ => Visibility::Hidden,
            };
        }
    }
    let dt = time.delta_secs();
    if dt > 0.0 {
        *fps += (1.0 / dt - *fps) * 0.05;
    }
    *since += dt;
    if *since < 0.25 {
        return;
    }
    *since = 0.0;
    let wow = benilla_assets::coords::bevy_to_wow(player.pos);
    let text = format!(
        "map {}   x {:.1}   y {:.1}   z {:.1}   {:.0} fps",
        map.map_or(0, |m| m.0),
        wow[0],
        wow[1],
        wow[2],
        *fps
    );
    if let Ok(mut t) = readout.single_mut() {
        if **t != text {
            **t = text;
        }
    }
}
