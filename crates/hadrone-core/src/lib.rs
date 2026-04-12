//! # Grid Layout Core
//!
//! The spatial engine for a cross-framework grid layout system.
//! This crate contains the deterministic math for grid compaction, collision detection,
//! and interaction sessions. It is designed to be headless and framework-agnostic.

pub mod collision;
pub mod events;
pub mod interaction;
pub mod responsive;
pub mod validate;

pub use collision::{CollisionResolver, CollisionStrategy, NoopCollisionResolver, PushDownResolver};
pub use events::{InteractionPhase, LayoutEvent};
pub use responsive::{BreakpointSpec, scale_layout_cols, select_breakpoint};
pub use validate::{LayoutIssue, repair_layout, validate_layout};

use serde::{Deserialize, Serialize};
use std::collections::HashSet;

/// Represents a handle for resizing a grid item.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ResizeHandle {
    North,
    South,
    East,
    West,
    NorthEast,
    NorthWest,
    SouthEast,
    SouthWest,
}

/// Human-readable label for assistive tech (e.g. `aria-label` on resize controls).
pub fn resize_handle_aria_label(handle: ResizeHandle) -> &'static str {
    match handle {
        ResizeHandle::North => "Resize top edge",
        ResizeHandle::South => "Resize bottom edge",
        ResizeHandle::East => "Resize right edge",
        ResizeHandle::West => "Resize left edge",
        ResizeHandle::NorthEast => "Resize top-right corner",
        ResizeHandle::NorthWest => "Resize top-left corner",
        ResizeHandle::SouthEast => "Resize bottom-right corner",
        ResizeHandle::SouthWest => "Resize bottom-left corner",
    }
}

/// A single item within the grid layout.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct LayoutItem {
    /// Unique identifier for the item.
    pub id: String,
    /// X coordinate in grid units.
    pub x: i32,
    /// Y coordinate in grid units.
    pub y: i32,
    /// Width in grid units.
    pub w: i32,
    /// Height in grid units.
    pub h: i32,
    /// Minimum allowed width.
    pub min_w: Option<i32>,
    /// Maximum allowed width.
    pub max_w: Option<i32>,
    /// Minimum allowed height.
    pub min_h: Option<i32>,
    /// Maximum allowed height.
    pub max_h: Option<i32>,
    /// Optional fixed aspect ratio **width / height** in grid units.
    pub aspect_ratio: Option<f32>,
    /// If true, the item cannot be dragged or resized and is fixed for compaction.
    pub is_static: bool,
    /// When `is_static` is false: allow dragging (subject to compaction).
    pub is_draggable: bool,
    /// When `is_static` is false: allow resizing.
    pub is_resizable: bool,
    /// Enabled resize handles for this item.
    pub resize_handles: HashSet<ResizeHandle>,
}

impl LayoutItem {
    /// User-driven drag allowed (keyboard / pointer).
    #[inline]
    pub fn can_drag(&self) -> bool {
        !self.is_static && self.is_draggable
    }

    /// User-driven resize allowed.
    #[inline]
    pub fn can_resize(&self) -> bool {
        !self.is_static && self.is_resizable
    }
}

impl Default for LayoutItem {
    fn default() -> Self {
        let mut handles = HashSet::new();
        handles.insert(ResizeHandle::SouthEast);
        Self {
            id: "".into(),
            x: 0,
            y: 0,
            w: 1,
            h: 1,
            min_w: None,
            max_w: None,
            min_h: None,
            max_h: None,
            aspect_ratio: None,
            is_static: false,
            is_draggable: true,
            is_resizable: true,
            resize_handles: handles,
        }
    }
}

/// Trait for grid compaction strategies.
/// Compaction is the process of resolving overlaps and settling items into a stable layout.
pub trait Compactor {
    /// Compacts the layout by resolving collisions and moving items.
    fn compact(&self, layout: &mut Vec<LayoutItem>, cols: i32);
}

/// Supported compaction strategies.
#[derive(Debug, Clone, Copy, PartialEq, Default, Serialize, Deserialize)]
pub enum CompactionType {
    /// Items settle towards the top of the grid (Y=0).
    #[default]
    Gravity,
    /// Items stay at their requested positions unless they collide.
    FreePlacement,
}

/// O(N * Cols) implementation using the "waterline" approach
pub struct RisingTideCompactor;

