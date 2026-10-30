//! What a character of a given race, gender and class looks like, with no server to say.
//!
//! A server describes a player to the client as a display id plus equipment. Offline there is
//! neither, so the character is dressed the way a class trainer is: a `CreatureDisplayInfo` row
//! whose `CreatureDisplayInfoExtra` names a race, a sex, a face and the robes or armour of the
//! class. Those rows ride the same attach and animation systems every streamed creature does.
//!
//! The tables are the class trainers' displays and the weapons a trainer of the class holds.
//! Which race and sex a display wears is only known once the creature catalog has loaded, so
//! [`choose`] takes that lookup as a closure.

/// A weapon in a unit's virtual item slots: `(slot, display, class, subclass, material, inv_type,
/// sheath)`, the creature form of `UNIT_VIRTUAL_ITEM_SLOT_DISPLAY` and `_INFO`.
pub type Weapon = (u16, u32, u8, u8, u8, u8, u8);

/// One class's look.
pub(crate) struct ClassLook {
    pub id: u8,
    /// `UNIT_FIELD_BYTES_0` byte 3: 0 mana, 1 rage, 3 energy.
    pub power: u8,
    /// Class-trainer `CreatureDisplayInfo` ids, one or more per race and sex the class comes in.
    pub trainers: &'static [u32],
    pub weapons: &'static [Weapon],
    /// `UNIT_FIELD_BYTES_2` byte 0: 0 stowed, 1 melee drawn, 2 ranged drawn.
    pub sheath: u8,
}

pub(crate) static CLASS_LOOKS: &[ClassLook] = &[
    ClassLook {
        id: 1,
        power: 1,
        trainers: &[
            1300, 1374, 1375, 1504, 1578, 1599, 1707, 1721, 1880, 2096, 2103, 2113, 2196, 2198,
            2614, 2620, 2658, 3053, 3054, 3055, 3280, 3287, 3343, 3399, 3431, 3743, 3793, 3794,
            4242, 4556, 6071, 11037,
        ],
        weapons: &[(0, 7483, 2, 7, 1, 13, 3), (1, 1706, 4, 6, 1, 14, 4)],
        sheath: 1,
    },
    ClassLook {
        id: 2,
        power: 0,
        trainers: &[
            1299, 1499, 1622, 3087, 3088, 3089, 3284, 3289, 3346, 3393, 7356,
        ],
        weapons: &[(0, 8011, 2, 4, 2, 13, 3), (1, 1705, 4, 6, 1, 14, 4)],
        sheath: 1,
    },
    ClassLook {
        id: 3,
        power: 0,
        trainers: &[
            1373, 1703, 1723, 1882, 2066, 2087, 2105, 2112, 2205, 2206, 2251, 3056, 3072, 3073,
            3299, 3309, 3310, 3312, 3395, 3558, 3744, 3810, 3811, 4239, 4241, 4372, 4560, 7538,
            10245,
        ],
        weapons: &[(0, 7480, 2, 6, 1, 17, 2), (2, 6235, 2, 2, 2, 15, 0)],
        sheath: 2,
    },
    ClassLook {
        id: 4,
        power: 3,
        trainers: &[
            1297, 1327, 1328, 1507, 1580, 1603, 1704, 1725, 1886, 2231, 2243, 2252, 2631, 2639,
            2659, 3100, 3101, 3113, 3351, 3407, 3436, 3749, 4360, 5146, 5528, 13171,
        ],
        weapons: &[(0, 7492, 2, 7, 1, 13, 3), (1, 6443, 2, 15, 1, 13, 3)],
        sheath: 1,
    },
    ClassLook {
        id: 5,
        power: 0,
        trainers: &[
            1295, 1495, 1579, 1602, 1708, 1733, 1897, 2137, 2138, 2139, 2200, 2201, 2202, 2618,
            2626, 3066, 3085, 3086, 3282, 3283, 3344, 3401, 3429, 4068, 4690, 4711, 10473, 10723,
            11044, 11048, 11053,
        ],
        weapons: &[(0, 1926, 2, 10, 2, 17, 2)],
        sheath: 1,
    },
    ClassLook {
        id: 7,
        power: 0,
        trainers: &[
            1360, 1878, 2082, 2102, 2123, 3746, 3816, 4231, 4552, 10180, 13341,
        ],
        weapons: &[(0, 7477, 2, 4, 2, 13, 3), (1, 1705, 4, 6, 1, 14, 4)],
        sheath: 1,
    },
    ClassLook {
        id: 8,
        power: 0,
        trainers: &[
            1294, 1470, 1484, 1592, 1600, 2134, 2135, 2644, 2657, 2810, 3108, 3109, 3292, 3293,
            4522, 4523, 4524, 4526, 4665, 5001, 6058, 6060, 6072, 7669, 10171, 10214, 10215, 10216,
            10548, 10733, 12849,
        ],
        weapons: &[(0, 1927, 2, 10, 2, 17, 2)],
        sheath: 1,
    },
    ClassLook {
        id: 9,
        power: 0,
        trainers: &[
            1324, 1325, 1326, 1469, 1581, 1604, 1884, 1930, 2637, 2646, 2675, 3115, 3116, 3122,
            3271, 3286, 3291, 3345, 3607, 3745, 4567,
        ],
        weapons: &[(0, 5542, 2, 10, 2, 17, 2)],
        sheath: 1,
    },
    ClassLook {
        id: 11,
        power: 0,
        trainers: &[
            1706, 1732, 2106, 2115, 2121, 2250, 2255, 2261, 3300, 3301, 3302, 3819, 3820, 7357,
            10738, 12053,
        ],
        weapons: &[(0, 22391, 2, 10, 2, 17, 2)],
        sheath: 1,
    },
];

