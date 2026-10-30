//! Local units, built the way the net layer builds streamed ones.
//!
//! There is no server, so a unit exists because a system spawned it: a [`Guid`], a [`NetEntity`]
//! and an [`ObjectStore`] holding the descriptor fields are all the attach, animation, equipment
//! and spell-visual systems need to dress it, animate it and show its weapons. [`spawn_unit`] is
//! that recipe; a game spawns its creatures, NPCs and extra characters through it.

use benilla_protocol::messages::ObjectFields;
use bevy::prelude::*;

use super::looks::Weapon;
use crate::creature_anim::MovementState;
use crate::net::{Guid, NetEntity, ObjectStore, ScriptedFacing};

// Descriptor field indices (`UpdateFields.h`, 1.12.1) for the fields a unit's look reads.
const FIELD_HEALTH: u16 = 22;
const FIELD_MAX_HEALTH: u16 = 28;
const FIELD_LEVEL: u16 = 34;
const FIELD_FACTION: u16 = 35;
const FIELD_BYTES_0: u16 = 36;
const FIELD_VIRTUAL_ITEM_DISPLAY: u16 = 37;
const FIELD_VIRTUAL_ITEM_INFO: u16 = 40;
const FIELD_BYTES_2: u16 = 164;

/// `FactionTemplate.dbc`: friendly to players, and the hostile monster template.
pub const FACTION_FRIENDLY: u32 = 35;
pub const FACTION_MONSTER: u32 = 14;

/// The high word of a creature guid (`HIGHGUID_UNIT`).
const HIGH_CREATURE: u64 = 0xF130 << 48;

/// The guid [`spawn_unit`] gives the unit with this `serial`. Serials are the caller's to keep
/// unique; the sandbox player takes serial 1.
pub const fn creature_guid(serial: u64) -> u64 {
    HIGH_CREATURE | serial
}

/// Everything that makes a unit look like itself.
pub struct UnitSpec<'a> {
    /// Unique among local units; becomes the guid ([`creature_guid`]).
    pub serial: u64,
    /// `CreatureDisplayInfo.dbc` id: the model, skin and, for a humanoid, face and clothes.
    pub display: u32,
    /// Model scale, 1.0 for the native size.
    pub scale: f32,
    pub level: u32,
    /// A `FactionTemplate.dbc` id ([`FACTION_FRIENDLY`], [`FACTION_MONSTER`]).
    pub faction: u32,
    /// `UNIT_FIELD_BYTES_0`: race | class << 8 | gender << 16 | power type << 24 ([`bytes0`]).
    pub bytes0: u32,
    /// Weapons held in the virtual item slots.
    pub weapons: &'a [Weapon],
    /// `UNIT_FIELD_BYTES_2` byte 0: 0 stowed, 1 melee drawn, 2 ranged drawn.
    pub sheath: u8,
}

/// Pack `UNIT_FIELD_BYTES_0`.
pub fn bytes0(race: u8, class: u8, gender: u8, power: u8) -> u32 {
    u32::from(race) | u32::from(class) << 8 | u32::from(gender) << 16 | u32::from(power) << 24
}

/// Marks every entity the sandbox itself spawned, the player included, so a game can find or
/// clear them.
#[derive(Component)]
pub struct SandboxEntity;

/// Spawn a unit the attach builds a body for at `pos` (Bevy coordinates; see
/// [`wow_to_bevy`](crate::sandbox::wow_to_bevy)) facing `yaw` (radians; equal to the WoW
/// orientation). The caller adds its own components to the returned entity.
///
/// The unit stands where it is put: nothing re-seats a local unit on the ground, so a game that
/// wants it grounded sets `translation.y` from [`WorldPoint::terrain_height_under`](crate::sandbox::WorldPoint).
pub fn spawn_unit(commands: &mut Commands, spec: &UnitSpec, pos: Vec3, yaw: f32) -> Entity {
    let guid = creature_guid(spec.serial);
    let mut pairs: Vec<(u16, u32)> = vec![
        (0, guid as u32),
        (1, (guid >> 32) as u32),
        // OBJECT_FIELD_TYPE: object | unit.
        (2, 0x9),
        (4, spec.scale.to_bits()),
        (FIELD_HEALTH, 100),
        (FIELD_MAX_HEALTH, 100),
        (FIELD_LEVEL, spec.level),
        (FIELD_FACTION, spec.faction),
        (FIELD_BYTES_0, spec.bytes0),
        (FIELD_BYTES_2, u32::from(spec.sheath)),
    ];
    for &(slot, display, class, sub, material, inv, sheath) in spec.weapons {
        pairs.push((FIELD_VIRTUAL_ITEM_DISPLAY + slot, display));
        pairs.push((
            FIELD_VIRTUAL_ITEM_INFO + 2 * slot,
            u32::from(class)
                | u32::from(sub) << 8
                | u32::from(material) << 16
                | u32::from(inv) << 24,
        ));
        pairs.push((FIELD_VIRTUAL_ITEM_INFO + 2 * slot + 1, u32::from(sheath)));
    }
    commands
        .spawn((
            Guid(guid),
            NetEntity {
                kind: benilla_protocol::EntityKind::Unit,
                display_id: Some(spec.display),
                scale: spec.scale,
            },
            ObjectStore(ObjectFields::from_pairs(&pairs)),
            MovementState::default(),
            // No wire facing exists to swing a local unit back to: its owner writes the rotation.
            ScriptedFacing,
            Transform::from_translation(pos).with_rotation(Quat::from_rotation_y(yaw)),
            Visibility::default(),
            SandboxEntity,
        ))
        .id()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bytes0_packs_race_class_gender_power() {
        assert_eq!(bytes0(1, 8, 0, 0), 0x0000_0801);
        assert_eq!(bytes0(4, 11, 1, 3), 0x0301_0B04);
    }

    #[test]
    fn a_creature_guid_carries_its_high_word_and_serial() {
        assert_eq!(creature_guid(1), 0xF130_0000_0000_0001);
        assert_eq!(creature_guid(7) >> 48, 0xF130);
    }
}