impl Compactor for RisingTideCompactor {
    fn compact(&self, layout: &mut Vec<LayoutItem>, cols: i32) {
        // Sort items by Y, then X
        layout.sort_by(|a, b| a.y.cmp(&b.y).then(a.x.cmp(&b.x)));

        let mut waterline = vec![0; cols as usize];
        let mut new_layout = Vec::with_capacity(layout.len());

        for mut item in layout.drain(..) {
            if !item.is_static {
                // Find max waterline in the range [x, x+w)
                let start_col = item.x.max(0) as usize;
                let end_col = (item.x + item.w).min(cols) as usize;

                let max_y = waterline[start_col..end_col]
                    .iter()
                    .max()
                    .copied()
                    .unwrap_or(0);

                // Set Y to the max waterline found
                item.y = max_y;

                // Update waterline for the range
                let new_y_plus_h = item.y + item.h;
                waterline[start_col..end_col].fill(new_y_plus_h);
            } else {
                // Static items update the waterline at their fixed position
                let start_col = item.x.max(0) as usize;
                let end_col = (item.x + item.w).min(cols) as usize;
                let new_y_plus_h = item.y + item.h;
                for val in &mut waterline[start_col..end_col] {
                    *val = (*val).max(new_y_plus_h);
                }
            }
            new_layout.push(item);
        }

        *layout = new_layout;
    }
}

pub struct FreePlacementCompactor;

impl Compactor for FreePlacementCompactor {
    fn compact(&self, layout: &mut Vec<LayoutItem>, _cols: i32) {
        // Sort items by Y, then X
        layout.sort_by(|a, b| a.y.cmp(&b.y).then(a.x.cmp(&b.x)));

        let mut processed: Vec<LayoutItem> = Vec::with_capacity(layout.len());

        for mut item in layout.drain(..) {
            // Keep bumping y if there's a collision with any already processed item
            // This preserves the item's requested Y as much as possible without overlaps
            while processed.iter().any(|other| collides(&item, other)) {
                item.y += 1;
            }
            processed.push(item);
        }

        *layout = processed;
    }
}

/// The main orchestration point for grid operations.
pub struct LayoutEngine {
    /// The compaction strategy to use.
    pub compactor: Box<dyn Compactor>,
    /// Displacement policy after overlaps.
    pub collision: Box<dyn CollisionResolver>,
    /// The number of columns in the grid.
    pub cols: i32,
}

impl LayoutEngine {
    pub fn new(
        compactor: Box<dyn Compactor>,
        collision: Box<dyn CollisionResolver>,
        cols: i32,
    ) -> Self {
        Self {
            compactor,
            collision,
            cols,
        }
    }

    /// Gravity compaction with push-down collision resolution.
    pub fn with_default_collision(compactor: Box<dyn Compactor>, cols: i32) -> Self {
        Self::new(compactor, CollisionStrategy::PushDown.build(), cols)
    }

    pub fn compact(&self, layout: &mut Vec<LayoutItem>) {
        self.compactor.compact(layout, self.cols);
    }

    pub fn move_element(&self, layout: &mut Vec<LayoutItem>, id: &str, x: i32, y: i32) {
        if let Some(index) = layout.iter().position(|i| i.id == id) {
            let mut item = layout[index].clone();
            if !item.can_drag() {
                return;
            }

            item.x = x.max(0).min(self.cols - item.w);
            item.y = y.max(0);

            layout[index] = item;
            self.collision.resolve_collisions(layout, id);
            self.compact(layout);
        }
    }

    #[allow(clippy::too_many_arguments)]
    pub fn resize_element(
        &self,
        layout: &mut Vec<LayoutItem>,
        id: &str,
        x: i32,
        y: i32,
        w: i32,
        h: i32,
        handle: Option<ResizeHandle>,
    ) {
        if let Some(index) = layout.iter().position(|i| i.id == id) {
            let mut item = layout[index].clone();
            if !item.can_resize() {
                return;
            }

            let mut final_w = w;
            let mut final_h = h;
            if let Some(min) = item.min_w {
                final_w = final_w.max(min);
            }
            if let Some(max) = item.max_w {
                final_w = final_w.min(max);
            }
            if let Some(min) = item.min_h {
                final_h = final_h.max(min);
            }
            if let Some(max) = item.max_h {
                final_h = final_h.min(max);
            }

            let final_x = x.max(0).min(self.cols - final_w);
            let final_y = y.max(0);

            item.x = final_x;
            item.y = final_y;
            item.w = final_w.max(1);
            item.h = final_h.max(1);

            apply_aspect_and_clamp(&mut item, self.cols, handle);

            layout[index] = item;
            self.collision.resolve_collisions(layout, id);
            self.compact(layout);
        }
    }
}

