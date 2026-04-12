//! Breakpoint selection and proportional layout scaling (RGL-style).

use crate::LayoutItem;
use serde::{Deserialize, Serialize};

/// Describes one responsive breakpoint (usually paired with CSS media queries).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct BreakpointSpec {
    pub name: String,
    /// Minimum container width in px for this breakpoint to apply.
    pub min_width_px: i32,
    pub cols: i32,
}

/// Pick the breakpoint with the largest `min_width_px` such that `container_width_px >= min_width_px`.
/// `breakpoints` should be sorted by `min_width_px` ascending.
pub fn select_breakpoint(
    breakpoints: &[BreakpointSpec],
    container_width_px: i32,
) -> Option<&BreakpointSpec> {
    let mut best: Option<&BreakpointSpec> = None;
    for bp in breakpoints {
        if container_width_px >= bp.min_width_px
            && best.is_none_or(|b| bp.min_width_px >= b.min_width_px)
        {
            best = Some(bp);
        }
    }
    best
}

/// Scale `x`/`w` when column count changes (React-Grid-Layout style).
/// `h`/`y` are left unchanged; run a [`crate::Compactor`] afterward if needed.
pub fn scale_layout_cols(items: &[LayoutItem], from_cols: i32, to_cols: i32) -> Vec<LayoutItem> {
    if from_cols < 1 || to_cols < 1 || from_cols == to_cols {
        return items.to_vec();
    }
    let ratio = to_cols as f64 / from_cols as f64;
    items
        .iter()
        .map(|it| {
            let mut n = it.clone();
            n.x = (it.x as f64 * ratio).round() as i32;
            n.w = ((it.w as f64 * ratio).round() as i32).max(1);
            n.x = n.x.max(0).min((to_cols - n.w).max(0));
            n
        })
        .collect()
}
