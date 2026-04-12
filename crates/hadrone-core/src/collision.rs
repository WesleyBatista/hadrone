//! Pluggable collision resolution when an item overlaps others after move/resize.

use crate::LayoutItem;

/// Resolves overlaps after the focused item has been placed.
pub trait CollisionResolver: Send + Sync {
    fn resolve_collisions(&self, layout: &mut Vec<LayoutItem>, moved_id: &str);
}

/// Default RGL-style behavior: overlapping non-static items are pushed downward.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct PushDownResolver;

impl CollisionResolver for PushDownResolver {
    fn resolve_collisions(&self, layout: &mut Vec<LayoutItem>, moved_id: &str) {
        let Some(moved_item) = layout.iter().find(|i| i.id == moved_id).cloned() else {
            return;
        };

        let mut items_to_move = Vec::new();
        for item in layout.iter() {
            if item.id != moved_id && crate::collides(&moved_item, item) {
                items_to_move.push(item.id.clone());
            }
        }

        for id in items_to_move {
            if let Some(index) = layout.iter().position(|i| i.id == id)
                && !layout[index].is_static
            {
                layout[index].y = moved_item.y + moved_item.h;
                let next_id = layout[index].id.clone();
                self.resolve_collisions(layout, &next_id);
            }
        }
    }
}

/// No automatic displacement; overlaps remain until the next compaction.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct NoopCollisionResolver;

impl CollisionResolver for NoopCollisionResolver {
    fn resolve_collisions(&self, _layout: &mut Vec<LayoutItem>, _moved_id: &str) {}
}

/// Built-in strategies for [`crate::LayoutEngine`].
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum CollisionStrategy {
    #[default]
    PushDown,
    /// Skip collision resolution (may leave overlaps until compaction).
    None,
}

impl CollisionStrategy {
    pub fn build(self) -> Box<dyn CollisionResolver> {
        match self {
            CollisionStrategy::PushDown => Box::new(PushDownResolver),
            CollisionStrategy::None => Box::new(NoopCollisionResolver),
        }
    }
}
