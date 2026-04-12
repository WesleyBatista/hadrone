use dioxus::prelude::keyboard_types::Key;
use dioxus::prelude::*;
use hadrone_core::interaction::{InteractionSession, InteractionType};
use hadrone_core::{
    CollisionStrategy, CompactionType, Compactor, FreePlacementCompactor, InteractionPhase,
    LayoutEngine, LayoutEvent, LayoutItem, ResizeHandle, RisingTideCompactor,
    resize_handle_aria_label,
};
use std::time::Duration;

fn apply_keyboard_cell_nudge(
    mut layout: Signal<Vec<LayoutItem>>,
    cols: i32,
    compaction: CompactionType,
    item_id: &str,
    dx: i32,
    dy: i32,
) {
    let mut l = layout.peek().clone();
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
    engine.move_element(&mut l, item_id, nx, ny);
    layout.set(l);
}

#[cfg(target_arch = "wasm32")]
fn wasm_attach_resize_width_observer(el: web_sys::Element, mut width: Signal<f32>) {
    use wasm_bindgen::JsCast;
    use wasm_bindgen::closure::Closure;

    let initial = el.client_width() as f32;
    if initial > 0.0 {
        width.set(initial);
    }

    let el_measure = el.clone();
    let closure = Closure::wrap(Box::new(
        move |_entries: js_sys::Array, _obs: web_sys::ResizeObserver| {
            let w = el_measure.client_width() as f32;
            if w > 0.0 {
                width.set(w);
            }
        },
    )
        as Box<dyn FnMut(js_sys::Array, web_sys::ResizeObserver)>);

    if let Ok(obs) = web_sys::ResizeObserver::new(closure.as_ref().unchecked_ref()) {
        obs.observe(&el);
    }
    closure.forget();
}

/// Configuration for the grid system layout.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct GridConfig {
    /// Number of columns in the grid.
    pub cols: i32,
    /// Height of each row in pixels.
    pub row_height: f32,
    /// Vertical and horizontal margin between items (x, y).
    pub margin: (i32, i32),
    /// Padding inside the container around the grid (matches [`GridLayoutProps::container_padding`]).
    pub container_padding: (i32, i32),
}

/// Props for the [GridLayout] component.
#[derive(Props, Clone, PartialEq)]
#[allow(unpredictable_function_pointer_comparisons)]
pub struct GridLayoutProps {
    /// Reactive signal containing the current arrangement of items.
    pub layout: Signal<Vec<LayoutItem>>,
    /// Fixed number of columns for the grid.
    /// Change this to trigger a re-render and re-compaction of the layout.
    pub cols: i32,
    /// Base height for a single grid row.
    pub row_height: f32,
    /// Spacing between items (horizontal, vertical).
    pub margin: (i32, i32),
    /// Strategy for compacting items (Gravity or FreePlacement).
    pub compaction: CompactionType,
    /// Render function for the visual content of each item.
    pub render_item: fn(LayoutItem) -> Element,
    /// Optional event handler triggered after an item is moved or resized.
    pub on_layout_change: Option<EventHandler<Vec<LayoutItem>>>,
    /// Padding inside the container around the grid content (px), for visual alignment with CSS.
    #[props(default = (0, 0))]
    pub container_padding: (i32, i32),
    /// Collision policy while dragging/resizing.
    #[props(default = CollisionStrategy::PushDown)]
    pub collision_strategy: CollisionStrategy,
    /// Structured lifecycle events (start/stop/update).
    pub on_layout_event: Option<EventHandler<LayoutEvent>>,
    /// When true and `on_layout_event` is set, emit [`InteractionPhase::Update`] on every pointer move.
    #[props(default = false)]
    pub emit_interaction_updates: bool,
    /// When true, arrow keys move the focused widget by one grid cell (focus the widget body first).
    #[props(default = false)]
    pub keyboard_cell_nudge: bool,
}

