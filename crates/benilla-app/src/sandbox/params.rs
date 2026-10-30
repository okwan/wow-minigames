//! The sandbox's start-up parameters: where in the world the local character stands and who it is.
//!
//! Read once, before the app builds, from the page's query string (`?map=0&x=-8949.95&…`) in a
//! browser or from `WOW_SANDBOX_*` environment variables natively, both through
//! [`crate::webenv::var`]. The parsing is a pure function of a lookup closure ([`parse`]), so it
//! is unit-tested without either.

use bevy::prelude::Resource;

/// Races the 1.12 client plays, by `ChrRaces.dbc` id.
const RACES: &[(u8, &[&str])] = &[
    (1, &["human"]),
    (2, &["orc"]),
    (3, &["dwarf"]),
    (4, &["nightelf", "nelf"]),
    (5, &["undead", "scourge", "forsaken"]),
    (6, &["tauren"]),
    (7, &["gnome"]),
    (8, &["troll"]),
];

/// Classes by `ChrClasses.dbc` id (6 and 10 are 1.12's unused gaps).
const CLASSES: &[(u8, &str)] = &[
    (1, "warrior"),
    (2, "paladin"),
    (3, "hunter"),
    (4, "rogue"),
    (5, "priest"),
    (7, "shaman"),
    (8, "mage"),
    (9, "warlock"),
    (11, "druid"),
];

/// Everything a sandbox start can be told. It is also the resource the plugin inserts, for a game
/// that wants to know where the character started. WoW's own conventions throughout: `x` and `y` are the
/// world's horizontal axes (`x` north, `y` west), `z` is up, `facing` is the orientation in
/// radians (0 faces north, +x; counter-clockwise from above).
#[derive(Resource, Clone, Debug, PartialEq)]
pub struct SandboxParams {
    /// `Map.dbc` id: 0 Eastern Kingdoms, 1 Kalimdor.
    pub map: u32,
    /// `[x, y, z]` in WoW coordinates.
    pub position: [f32; 3],
    /// Stand on the terrain at `x`, `y` instead of at `z`: set when a start gives a place but no
    /// height, which nobody knows by heart. The character is put a little above the ground once
    /// the terrain there has loaded.
    pub snap_to_ground: bool,
    /// Orientation in radians.
    pub facing: f32,
    /// `ChrRaces.dbc` id.
    pub race: u8,
    /// 0 male, 1 female.
    pub gender: u8,
    /// `ChrClasses.dbc` id.
    pub class: u8,
    /// Minute of the day the light is pinned to (0..1440); there is no server clock to follow.
    pub minute: u32,
    /// Silence all audio. On by default: a muted sandbox opens no audio device and decodes no
    /// sound; `?mute=0` (or `M` in game) turns it on.
    pub mute: bool,
}

impl Default for SandboxParams {
    /// A Human Mage at the Northshire Abbey, Elwynn Forest: the 1.12 human start
    /// (`playercreateinfo`: map 0, -8949.95, -132.493, 83.5312), a yard above the ground so the
    /// body settles onto it instead of starting inside it.
    fn default() -> Self {
        Self {
            map: 0,
            position: [-8949.95, -132.493, 84.5],
            snap_to_ground: false,
            facing: 0.0,
            race: 1,
            gender: 0,
            class: 8,
            minute: 13 * 60,
            mute: true,
        }
    }
}

impl SandboxParams {
    /// The parameters this process was started with, problems logged and defaulted.
    pub fn from_env() -> Self {
        let (params, problems) = parse(env_param);
        for p in problems {
            eprintln!("sandbox: {p}");
        }
        params
    }
}

/// One parameter by short name (`map`, `x`, …). `WOW_SANDBOX_<NAME>` natively and as
/// `?sandbox_<name>=` in a browser; a browser also takes the bare `?<name>=` the docs show, and
/// natively `WOW_MAP` (the engine's own server-less map variable) still answers for the map.
fn env_param(name: &str) -> Option<String> {
    let upper = name.to_ascii_uppercase();
    crate::webenv::var(&format!("WOW_SANDBOX_{upper}")).or_else(|| {
        (cfg!(target_arch = "wasm32") || name == "map")
            .then(|| crate::webenv::var(&format!("WOW_{upper}")))
            .flatten()
    })
}

