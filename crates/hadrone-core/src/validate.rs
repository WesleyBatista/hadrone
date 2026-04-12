//! Layout validation and automatic repair for persisted or imported grids.

use crate::LayoutItem;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Issues found by [`validate_layout`].
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum LayoutIssue {
    DuplicateId {
        id: String,
    },
    NonPositiveSize {
        id: String,
        w: i32,
        h: i32,
    },
    OutOfHorizontalBounds {
        id: String,
        x: i32,
        w: i32,
        cols: i32,
    },
    MinMaxWidth {
        id: String,
        w: i32,
        min_w: Option<i32>,
        max_w: Option<i32>,
    },
    MinMaxHeight {
        id: String,
        h: i32,
        min_h: Option<i32>,
        max_h: Option<i32>,
    },
    Overlap {
        a: String,
        b: String,
    },
}

/// Full validation pass (does not mutate).
pub fn validate_layout(layout: &[LayoutItem], cols: i32) -> Result<(), Vec<LayoutIssue>> {
    let mut issues = Vec::new();
    let mut seen: HashMap<&str, usize> = HashMap::new();

    for item in layout {
        if seen.insert(&item.id[..], 1).is_some() {
            issues.push(LayoutIssue::DuplicateId {
                id: item.id.clone(),
            });
        }
        if item.w < 1 || item.h < 1 {
            issues.push(LayoutIssue::NonPositiveSize {
                id: item.id.clone(),
                w: item.w,
                h: item.h,
            });
        }
        if item.x < 0 || item.x + item.w > cols {
            issues.push(LayoutIssue::OutOfHorizontalBounds {
                id: item.id.clone(),
                x: item.x,
                w: item.w,
                cols,
            });
        }
        if let Some(min) = item.min_w
            && item.w < min
        {
            issues.push(LayoutIssue::MinMaxWidth {
                id: item.id.clone(),
                w: item.w,
                min_w: item.min_w,
                max_w: item.max_w,
            });
        }
        if let Some(max) = item.max_w
            && item.w > max
        {
            issues.push(LayoutIssue::MinMaxWidth {
                id: item.id.clone(),
                w: item.w,
                min_w: item.min_w,
                max_w: item.max_w,
            });
        }
        if let Some(min) = item.min_h
            && item.h < min
        {
            issues.push(LayoutIssue::MinMaxHeight {
                id: item.id.clone(),
                h: item.h,
                min_h: item.min_h,
                max_h: item.max_h,
            });
        }
        if let Some(max) = item.max_h
            && item.h > max
        {
            issues.push(LayoutIssue::MinMaxHeight {
                id: item.id.clone(),
                h: item.h,
                min_h: item.min_h,
                max_h: item.max_h,
            });
        }
    }

    for i in 0..layout.len() {
        for j in (i + 1)..layout.len() {
            if crate::collides(&layout[i], &layout[j]) {
                issues.push(LayoutIssue::Overlap {
                    a: layout[i].id.clone(),
                    b: layout[j].id.clone(),
                });
            }
        }
    }

    if issues.is_empty() {
        Ok(())
    } else {
        Err(issues)
    }
}

/// Clamp positions/sizes into grid and min/max bounds; dedupe ids by suffixing.
pub fn repair_layout(layout: &mut [LayoutItem], cols: i32) {
    let mut seen: HashMap<String, u32> = HashMap::new();
    for item in layout.iter_mut() {
        let base = item.id.clone();
        let n = seen.entry(base.clone()).or_insert(0);
        if *n > 0 {
            item.id = format!("{base}-{}", *n);
        }
        *n += 1;

        item.w = item.w.max(1);
        item.h = item.h.max(1);
        item.x = item.x.max(0);
        if item.w > cols {
            item.w = cols;
        }
        item.x = item.x.min((cols - item.w).max(0));

        if let Some(min) = item.min_w {
            item.w = item.w.max(min);
        }
        if let Some(max) = item.max_w {
            item.w = item.w.min(max);
        }
        if let Some(min) = item.min_h {
            item.h = item.h.max(min);
        }
        if let Some(max) = item.max_h {
            item.h = item.h.min(max);
        }

        item.w = item.w.min(cols);
        item.x = item.x.min((cols - item.w).max(0));
    }
}