/// Applies `aspect_ratio` (w/h) plus min/max; re-clamps `x`/`w` to `cols`.
pub fn apply_aspect_and_clamp(item: &mut LayoutItem, cols: i32, handle: Option<ResizeHandle>) {
    let mut w = item.w.max(1);
    let mut h = item.h.max(1);

    if let Some(ar) = item.aspect_ratio.filter(|a| a.is_finite() && *a > 0.0) {
        let prefer_width = handle.map(aspect_prefers_width).unwrap_or(true);
        for _ in 0..6 {
            if prefer_width {
                h = (w as f32 / ar).round() as i32;
            } else {
                w = (h as f32 * ar).round() as i32;
            }
            h = h.max(1);
            w = w.max(1);
            if let Some(min) = item.min_h {
                h = h.max(min);
            }
            if let Some(max) = item.max_h {
                h = h.min(max);
            }
            if let Some(min) = item.min_w {
                w = w.max(min);
            }
            if let Some(max) = item.max_w {
                w = w.min(max);
            }
            if prefer_width {
                w = (h as f32 * ar).round() as i32;
            } else {
                h = (w as f32 / ar).round() as i32;
            }
            w = w.max(1);
            h = h.max(1);
        }
    }

    item.w = w.min(cols).max(1);
    item.h = h.max(1);
    item.x = item.x.max(0).min((cols - item.w).max(0));
    item.y = item.y.max(0);
}

fn aspect_prefers_width(handle: ResizeHandle) -> bool {
    match handle {
        ResizeHandle::East
        | ResizeHandle::West
        | ResizeHandle::NorthEast
        | ResizeHandle::SouthEast => true,
        ResizeHandle::North | ResizeHandle::South => false,
        ResizeHandle::NorthWest | ResizeHandle::SouthWest => false,
    }
}

/// Checks if two layout items overlap.
pub fn collides(a: &LayoutItem, b: &LayoutItem) -> bool {
    if a.id == b.id {
        return false;
    }
    !(a.x + a.w <= b.x || a.x >= b.x + b.w || a.y + a.h <= b.y || a.y >= b.y + b.h)
}

