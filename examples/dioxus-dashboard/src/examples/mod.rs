pub mod aspect_ratio;
pub mod basic;
pub mod collisions;
pub mod debugger;
pub mod dynamic_add_remove;
pub mod gravity;
pub mod min_max;
pub mod no_dragging;
pub mod responsive;

pub use aspect_ratio::AspectRatioExample;
pub use basic::BasicExample;
pub use collisions::CollisionsExample;
pub use debugger::DebuggerExample;
pub use dynamic_add_remove::DynamicAddRemoveExample;
pub use gravity::GravityExample;
pub use min_max::MinMaxExample;
pub use no_dragging::NoDraggingExample;
pub use responsive::ResponsiveExample;

use hadrone_core::LayoutItem;
use std::collections::HashSet;

pub fn default_resize_handles() -> HashSet<hadrone_core::ResizeHandle> {
    let mut handles = HashSet::new();
    handles.insert(hadrone_core::ResizeHandle::SouthEast);
    handles.insert(hadrone_core::ResizeHandle::South);
    handles.insert(hadrone_core::ResizeHandle::East);
    handles
}

pub fn generate_random_layout(count: usize, cols: i32) -> Vec<LayoutItem> {
    (0..count)
        .map(|i| {
            LayoutItem {
                id: format!("item-{}", i),
                x: (i as i32 * 2) % cols,
                y: ((i / 6) as i32) * 2,
                w: 2,
                h: 2,
                resize_handles: default_resize_handles(),
                ..Default::default()
            }
        })
        .collect()
}