pub(crate) fn class_look(class: u8) -> Option<&'static ClassLook> {
    CLASS_LOOKS.iter().find(|c| c.id == class)
}

/// How close [`choose`] came to the character asked for.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Match {
    /// A trainer of exactly this race and sex.
    Exact,
    /// This race, the other sex: the class has no trainer of the asked sex in the race.
    SameRace,
    /// Neither: a combination 1.12 never offered (a Tauren Mage), or a display the catalog does
    /// not know. The first display that resolves stands in.
    StandIn,
}

/// The display to dress the character in, out of `class`'s trainers. `race_sex` answers what race
/// and sex a display wears (`None` for one it cannot say), so this works for any catalog.
/// Returns `None` only when no trainer resolves at all.
pub(crate) fn choose(
    class: &ClassLook,
    race: u8,
    sex: u8,
    race_sex: impl Fn(u32) -> Option<(u8, u8)>,
) -> Option<(u32, u8, u8, Match)> {
    let known: Vec<(u32, u8, u8)> = class
        .trainers
        .iter()
        .filter_map(|&d| race_sex(d).map(|(r, s)| (d, r, s)))
        .collect();
    let pick = |f: &dyn Fn(&&(u32, u8, u8)) -> bool| known.iter().find(f).copied();
    if let Some((d, r, s)) = pick(&|&&(_, r, s)| r == race && s == sex) {
        return Some((d, r, s, Match::Exact));
    }
    if let Some((d, r, s)) = pick(&|&&(_, r, _)| r == race) {
        return Some((d, r, s, Match::SameRace));
    }
    known.first().map(|&(d, r, s)| (d, r, s, Match::StandIn))
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A catalog that says display `d` is race `d % 8 + 1`, sex `d / 8 % 2`.
    fn fake(d: u32) -> Option<(u8, u8)> {
        Some(((d % 8) as u8 + 1, (d / 8 % 2) as u8))
    }

    #[test]
    fn every_class_the_params_accept_has_a_look() {
        for id in [1, 2, 3, 4, 5, 7, 8, 9, 11] {
            let c = class_look(id).unwrap_or_else(|| panic!("class {id}"));
            assert!(!c.trainers.is_empty() && !c.weapons.is_empty());
        }
        assert!(class_look(6).is_none());
    }

    #[test]
    fn an_exact_race_and_sex_wins() {
        let mage = class_look(8).unwrap();
        let (d, r, s, m) = choose(mage, 4, 1, fake).unwrap();
        assert_eq!((r, s, m), (4, 1, Match::Exact));
        assert_eq!(fake(d), Some((4, 1)));
    }

    #[test]
    fn a_missing_sex_falls_back_to_the_race_then_to_anything() {
        let mage = class_look(8).unwrap();
        // Only male displays resolve: a female request keeps the race.
        let males = |d: u32| fake(d).filter(|&(_, s)| s == 0);
        let (_, r, s, m) = choose(mage, 2, 1, males).unwrap();
        assert_eq!((r, s, m), (2, 0, Match::SameRace));
        // Only humans resolve: a Tauren request stands in with the first human.
        let humans = |d: u32| fake(d).filter(|&(r, _)| r == 1);
        let (_, r, _, m) = choose(mage, 6, 0, humans).unwrap();
        assert_eq!((r, m), (1, Match::StandIn));
    }

    #[test]
    fn nothing_resolving_is_none() {
        assert!(choose(class_look(8).unwrap(), 1, 0, |_| None).is_none());
    }
}