/// Parse the parameters out of `get`, a lookup by short name. Returns the parameters, with the
/// default standing for every value that is absent or unreadable, and one message per unreadable
/// value.
pub(crate) fn parse(get: impl Fn(&str) -> Option<String>) -> (SandboxParams, Vec<String>) {
    let mut out = SandboxParams::default();
    let mut problems = Vec::new();
    let mut take = |name: &str, apply: &mut dyn FnMut(&str) -> Option<()>| {
        let Some(raw) = get(name) else { return };
        let raw = raw.trim();
        if !raw.is_empty() && apply(raw).is_none() {
            problems.push(format!("ignoring {name}={raw:?}: not understood"));
        }
    };
    take("map", &mut |s| {
        out.map = parse_map(s)?;
        Some(())
    });
    let (mut xy_given, mut z_given) = (false, false);
    for (i, axis) in ["x", "y", "z"].into_iter().enumerate() {
        take(axis, &mut |s| {
            out.position[i] = finite(s.parse().ok()?)?;
            if i == 2 {
                z_given = true;
            } else {
                xy_given = true;
            }
            Some(())
        });
    }
    take("facing", &mut |s| {
        out.facing = parse_angle(s)?;
        Some(())
    });
    take("race", &mut |s| {
        out.race = parse_race(s)?;
        Some(())
    });
    take("gender", &mut |s| {
        out.gender = parse_gender(s)?;
        Some(())
    });
    take("class", &mut |s| {
        out.class = parse_class(s)?;
        Some(())
    });
    take("time", &mut |s| {
        out.minute = parse_time(s)?;
        Some(())
    });
    take("mute", &mut |s| {
        out.mute = parse_bool(s)?;
        Some(())
    });
    out.snap_to_ground = xy_given && !z_given;
    (out, problems)
}

fn finite(v: f32) -> Option<f32> {
    v.is_finite().then_some(v)
}

/// A `Map.dbc` id, or `eastern`/`azeroth` (0) and `kalimdor` (1).
pub(crate) fn parse_map(s: &str) -> Option<u32> {
    let l = s.to_ascii_lowercase();
    match l.as_str() {
        "eastern" | "easternkingdoms" | "azeroth" | "ek" => Some(0),
        "kalimdor" => Some(1),
        _ => l.parse().ok(),
    }
}

/// Radians, or degrees with a `deg` suffix (`90deg`).
pub(crate) fn parse_angle(s: &str) -> Option<f32> {
    let l = s.to_ascii_lowercase();
    match l.strip_suffix("deg") {
        Some(deg) => finite(deg.trim().parse::<f32>().ok()?.to_radians()),
        None => finite(l.parse().ok()?),
    }
}

/// A race id 1..=8 or its name; spaces, dashes and underscores in the name are ignored.
pub(crate) fn parse_race(s: &str) -> Option<u8> {
    let key: String = s
        .chars()
        .filter(|c| !matches!(c, ' ' | '-' | '_'))
        .collect::<String>()
        .to_ascii_lowercase();
    if let Ok(id) = key.parse::<u8>() {
        return RACES.iter().any(|(r, _)| *r == id).then_some(id);
    }
    RACES
        .iter()
        .find(|(_, names)| names.contains(&key.as_str()))
        .map(|(id, _)| *id)
}

/// A class id or its name.
pub(crate) fn parse_class(s: &str) -> Option<u8> {
    let key = s.to_ascii_lowercase();
    if let Ok(id) = key.parse::<u8>() {
        return CLASSES.iter().any(|(c, _)| *c == id).then_some(id);
    }
    CLASSES.iter().find(|(_, n)| *n == key).map(|(id, _)| *id)
}

/// `male`/`m`/`0` or `female`/`f`/`1`.
pub(crate) fn parse_gender(s: &str) -> Option<u8> {
    match s.to_ascii_lowercase().as_str() {
        "male" | "m" | "0" => Some(0),
        "female" | "f" | "1" => Some(1),
        _ => None,
    }
}

/// `1`/`true`/`on`/`yes` or `0`/`false`/`off`/`no`.
pub(crate) fn parse_bool(s: &str) -> Option<bool> {
    match s.to_ascii_lowercase().as_str() {
        "1" | "true" | "on" | "yes" => Some(true),
        "0" | "false" | "off" | "no" => Some(false),
        _ => None,
    }
}

