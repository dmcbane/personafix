use serde::{Deserialize, Serialize};
use ts_rs::TS;

use super::edition::Edition;

/// The core physical and mental attributes shared across SR4 and SR5.
#[derive(Debug, Clone, Default, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct Attributes {
    pub body: u8,
    pub agility: u8,
    pub reaction: u8,
    pub strength: u8,
    pub willpower: u8,
    pub logic: u8,
    pub intuition: u8,
    pub charisma: u8,
    /// Edge attribute (both editions, but mechanics differ).
    pub edge: u8,
    /// Essence is tracked as centessence (integer hundredths) to avoid floating point.
    /// 600 = 6.00 Essence.
    pub essence: i32,
    /// Magic rating. None if mundane.
    pub magic: Option<u8>,
    /// Resonance rating. None if not a technomancer.
    pub resonance: Option<u8>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, TS)]
#[ts(export)]
pub enum Metatype {
    Human,
    Elf,
    Dwarf,
    Ork,
    Troll,
}

/// Racial attribute limits for a metatype in a given edition.
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct RacialLimits {
    pub metatype: Metatype,
    pub edition: Edition,
    pub body: (u8, u8),
    pub agility: (u8, u8),
    pub reaction: (u8, u8),
    pub strength: (u8, u8),
    pub willpower: (u8, u8),
    pub logic: (u8, u8),
    pub intuition: (u8, u8),
    pub charisma: (u8, u8),
    pub edge: (u8, u8),
}
