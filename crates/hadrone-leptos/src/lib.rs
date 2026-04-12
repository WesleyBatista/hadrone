use hadrone_core::interaction::{InteractionSession, InteractionType};
use hadrone_core::{
    resize_handle_aria_label, CollisionStrategy, CompactionType, Compactor, FreePlacementCompactor,
    LayoutEngine, LayoutItem, ResizeHandle, RisingTideCompactor,
};

pub use hadrone_core::{InteractionPhase, LayoutEvent};
use leptos::ev;
use leptos::ev::PointerEvent;
use leptos::*;

fn leptos_apply_keyboard_cell_nudge(
    layout: RwSignal<Vec<LayoutItem>>,
    cols: i32,
    compaction: CompactionType,
    item_id: &str,
    dx: i32,
    dy: i32,
) {
    layout.update(|l| {
        let Some((nx, ny)) = l
            .iter()
            .find(|i| i.id == item_id)
            .filter(|it| it.can_drag())
            .map(|it| (it.x + dx, it.y + dy))
        else {
            return;
        };
        let compactor: Box<dyn Compactor> = match compaction {
            CompactionType::Gravity => Box::new(RisingTideCompactor),
            CompactionType::FreePlacement => Box::new(FreePlacementCompactor),
        };
        let engine = LayoutEngine::with_default_collision(compactor, cols);
        engine.move_element(l, item_id, nx, ny);
    });
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct GridConfig {
    pub cols: i32,
    pub row_height: f32,
    pub margin: (i32, i32),
}

#[component]
pub fn GridLayout(
    layout: RwSignal<Vec<LayoutItem>>,
    #[prop(into)] cols: Signal<i32>,
    #[prop(into)] row_height: Signal<f32>,
    #[prop(into)] margin: Signal<(i32, i32)>,
    #[prop(into)] compaction: Signal<CompactionType>,
    #[prop(default = false)] keyboard_cell_nudge: bool,
    render_item: fn(LayoutItem) -> View,
) -> impl IntoView {
    let active = create_rw_signal(None::<InteractionSession>);
    let visual_delta = create_rw_signal(None::<(f32, f32, f32, f32)>);
    let container_width = create_rw_signal(1200.0f32);
    let container_ref = create_node_ref::<html::Div>();

    // Track container width
    create_effect(move |_| {
        if let Some(el) = container_ref.get() {
            let el_clone = el.clone();
            let handle = window_event_listener(ev::resize, move |_| {
                container_width.set(el_clone.client_width() as f32);
            });
            on_cleanup(move || drop(handle));
            container_width.set(el.client_width() as f32);
        }
    });

    let total_height = create_memo(move |prev: Option<&f32>| {
        if active.get().is_some() {
            return *prev.unwrap_or(&500.0);
        }
        let l = layout.get();
        let max_y = l.iter().map(|item| item.y + item.h).max().unwrap_or(0);
        let rh = row_height.get();
        let my = margin.get().1;
        (max_y as f32 * (rh + my as f32)).max(500.0)
    });

    // Auto-compaction when cols/compaction changes
    create_effect(move |_| {
        if active.get().is_some() {
            return;
        }
        let mut current_layout = layout.get_untracked();
        let ccols = cols.get();
        let compactor: Box<dyn Compactor> = match compaction.get() {
            CompactionType::Gravity => Box::new(RisingTideCompactor),
            CompactionType::FreePlacement => Box::new(FreePlacementCompactor),
        };
        let engine = LayoutEngine::with_default_collision(compactor, ccols);
        for item in current_layout.iter_mut() {
            if !item.is_static {
                item.w = item.w.min(ccols);
                item.x = item.x.max(0).min(ccols - item.w);
            }
        }
        engine.compact(&mut current_layout);
        layout.set(current_layout);
    });

    let on_pointer_move = move |e: PointerEvent| {
        if let Some(interaction) = active.get().as_ref() {
            let coords = (e.client_x() as f32, e.client_y() as f32);
            visual_delta.set(Some(interaction.get_visual_delta(coords)));
            let mut new_layout = layout.get_untracked();
            interaction.update(coords, &mut new_layout, cols.get());
            layout.set(new_layout);
        }
    };

    let on_resize_up = move |e: PointerEvent| {
        if active.get().is_some() {
            handle_capture_release(e.pointer_id());
            active.set(None);
            visual_delta.set(None);
        }
    };

    let style = move || {
        format!(
        "position: relative; width: 100%; height: {}px; contain: layout; touch-action: none; user-select: none;",
        total_height.get()
    )
    };

    let pointer_interaction = Signal::derive(move || active.get().is_some());

    view! {
        <div
            node_ref=container_ref
            class="hadrone-container"
            style=style
            data-active=move || active.get().is_some().to_string()
            role="application"
            aria-label="Draggable grid layout. Use Tab to reach widgets and resize handles. Arrow keys move the focused widget when keyboard nudge is enabled."
            on:pointermove=on_pointer_move
            on:pointerup=on_resize_up
            on:pointerleave=on_resize_up
            on:pointercancel=on_resize_up
        >
            <style>
                "
                .resize-handle { opacity: 0; pointer-events: none; transition: opacity 0.15s ease-in-out; }
                .grid-item:hover .resize-handle { opacity: 1; pointer-events: auto; }
                .hadrone-container[data-active=\"true\"] { cursor: grabbing !important; }
                .hadrone-container[data-active=\"true\"] .grid-item:not([data-active=\"true\"]) .resize-handle { opacity: 0 !important; pointer-events: none !important; }
                .grid-item[data-active=\"true\"] .resize-handle { opacity: 1 !important; pointer-events: auto !important; }
                .grid-item-inner:focus-visible { outline: 2px solid #2563eb; outline-offset: 2px; }
                .resize-handle:focus-visible { opacity: 1 !important; pointer-events: auto !important; outline: 2px solid #2563eb; outline-offset: 2px; }
                "
            </style>
            <For
                each=move || layout.get()
                key=|item| item.id.clone()
                children=move |item| {
                    let item_id = item.id.clone();
                    let item_id_for_active   = item_id.clone();
                    let item_id_for_rect     = item_id.clone();
                    let item_id_for_delta    = item_id.clone();
                    let item_id_for_drag     = item_id.clone();
                    let item_id_for_resize   = item_id.clone();

                    let is_active_sig = Signal::derive(move || {
                        active.get().as_ref().is_some_and(|a| a.id == item_id_for_active)
                    });

                    let start_rect_sig = Signal::derive(move || {
                        if active.get().as_ref().is_some_and(|a| a.id == item_id_for_rect) {
                            active.get().as_ref().map(|a| a.start_rect)
                        } else {
                            None
                        }
                    });

                    let visual_delta_sig = Signal::derive(move || {
                        if active.get().as_ref().is_some_and(|a| a.id == item_id_for_delta) {
                            visual_delta.get()
                        } else {
                            None
                        }
                    });

                    let on_drag_start = move |e: PointerEvent| {
                        handle_capture_set(e.pointer_id());
                        let start_mouse = (e.client_x() as f32, e.client_y() as f32);
                        let Some(i) = layout
                            .get_untracked()
                            .into_iter()
                            .find(|it| it.id == item_id_for_drag)
                        else {
                            return;
                        };
                        if !i.can_drag() {
                            return;
                        }
                        let session = InteractionSession {
                            id: i.id.clone(),
                            start_mouse,
                            start_rect: (i.x, i.y, i.w, i.h),
                            interaction_type: InteractionType::Drag,
                            handle: ResizeHandle::SouthEast,
                            col_width_px: container_width.get() / cols.get() as f32,
                            row_height_px: row_height.get(),
                            margin: margin.get(),
                            container_padding: (0, 0),
                            compaction: compaction.get(),
                            collision: CollisionStrategy::PushDown,
                        };
                        visual_delta.set(Some(session.get_visual_delta(start_mouse)));
                        active.set(Some(session));
                    };

                    let handles: Vec<ResizeHandle> = item
                        .resize_handles
                        .iter()
                        .cloned()
                        .filter(|h| {
                            item.can_resize()
                                && matches!(
                                    h,
                                    ResizeHandle::SouthEast | ResizeHandle::South | ResizeHandle::East
                                )
                        })
                        .collect();

                    let on_resize_start = move |e: PointerEvent, handle: ResizeHandle| {
                        handle_capture_set(e.pointer_id());
                        let start_mouse = (e.client_x() as f32, e.client_y() as f32);
                        let Some(i) = layout
                            .get_untracked()
                            .into_iter()
                            .find(|it| it.id == item_id_for_resize)
                        else {
                            return;
                        };
                        if !i.can_resize() {
                            return;
                        }
                        let session = InteractionSession {
                            id: i.id.clone(),
                            start_mouse,
                            start_rect: (i.x, i.y, i.w, i.h),
                            interaction_type: InteractionType::Resize,
                            handle,
                            col_width_px: container_width.get() / cols.get() as f32,
                            row_height_px: row_height.get(),
                            margin: margin.get(),
                            container_padding: (0, 0),
                            compaction: compaction.get(),
                            collision: CollisionStrategy::PushDown,
                        };
                        visual_delta.set(Some(session.get_visual_delta(start_mouse)));
                        active.set(Some(session));
                    };

                    view! {
                        <GridItem
                            item=item.clone()
                            layout=layout
                            cols=cols
                            row_height=row_height
                            margin=margin
                            compaction=compaction
                            keyboard_cell_nudge=keyboard_cell_nudge
                            pointer_interaction=pointer_interaction
                            is_active=is_active_sig
                            start_rect=start_rect_sig
                            visual_delta=visual_delta_sig
                            render_item=render_item
                            on_drag_start=on_drag_start
                            on_resize_start=on_resize_start
                            resize_handles=handles
                        />
                    }
                }
            />
        </div>
    }
}

fn handle_capture_set(_pid: i32) {
    #[cfg(target_arch = "wasm32")]
    {
        if let Some(el) = web_sys::window()
            .and_then(|w| w.document())
            .and_then(|d| d.query_selector(".hadrone-container").ok().flatten())
        {
            let _ = el.set_pointer_capture(_pid);
        }
    }
}

fn handle_capture_release(_pid: i32) {
    #[cfg(target_arch = "wasm32")]
    {
        if let Some(el) = web_sys::window().and_then(|w| w.document()).and_then(|d| {
            d.query_selector(".hadrone-container[data-active='true']")
                .ok()
                .flatten()
        }) {
            let _ = el.release_pointer_capture(_pid);
        }
    }
}

#[component]
pub fn GridItem<F, R>(
    item: LayoutItem,
    layout: RwSignal<Vec<LayoutItem>>,
    #[prop(into)] cols: Signal<i32>,
    #[prop(into)] row_height: Signal<f32>,
    #[prop(into)] margin: Signal<(i32, i32)>,
    #[prop(into)] compaction: Signal<CompactionType>,
    #[prop(into)] pointer_interaction: Signal<bool>,
    keyboard_cell_nudge: bool,
    #[prop(into)] is_active: Signal<bool>,
    #[prop(into)] start_rect: Signal<Option<(i32, i32, i32, i32)>>,
    #[prop(into)] visual_delta: Signal<Option<(f32, f32, f32, f32)>>,
    render_item: fn(LayoutItem) -> View,
    on_drag_start: F,
    on_resize_start: R,
    resize_handles: Vec<ResizeHandle>,
) -> impl IntoView
where
    F: Fn(PointerEvent) + 'static,
    R: Fn(PointerEvent, ResizeHandle) + Clone + 'static,
{
    let col_width_pct = move || 100.0 / cols.get() as f32;

    let style = move || {
        let (left_str, top_str, width_str, height_str) =
            if let (Some((dx, dy, dw, dh)), Some(sr)) = (visual_delta.get(), start_rect.get()) {
                let start_left_pct = sr.0 as f32 * col_width_pct();
                let start_top_px = sr.1 as f32 * (row_height.get() + margin.get().1 as f32);
                let start_width_pct = sr.2 as f32 * col_width_pct();
                let start_height_px =
                    sr.3 as f32 * row_height.get() + (sr.3 as f32 - 1.0) * margin.get().1 as f32;
                (
                    format!("calc({}% + {}px)", start_left_pct, dx),
                    format!("{}px", start_top_px + dy),
                    format!(
                        "calc({}% - {}px + {}px)",
                        start_width_pct,
                        margin.get().0,
                        dw
                    ),
                    format!("{}px", start_height_px + dh),
                )
            } else {
                (
                    format!("{}%", item.x as f32 * col_width_pct()),
                    format!(
                        "{}px",
                        item.y as f32 * (row_height.get() + margin.get().1 as f32)
                    ),
                    format!(
                        "calc({}% - {}px)",
                        item.w as f32 * col_width_pct(),
                        margin.get().0
                    ),
                    format!(
                        "{}px",
                        item.h as f32 * row_height.get()
                            + (item.h as f32 - 1.0) * margin.get().1 as f32
                    ),
                )
            };

        let transform = if is_active.get() {
            "scale(1.025) translate3d(0,0,0)"
        } else {
            "scale(1) translate3d(0,0,0)"
        };
        let z = if is_active.get() { 100 } else { 0 };

        format!(
            "position: absolute; left: {}; top: {}; width: {}; height: {}; z-index: {}; pointer-events: auto; transform: {}; transition: transform 0.15s ease-out; touch-action: none; user-select: none;",
            left_str, top_str, width_str, height_str, z, transform
        )
    };

    let item_id_for_kb = item.id.clone();
    let handles_view = resize_handles.into_iter().map(|handle| {
        let on_resize_start = on_resize_start.clone();
        let handle_style = resize_handle_style(handle);
        let aria = resize_handle_aria_label(handle);
        view! {
            <div
                class="resize-handle"
                style=format!("position: absolute; touch-action: none; z-index: 20; {}", handle_style)
                tabindex="0"
                role="button"
                aria-label=aria
                on:pointerdown=move |e: PointerEvent| {
                    e.stop_propagation();
                    on_resize_start(e, handle);
                }
            >
                {if handle == ResizeHandle::SouthEast {
                    view! {
                        <svg width="14" height="14" viewBox="0 0 12 12" style="opacity: 0.4; pointer-events: none;">
                            <path d="M10 2 L10 10 L2 10 Z" fill="currentColor"/>
                        </svg>
                    }.into_view()
                } else {
                    view! { <div></div> }.into_view()
                }}
            </div>
        }
    }).collect::<Vec<_>>();

    let aria_widget = format!("Widget {}, draggable grid item", item.id);

    view! {
        <div class="grid-item" style=style data-active=move || is_active.get().to_string()>
            <div
                class="grid-item-inner"
                style="width: 100%; height: 100%; position: relative;"
                tabindex="0"
                role="group"
                aria-label=aria_widget.clone()
                aria-grabbed=move || if is_active.get() { "true" } else { "false" }
                on:pointerdown=on_drag_start
                on:keydown=move |ev: ev::KeyboardEvent| {
                    if !keyboard_cell_nudge || pointer_interaction.get() {
                        return;
                    }
                    let (dx, dy) = match ev.key().as_str() {
                        "ArrowLeft" => (-1, 0),
                        "ArrowRight" => (1, 0),
                        "ArrowUp" => (0, -1),
                        "ArrowDown" => (0, 1),
                        _ => return,
                    };
                    ev.prevent_default();
                    ev.stop_propagation();
                    leptos_apply_keyboard_cell_nudge(
                        layout,
                        cols.get(),
                        compaction.get(),
                        &item_id_for_kb,
                        dx,
                        dy,
                    );
                }
            >
                {render_item(item.clone())}
            </div>
            {handles_view}
        </div>
    }
}

fn resize_handle_style(handle: ResizeHandle) -> String {
    match handle {
        ResizeHandle::SouthEast =>
            "bottom: -8px; right: -8px; cursor: nwse-resize; width: 40px; height: 40px; display: flex; align-items: flex-end; justify-content: flex-end; padding: 12px;".into(),
        ResizeHandle::South =>
            "bottom: -8px; left: 30px; right: 30px; height: 16px; cursor: ns-resize; display: flex; justify-content: center; align-items: center;".into(),
        ResizeHandle::East =>
            "top: 30px; bottom: 30px; right: -8px; width: 16px; cursor: ew-resize; display: flex; align-items: center; justify-content: center;".into(),
        _ => "".into(),
    }
}
