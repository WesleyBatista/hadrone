//! Property-based checks for layout invariants.

use hadrone_core::{
    CollisionStrategy, FreePlacementCompactor, LayoutEngine, LayoutItem, RisingTideCompactor,
    collides, repair_layout, validate_layout,
};
use proptest::prelude::*;

fn layout_item_strategy() -> impl Strategy<Value = LayoutItem> {
    (
        any::<u16>(),
        0i32..8i32,
        0i32..20i32,
        1i32..5i32,
        1i32..6i32,
    )
        .prop_map(|(id, x, y, w, h)| LayoutItem {
            id: format!("i-{id}"),
            x,
            y,
            w,
            h,
            ..Default::default()
        })
}

proptest! {
    #[test]
    fn repair_then_validate(cols in 4i32..16i32, mut items in prop::collection::vec(layout_item_strategy(), 1..12)) {
        repair_layout(&mut items, cols);
        let _ = validate_layout(&items, cols);
    }
}

#[test]
fn pushdown_then_no_overlaps_gravity() {
    let mut layout = vec![
        LayoutItem {
            id: "a".into(),
            x: 0,
            y: 5,
            w: 3,
            h: 2,
            ..Default::default()
        },
        LayoutItem {
            id: "b".into(),
            x: 0,
            y: 0,
            w: 3,
            h: 2,
            ..Default::default()
        },
    ];
    let engine = LayoutEngine::with_default_collision(Box::new(RisingTideCompactor), 12);
    engine.compact(&mut layout);
    for i in 0..layout.len() {
        for j in (i + 1)..layout.len() {
            assert!(!collides(&layout[i], &layout[j]));
        }
    }
}

#[test]
fn noop_collision_freeplacement_compact_clears_overlap() {
    let mut layout = vec![
        LayoutItem {
            id: "a".into(),
            x: 0,
            y: 0,
            w: 4,
            h: 2,
            ..Default::default()
        },
        LayoutItem {
            id: "b".into(),
            x: 1,
            y: 0,
            w: 2,
            h: 2,
            ..Default::default()
        },
    ];
    let engine = LayoutEngine::new(
        Box::new(FreePlacementCompactor),
        CollisionStrategy::None.build(),
        12,
    );
    engine.move_element(&mut layout, "b", 1, 0);
    let engine2 = LayoutEngine::with_default_collision(Box::new(FreePlacementCompactor), 12);
    engine2.compact(&mut layout);
    for i in 0..layout.len() {
        for j in (i + 1)..layout.len() {
            assert!(!collides(&layout[i], &layout[j]));
        }
    }
}