/// `HH:MM`, 24 hours, as the minute of the day.
pub(crate) fn parse_time(s: &str) -> Option<u32> {
    let (h, m) = s.split_once(':')?;
    let (h, m) = (h.trim().parse::<u32>().ok()?, m.trim().parse::<u32>().ok()?);
    (h < 24 && m < 60).then_some(h * 60 + m)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn from(pairs: &[(&str, &str)]) -> (SandboxParams, Vec<String>) {
        parse(|k| {
            pairs
                .iter()
                .find(|(n, _)| *n == k)
                .map(|(_, v)| (*v).to_string())
        })
    }

    #[test]
    fn nothing_given_is_a_human_mage_at_northshire() {
        let (p, problems) = from(&[]);
        assert!(problems.is_empty());
        assert_eq!(p, SandboxParams::default());
        assert_eq!((p.map, p.race, p.gender, p.class), (0, 1, 0, 8));
        assert!((p.position[0] - -8949.95).abs() < 1e-3);
    }

    #[test]
    fn every_key_is_read() {
        let (p, problems) = from(&[
            ("map", "1"),
            ("x", "-1000.5"),
            ("y", "2000"),
            ("z", "30"),
            ("facing", "1.5"),
            ("race", "Night Elf"),
            ("gender", "female"),
            ("class", "druid"),
            ("time", "06:30"),
        ]);
        assert!(problems.is_empty(), "{problems:?}");
        assert_eq!(p.map, 1);
        assert_eq!(p.position, [-1000.5, 2000.0, 30.0]);
        assert_eq!(p.facing, 1.5);
        assert_eq!((p.race, p.gender, p.class), (4, 1, 11));
        assert_eq!(p.minute, 6 * 60 + 30);
    }

    #[test]
    fn a_place_without_a_height_stands_on_the_ground() {
        let (p, _) = from(&[("map", "1"), ("x", "-950"), ("y", "-2700")]);
        assert!(p.snap_to_ground);
        let (p, _) = from(&[("x", "-950"), ("y", "-2700"), ("z", "92")]);
        assert!(!p.snap_to_ground);
        // A height that does not parse is not a height.
        let (p, problems) = from(&[("x", "-950"), ("y", "-2700"), ("z", "high")]);
        assert!(p.snap_to_ground && problems.len() == 1);
        // Nothing about the place: the default's known height stands.
        let (p, _) = from(&[("map", "0")]);
        assert!(!p.snap_to_ground);
    }

    #[test]
    fn a_bad_value_keeps_its_default_and_is_reported_once() {
        let (p, problems) = from(&[
            ("x", "north"),
            ("race", "murloc"),
            ("class", "6"),
            ("z", "NaN"),
        ]);
        assert_eq!(p, SandboxParams::default());
        assert_eq!(problems.len(), 4, "{problems:?}");
    }

    #[test]
    fn blank_values_are_absent_not_wrong() {
        let (p, problems) = from(&[("map", ""), ("race", "   ")]);
        assert_eq!(p, SandboxParams::default());
        assert!(problems.is_empty());
    }

    #[test]
    fn names_and_numbers_both_work() {
        assert_eq!(parse_race("5"), Some(5));
        assert_eq!(parse_race("night-elf"), Some(4));
        assert_eq!(parse_race("NIGHT_ELF"), Some(4));
        assert_eq!(parse_race("Undead"), Some(5));
        assert_eq!(parse_race("9"), None);
        assert_eq!(parse_class("Warlock"), Some(9));
        assert_eq!(parse_class("11"), Some(11));
        assert_eq!(parse_class("10"), None);
        assert_eq!(parse_gender("F"), Some(1));
        assert_eq!(parse_gender("2"), None);
        assert_eq!(parse_map("Kalimdor"), Some(1));
        assert_eq!(parse_map("eastern"), Some(0));
        assert_eq!(parse_map("30"), Some(30));
        assert_eq!(parse_map("-1"), None);
    }

    #[test]
    fn angles_take_degrees() {
        assert_eq!(parse_angle("0"), Some(0.0));
        let right = parse_angle("90deg").unwrap();
        assert!((right - std::f32::consts::FRAC_PI_2).abs() < 1e-6);
        assert_eq!(parse_angle("inf"), None);
    }

    #[test]
    fn sound_is_muted_unless_asked_for() {
        let (p, problems) = from(&[]);
        assert!(p.mute && problems.is_empty());
        let (p, problems) = from(&[("mute", "0")]);
        assert!(!p.mute && problems.is_empty());
        assert!(from(&[("mute", "1")]).0.mute);
        assert!(!from(&[("mute", "off")]).0.mute);
        // Unreadable: stays muted, and says so.
        let (p, problems) = from(&[("mute", "maybe")]);
        assert!(p.mute && problems.len() == 1);
    }

    #[test]
    fn the_time_of_day_is_bounded() {
        assert_eq!(parse_time("00:00"), Some(0));
        assert_eq!(parse_time("23:59"), Some(1439));
        assert_eq!(parse_time("24:00"), None);
        assert_eq!(parse_time("12:60"), None);
        assert_eq!(parse_time("noon"), None);
    }
}
