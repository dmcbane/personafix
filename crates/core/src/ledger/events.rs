use serde::{Deserialize, Serialize};
use ts_rs::TS;

/// All character mutations are ledger events. The current character state
/// is a projection of the base + all events. Events are append-only.
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub enum LedgerEvent {
    /// A shadowrun was completed.
    RunCompleted {
        run_id: String,
        name: String,
        date: String,
        notes: String,
    },
    /// Karma received (from a run, GM award, etc.).
    KarmaReceived {
        amount: i32,
        reason: String,
        run_id: Option<String>,
    },
    /// Karma spent on an improvement.
    KarmaSpent { amount: i32, description: String },
    /// Nuyen received.
    NuyenReceived {
        amount: i64,
        reason: String,
        run_id: Option<String>,
    },
    /// Nuyen spent.
    NuyenSpent { amount: i64, description: String },
    /// Gear acquired.
    GearAcquired {
        item_id: String,
        item_name: String,
        cost: i64,
    },
    /// Gear lost or sold.
    GearLost { item_id: String, item_name: String },
    /// Contact added.
    ContactAdded {
        contact_id: String,
        name: String,
        connection: u8,
        loyalty: u8,
    },
    /// Contact loyalty or connection changed.
    ContactChanged {
        contact_id: String,
        new_connection: u8,
        new_loyalty: u8,
    },
    /// Contact lost (burned, killed, etc.).
    ContactLost { contact_id: String, reason: String },
    /// Skill improved via karma.
    SkillImproved {
        skill_name: String,
        from: u8,
        to: u8,
        karma_cost: i32,
    },
    /// Attribute improved via karma.
    AttributeImproved {
        attribute: String,
        from: u8,
        to: u8,
        karma_cost: i32,
    },
    /// Initiation grade increased (mages).
    Initiated { new_grade: u8, karma_cost: i32 },
    /// Submersion grade increased (technomancers).
    Submerged { new_grade: u8, karma_cost: i32 },
    /// Quality added post-creation.
    QualityAdded {
        quality_id: String,
        quality_name: String,
        karma_cost: i32,
    },
    /// Quality removed (bought off).
    QualityRemoved {
        quality_id: String,
        quality_name: String,
        karma_cost: i32,
    },
}