/// Build an engine from compaction + collision strategy (common in interaction code).
pub fn layout_engine(
    compaction: CompactionType,
    collision: CollisionStrategy,
    cols: i32,
) -> LayoutEngine {
    let compactor: Box<dyn Compactor> = match compaction {
        CompactionType::Gravity => Box::new(RisingTideCompactor),
        CompactionType::FreePlacement => Box::new(FreePlacementCompactor),
    };
    LayoutEngine::new(compactor, collision.build(), cols)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::interaction::{InteractionSession, InteractionType};
    use crate::validate::LayoutIssue;
    use std::collections::HashSet;

    fn item(id: &str, x: i32, y: i32, w: i32, h: i32) -> LayoutItem {
        LayoutItem {
            id: id.into(),
            x,
            y,
            w,
            h,
            ..Default::default()
        }
    }

    #[test]
    fn collides_false_for_same_id() {
        let a = item("a", 0, 0, 2, 2);
        assert!(!collides(&a, &a));
    }

    #[test]
    fn collides_true_when_overlapping() {
        let a = item("a", 0, 0, 2, 2);
        let b = item("b", 1, 1, 2, 2);
        assert!(collides(&a, &b));
    }

    #[test]
    fn collides_false_when_adjacent() {
        let a = item("a", 0, 0, 2, 2);
        let b = item("b", 2, 0, 2, 2);
        assert!(!collides(&a, &b));
    }

    #[test]
    fn rising_tide_stacks_vertically() {
        let mut layout = vec![item("a", 0, 5, 4, 2), item("b", 0, 0, 4, 2)];
        RisingTideCompactor.compact(&mut layout, 12);
        let a = layout.iter().find(|i| i.id == "a").unwrap();
        let b = layout.iter().find(|i| i.id == "b").unwrap();
        assert_eq!(b.y, 0);
        assert_eq!(a.y, 2);
        assert!(!collides(a, b));
    }

    #[test]
    fn free_placement_pushes_second_item_down() {
        let mut layout = vec![item("a", 0, 0, 4, 4), item("b", 1, 1, 2, 2)];
        FreePlacementCompactor.compact(&mut layout, 12);
        let b = layout.iter().find(|i| i.id == "b").unwrap();
        assert_eq!(b.y, 4);
    }

    #[test]
    fn move_element_clamps_to_grid_width() {
        let engine = LayoutEngine::with_default_collision(Box::new(RisingTideCompactor), 6);
        let mut layout = vec![item("w", 0, 0, 4, 1)];
        engine.move_element(&mut layout, "w", 10, 0);
        let w = layout.iter().find(|i| i.id == "w").unwrap();
        assert_eq!(w.x, 2);
    }

    #[test]
    fn static_item_does_not_move_in_compactor() {
        let mut layout = vec![LayoutItem {
            id: "s".into(),
            x: 0,
            y: 3,
            w: 2,
            h: 1,
            is_static: true,
            ..Default::default()
        }];
        RisingTideCompactor.compact(&mut layout, 12);
        assert_eq!(layout[0].y, 3);
    }

    #[test]
    fn resize_element_applies_min_width() {
        let engine = LayoutEngine::with_default_collision(Box::new(RisingTideCompactor), 12);
        let mut handles = HashSet::new();
        handles.insert(ResizeHandle::SouthEast);
        let mut layout = vec![LayoutItem {
            id: "x".into(),
            x: 0,
            y: 0,
            w: 4,
            h: 2,
            min_w: Some(3),
            resize_handles: handles,
            ..Default::default()
        }];
        engine.resize_element(&mut layout, "x", 0, 0, 1, 2, Some(ResizeHandle::East));
        let x = layout.iter().find(|i| i.id == "x").unwrap();
        assert_eq!(x.w, 3);
    }

    #[test]
    fn interaction_drag_updates_position() {
        let mut handles = HashSet::new();
        handles.insert(ResizeHandle::SouthEast);
        let mut layout = vec![LayoutItem {
            id: "d".into(),
            x: 0,
            y: 0,
            w: 2,
            h: 2,
            resize_handles: handles,
            ..Default::default()
        }];
        let session = InteractionSession {
            id: "d".into(),
            interaction_type: InteractionType::Drag,
            start_mouse: (0.0, 0.0),
            start_rect: (0, 0, 2, 2),
            handle: ResizeHandle::SouthEast,
            col_width_px: 100.0,
            row_height_px: 50.0,
            margin: (0, 10),
            container_padding: (0, 0),
            compaction: CompactionType::Gravity,
            collision: CollisionStrategy::PushDown,
        };
        session.update((200.0, 0.0), &mut layout, 12);
        let d = layout.iter().find(|i| i.id == "d").unwrap();
        assert_eq!(d.x, 2);
        assert_eq!(d.y, 0);
    }

    #[test]
    fn scale_layout_cols_halves_positions() {
        let items = vec![item("a", 4, 1, 4, 2)];
        let out = scale_layout_cols(&items, 12, 6);
        let a = out.iter().find(|i| i.id == "a").unwrap();
        assert_eq!(a.x, 2);
        assert_eq!(a.w, 2);
    }

    #[test]
    fn aspect_ratio_enforced_on_resize() {
        let engine = LayoutEngine::with_default_collision(Box::new(RisingTideCompactor), 12);
        let mut layout = vec![LayoutItem {
            id: "ar".into(),
            x: 0,
            y: 0,
            w: 4,
            h: 2,
            aspect_ratio: Some(2.0),
            ..Default::default()
        }];
        engine.resize_element(&mut layout, "ar", 0, 0, 2, 2, Some(ResizeHandle::East));
        let it = layout.iter().find(|i| i.id == "ar").unwrap();
        assert_eq!(it.w, 2);
        assert_eq!(it.h, 1);
    }

    #[test]
    fn validate_layout_detects_duplicate_ids() {
        let layout = vec![
            item("dup", 0, 0, 2, 2),
            LayoutItem {
                id: "dup".into(),
                x: 2,
                y: 0,
                w: 2,
                h: 2,
                ..Default::default()
            },
        ];
        let err = validate_layout(&layout, 12).unwrap_err();
        assert!(err.iter().any(|e| matches!(e, LayoutIssue::DuplicateId { .. })));
    }
}
