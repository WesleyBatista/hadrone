//! Structured layout lifecycle events (RGL-style callbacks).

use crate::interaction::InteractionType;
use crate::{CollisionStrategy, CompactionType, LayoutItem};

/// Phase of a drag or resize interaction.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum InteractionPhase {
    Start,
    Update,
    Stop,
    Cancel,
}

/// Emitted by hosts for undo stacks, analytics, and custom constraints.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum LayoutEvent {
    Interaction {
        phase: InteractionPhase,
        id: String,
        interaction: InteractionType,
        /// Snapshot of the full layout after applying this step (when applicable).
        layout: Vec<LayoutItem>,
        compaction: CompactionType,
        collision: CollisionStrategy,
    },
    /// Column count or compaction changed outside an interaction (e.g. responsive).
    ConfigChanged {
        cols: i32,
        compaction: CompactionType,
    },
}