/// The primary container for the grid layout system.
///
/// This component manages the spatial distribution of [GridItem] children and handles
/// all drag/resize interactions using high-performance pointer capture.
#[component]
pub fn GridLayout(props: GridLayoutProps) -> Element {
    let mut layout = props.layout;
    let compaction = props.compaction;
    let collision_strategy = props.collision_strategy;
    let emit_interaction_updates = props.emit_interaction_updates;
    let on_layout_event = props.on_layout_event;
    let container_pad = props.container_padding;
    let config = GridConfig {
        cols: props.cols,
        row_height: props.row_height,
        margin: props.margin,
        container_padding: container_pad,
    };

    let mut active = use_signal(|| None::<InteractionSession>);
    let mut visual_delta = use_signal(|| None::<(f32, f32, f32, f32)>);
    let container_width = use_signal(|| 1200.0);

    // Style for the container
    let is_active = active.read().is_some();

    // Track container width: ResizeObserver on wasm; polling eval on native/desktop renderers.
    #[cfg(not(target_arch = "wasm32"))]
    use_effect(move || {
        let mut width = container_width;
        spawn(async move {
            loop {
                if let Ok(eval) =
                    document::eval("document.querySelector('.hadrone-container')?.clientWidth")
                        .await
                    && let Some(w) = eval.as_f64()
                {
                    width.set(w as f32);
                }
                tokio::time::sleep(Duration::from_millis(500)).await;
            }
        });
    });

    let total_height = use_memo(move || {
        let max_y = layout
            .read()
            .iter()
            .map(|item| item.y + item.h)
            .max()
            .unwrap_or(0);
        (max_y as f32 * (props.row_height + props.margin.1 as f32)).max(500.0)
    });
    let container_style = format!(
        "position: relative; width: 100%; height: {h}px; contain: layout; touch-action: none; user-select: none; \
         box-sizing: border-box; padding-left: {pad_x}px; padding-top: {pad_y}px; \
         --grid-cols: {cols}; --row-height: {row_height}px; --margin-x: {mx}px; --margin-y: {my}px;",
        h = total_height(),
        pad_x = container_pad.0,
        pad_y = container_pad.1,
        cols = props.cols,
        row_height = props.row_height,
        mx = props.margin.0,
        my = props.margin.1
    );

    // Auto-Compaction on column change or compaction strategy change
    use_effect(move || {
        // Skip reactive compaction during interactions to avoid double-renders
        if active.peek().is_some() {
            return;
        }

        let mut current_layout = layout.peek().clone();
        let compactor: Box<dyn Compactor> = match compaction {
            CompactionType::Gravity => Box::new(RisingTideCompactor),
            CompactionType::FreePlacement => Box::new(FreePlacementCompactor),
        };
        let engine = LayoutEngine::with_default_collision(compactor, props.cols);

        // Ensure no items are wider than the grid
        for item in current_layout.iter_mut() {
            if !item.is_static {
                item.w = item.w.min(props.cols);
                item.x = item.x.max(0).min(props.cols - item.w);
            }
        }

        engine.compact(&mut current_layout);
        layout.set(current_layout);
    });

    // Clone for stable iteration in RSX
    let current_layout = layout.read().clone();
    let interaction_active = active.read().is_some();
    let keyboard_cell_nudge = props.keyboard_cell_nudge;

    rsx! {
        div {
            class: "hadrone-container",
            style: "{container_style}",
            "data-active": "{is_active}",
            role: "application",
            aria_label: "Draggable grid layout. Use Tab to reach widgets and resize handles. Arrow keys move the focused widget when keyboard nudge is enabled.",
            onmounted: move |evt| {
                #[cfg(target_arch = "wasm32")]
                if let Some(el) = evt.data().downcast::<web_sys::Element>() {
                    wasm_attach_resize_width_observer(el.clone(), container_width);
                }
                #[cfg(not(target_arch = "wasm32"))]
                drop(evt);
            },

            // Unified Event Handlers on the Container
            onpointermove: move |e: Event<PointerData>| {
                if let Some(interaction) = active.read().as_ref() {
                    let coords = e.data.client_coordinates();
                    visual_delta.set(Some(interaction.get_visual_delta((coords.x as f32, coords.y as f32))));

                    // --- Optimized Auto-Scroll (calculated in Rust) ---
                    #[cfg(target_arch = "wasm32")]
                    {
                        let y = coords.y as f32;
                        if y < 100.0 {
                            let _ = document::eval("window.scrollBy(0, -10)");
                        } else {
                            let _ = document::eval(&format!(r#"if (window.innerHeight - {} < 100) window.scrollBy(0, 10);"#, y));
                        }
                    }

                    let mut new_layout = layout.peek().clone();
                    interaction.update(
                        (coords.x as f32, coords.y as f32),
                        &mut new_layout,
                        config.cols,
                    );

                    if new_layout != *layout.peek() {
                        layout.set(new_layout);
                    }

                    if emit_interaction_updates
                        && let Some(ref h) = on_layout_event
                        && let Some(interaction) = active.read().as_ref()
                    {
                        h.call(LayoutEvent::Interaction {
                            phase: InteractionPhase::Update,
                            id: interaction.id.clone(),
                            interaction: interaction.interaction_type,
                            layout: layout.peek().clone(),
                            compaction,
                            collision: collision_strategy,
                        });
                    }
                }
            },
            onpointerup: move |e| {
                let ended = active.read().as_ref().cloned();
                if let Some(interaction) = ended {
                    if let Some(ref h) = on_layout_event {
                        h.call(LayoutEvent::Interaction {
                            phase: InteractionPhase::Stop,
                            id: interaction.id.clone(),
                            interaction: interaction.interaction_type,
                            layout: layout.peek().clone(),
                            compaction,
                            collision: collision_strategy,
                        });
                    }
                    let pid = e.data.pointer_id();
                    let _ = document::eval(&format!(r#"
                        const container = document.querySelector(".hadrone-container[data-active='true']");
                        if (container) container.releasePointerCapture({});
                    "#, pid));
                    active.set(None);
                    visual_delta.set(None);
                }
            },
            onpointerleave: move |_| {
                let ended = active.read().as_ref().cloned();
                if let Some(interaction) = ended {
                    if let Some(ref h) = on_layout_event {
                        h.call(LayoutEvent::Interaction {
                            phase: InteractionPhase::Cancel,
                            id: interaction.id.clone(),
                            interaction: interaction.interaction_type,
                            layout: layout.peek().clone(),
                            compaction,
                            collision: collision_strategy,
                        });
                    }
                    active.set(None);
                    visual_delta.set(None);
                }
            },
            onpointercancel: move |_| {
                let ended = active.read().as_ref().cloned();
                if let Some(interaction) = ended {
                    if let Some(ref h) = on_layout_event {
                        h.call(LayoutEvent::Interaction {
                            phase: InteractionPhase::Cancel,
                            id: interaction.id.clone(),
                            interaction: interaction.interaction_type,
                            layout: layout.peek().clone(),
                            compaction,
                            collision: collision_strategy,
                        });
                    }
                    active.set(None);
                    visual_delta.set(None);
                }
            },

            // Global styles for handle hover and transitions
            style {
                r#"
                .resize-handle {{ opacity: 0; pointer-events: none; transition: opacity 0.15s ease-in-out; }}
                .grid-item:hover .resize-handle {{ opacity: 1; pointer-events: auto; }}
                .hadrone-container[data-active="true"] {{ cursor: grabbing !important; }}
                .hadrone-container[data-active="true"] .grid-item:not([data-active="true"]) .resize-handle {{ opacity: 0 !important; pointer-events: none !important; }}
                .grid-item[data-active="true"] .resize-handle {{ opacity: 1 !important; pointer-events: auto !important; }}
                .grid-item-inner:focus-visible {{ outline: 2px solid #2563eb; outline-offset: 2px; }}
                .resize-handle:focus-visible {{ opacity: 1 !important; pointer-events: auto !important; outline: 2px solid #2563eb; outline-offset: 2px; }}
                "#
            }

            for item in current_layout {
                {
                    let item_drag = item.clone();
                    let item_resize = item.clone();

                    let active_ref = active.read();
                    let is_active = active_ref.as_ref().is_some_and(|a| a.id == item.id);

                    rsx! {
                        GridItem {
                            key: "{item.id}",
                            item: item.clone(),
                            config,
                            is_active,
                            start_rect: if is_active { active_ref.as_ref().map(|a| a.start_rect) } else { None },
                            visual_delta: if is_active { visual_delta() } else { None },
                            render_item: props.render_item,
                            layout,
                            keyboard_cell_nudge,
                            compaction,
                            interaction_active,
                            on_drag_start: move |e: Event<PointerData>| {
                                if !item_drag.can_drag() {
                                    return;
                                }
                                let pid = e.data.pointer_id();
                                let _ = document::eval(&format!(r#"
                                    const container = document.querySelector(".hadrone-container");
                                    if (container) container.setPointerCapture({});
                                "#, pid));

                                let start_mouse = (e.data.client_coordinates().x as f32, e.data.client_coordinates().y as f32);
                                let session = InteractionSession {
                                    id: item_drag.id.clone(),
                                    start_mouse,
                                    start_rect: (item_drag.x, item_drag.y, item_drag.w, item_drag.h),
                                    interaction_type: InteractionType::Drag,
                                    handle: ResizeHandle::SouthEast,
                                    col_width_px: container_width() / config.cols as f32,
                                    row_height_px: config.row_height,
                                    margin: config.margin,
                                    container_padding: config.container_padding,
                                    compaction,
                                    collision: collision_strategy,
                                };

                                visual_delta.set(Some(session.get_visual_delta(start_mouse)));
                                active.set(Some(session));
                                if let Some(ref h) = on_layout_event {
                                    h.call(LayoutEvent::Interaction {
                                        phase: InteractionPhase::Start,
                                        id: item_drag.id.clone(),
                                        interaction: InteractionType::Drag,
                                        layout: layout.peek().clone(),
                                        compaction,
                                        collision: collision_strategy,
                                    });
                                }
                            },
                            on_resize_start: move |(e, handle): (Event<PointerData>, ResizeHandle)| {
                                if !item_resize.can_resize() {
                                    return;
                                }
                                let pid = e.data.pointer_id();
                                let _ = document::eval(&format!(r#"
                                    const container = document.querySelector(".hadrone-container");
                                    if (container) container.setPointerCapture({});
                                "#, pid));

                                let start_mouse = (e.data.client_coordinates().x as f32, e.data.client_coordinates().y as f32);
                                let session = InteractionSession {
                                    id: item_resize.id.clone(),
                                    start_mouse,
                                    start_rect: (item_resize.x, item_resize.y, item_resize.w, item_resize.h),
                                    interaction_type: InteractionType::Resize,
                                    handle,
                                    col_width_px: container_width() / config.cols as f32,
                                    row_height_px: config.row_height,
                                    margin: config.margin,
                                    container_padding: config.container_padding,
                                    compaction,
                                    collision: collision_strategy,
                                };

                                visual_delta.set(Some(session.get_visual_delta(start_mouse)));
                                active.set(Some(session));
                                if let Some(ref h) = on_layout_event {
                                    h.call(LayoutEvent::Interaction {
                                        phase: InteractionPhase::Start,
                                        id: item_resize.id.clone(),
                                        interaction: InteractionType::Resize,
                                        layout: layout.peek().clone(),
                                        compaction,
                                        collision: collision_strategy,
                                    });
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

pub type PointerEvent = Event<PointerData>;

#[derive(Props, Clone, PartialEq)]
#[allow(unpredictable_function_pointer_comparisons)]
pub struct GridItemProps {
    pub item: LayoutItem,
    pub config: GridConfig,
    pub is_active: bool,
    pub start_rect: Option<(i32, i32, i32, i32)>,
    pub visual_delta: Option<(f32, f32, f32, f32)>,
    pub layout: Signal<Vec<LayoutItem>>,
    pub keyboard_cell_nudge: bool,
    pub compaction: CompactionType,
    pub interaction_active: bool,
    pub render_item: fn(LayoutItem) -> Element,
    pub on_drag_start: EventHandler<PointerEvent>,
    pub on_resize_start: EventHandler<(PointerEvent, ResizeHandle)>,
}

/// A single interactive unit within the grid.
#[component]
pub fn GridItem(props: GridItemProps) -> Element {
    let item = props.item.clone();
    let item_id = item.id.clone();
    let config = props.config;
    let layout_sig = props.layout;
    let keyboard_cell_nudge = props.keyboard_cell_nudge;
    let compaction = props.compaction;
    let interaction_active = props.interaction_active;

    let mut x_anim = use_animation(item.x as f32, Duration::from_millis(200));
    let mut y_anim = use_animation(item.y as f32, Duration::from_millis(200));
    let mut w_anim = use_animation(item.w as f32, Duration::from_millis(200));
    let mut h_anim = use_animation(item.h as f32, Duration::from_millis(200));

    use_effect(move || {
        x_anim.set(item.x as f32);
        y_anim.set(item.y as f32);
        w_anim.set(item.w as f32);
        h_anim.set(item.h as f32);
    });

    let col_width_pct = 100.0 / config.cols as f32;

    let (left_str, top_str, width_str, height_str) = if let (
        Some((dx, dy, dw, dh)),
        Some(start_rect),
    ) = (props.visual_delta, props.start_rect)
    {
        let start_left_pct = start_rect.0 as f32 * col_width_pct;
        let start_top_px = start_rect.1 as f32 * (config.row_height + config.margin.1 as f32);
        let start_width_pct = start_rect.2 as f32 * col_width_pct;
        let start_height_px = start_rect.3 as f32 * config.row_height
            + (start_rect.3 as f32 - 1.0) * config.margin.1 as f32;

        (
            format!("calc({}% + {}px)", start_left_pct, dx),
            format!("{}px", start_top_px + dy),
            format!(
                "calc({}% - {}px + {}px)",
                start_width_pct, config.margin.0, dw
            ),
            format!("{}px", start_height_px + dh),
        )
    } else {
        (
            format!("{}%", x_anim.value() * col_width_pct),
            format!(
                "{}px",
                y_anim.value() * (config.row_height + config.margin.1 as f32)
            ),
            format!(
                "calc({}% - {}px)",
                w_anim.value() * col_width_pct,
                config.margin.0
            ),
            format!(
                "{}px",
                h_anim.value() * config.row_height
                    + (h_anim.value() - 1.0) * config.margin.1 as f32
            ),
        )
    };

    let transform = if props.is_active {
        "scale(1.025) translate3d(0, 0, 0)"
    } else {
        "scale(1.0) translate3d(0, 0, 0)"
    };

    let style = format!(
        "position: absolute; \
         left: {left_str}; \
         top: {top_str}; \
         width: {width_str}; \
         height: {height_str}; \
         z-index: {z}; \
         pointer-events: auto; \
         transform: {transform}; \
         transition: transform 0.15s ease-out; \
         touch-action: none; \
         user-select: none;",
        z = if props.is_active { 100 } else { 0 }
    );

    let grabbed = if props.is_active { "true" } else { "false" };
    let aria_item = format!("Widget {}, draggable grid item", item.id);

    rsx! {
        div {
            class: "grid-item",
            style: "{style}",
            "data-active": "{props.is_active}",
            div {
                class: "grid-item-inner",
                style: "width: 100%; height: 100%; position: relative;",
                tabindex: 0,
                role: "group",
                aria_label: "{aria_item}",
                aria_grabbed: "{grabbed}",
                onpointerdown: move |e| props.on_drag_start.call(e),
                onkeydown: move |e: Event<KeyboardData>| {
                    if !keyboard_cell_nudge || interaction_active {
                        return;
                    }
                    let (dx, dy) = match e.key() {
                        Key::ArrowLeft => (-1, 0),
                        Key::ArrowRight => (1, 0),
                        Key::ArrowUp => (0, -1),
                        Key::ArrowDown => (0, 1),
                        _ => return,
                    };
                    e.prevent_default();
                    e.stop_propagation();
                    apply_keyboard_cell_nudge(
                        layout_sig,
                        config.cols,
                        compaction,
                        &item_id,
                        dx,
                        dy,
                    );
                },

                { (props.render_item)(item.clone()) }
            }
            for handle in item
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
            {
                ResizeHandleComponent {
                    handle,
                    is_active: props.is_active,
                    on_pointerdown: move |e| props.on_resize_start.call((e, handle)),
                }
            }
        }
    }
}

#[component]
fn ResizeHandleComponent(
    handle: ResizeHandle,
    is_active: bool,
    on_pointerdown: EventHandler<PointerEvent>,
) -> Element {
    let (style, content, z) = match handle {
        ResizeHandle::SouthEast => (
            "bottom: -8px; right: -8px; cursor: nwse-resize; width: 40px; height: 40px; display: flex; align-items: flex-end; justify-content: flex-end; padding: 12px;",
            rsx! {
                svg {
                    width: "14",
                    height: "14",
                    view_box: "0 0 12 12",
                    style: "opacity: 0.2; pointer-events: none;",
                    path { d: "M10 2 L10 10 L2 10 Z", fill: "currentColor" }
                }
            },
            20,
        ),
        ResizeHandle::South => (
            "bottom: -8px; left: 10px; right: 30px; height: 16px; cursor: ns-resize; display: flex; justify-content: center; align-items: center;",
            rsx! { div { style: "width: 40px; height: 4px; background: transparent; border-radius: 2px;" } },
            10,
        ),
        ResizeHandle::East => (
            "top: 10px; bottom: 30px; right: -8px; width: 16px; cursor: ew-resize; display: flex; align-items: center; justify-content: center;",
            rsx! { div { style: "width: 4px; height: 40px; background: transparent; border-radius: 2px;" } },
            10,
        ),
        _ => return rsx! {},
    };

    let active_style = if is_active {
        "opacity: 1 !important; pointer-events: auto !important;"
    } else {
        ""
    };
    let label = resize_handle_aria_label(handle);

    rsx! {
        div {
            class: "resize-handle",
            style: "position: absolute; {style}; touch-action: none; z-index: {z}; {active_style}",
            tabindex: 0,
            role: "button",
            aria_label: "{label}",
            onpointerdown: move |e| on_pointerdown.call(e),
            {content}
        }
    }
}

fn use_animation(target: f32, _duration: std::time::Duration) -> Animation {
    let mut value = use_signal(|| target);
    let mut last_target = use_signal(|| target);

    if target != *last_target.read() {
        value.set(target);
        last_target.set(target);
    }

    Animation { value }
}

#[derive(Clone, Copy)]
struct Animation {
    value: Signal<f32>,
}

impl Animation {
    fn value(&self) -> f32 {
        *self.value.read()
    }
    fn set(&mut self, target: f32) {
        self.value.set(target)
    }
}
