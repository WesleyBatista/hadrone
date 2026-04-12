use gloo_events::EventListener;
use hadrone_core::interaction::{InteractionSession, InteractionType};
use hadrone_core::{
    resize_handle_aria_label, CollisionStrategy, CompactionType, Compactor, FreePlacementCompactor,
    LayoutEngine, LayoutItem, ResizeHandle, RisingTideCompactor,
};

pub use hadrone_core::{InteractionPhase, LayoutEvent};
use web_sys::KeyboardEvent;
use yew::prelude::*;

fn yew_apply_keyboard_cell_nudge(
    layout: &UseStateHandle<Vec<LayoutItem>>,
    cols: i32,
    compaction: CompactionType,
    item_id: &str,
    dx: i32,
    dy: i32,
) {
    let mut l = (**layout).clone();
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

#[derive(Clone, PartialEq, Properties)]
pub struct GridLayoutProps {
    pub layout: UseStateHandle<Vec<LayoutItem>>,
    pub cols: i32,
    pub row_height: f32,
    pub margin: (i32, i32),
    pub compaction: CompactionType,
    #[prop_or_default]
    pub keyboard_cell_nudge: bool,
    pub render_item: Callback<LayoutItem, Html>,
}

#[function_component(GridLayout)]
pub fn grid_layout(props: &GridLayoutProps) -> Html {
    let active = use_state(|| None::<InteractionSession>);
    let visual_delta = use_state(|| None::<(f32, f32, f32, f32)>);
    let container_ref = use_node_ref();
    let container_width = use_state(|| 1200.0f32);

    {
        let container_ref = container_ref.clone();
        let container_width = container_width.clone();
        use_effect_with((), move |_| {
            let update_width = {
                let container_ref = container_ref.clone();
                let container_width = container_width.clone();
                move || {
                    if let Some(el) = container_ref.cast::<web_sys::HtmlElement>() {
                        container_width.set(el.client_width() as f32);
                    }
                }
            };

            update_width();
            let listener = EventListener::new(
                &web_sys::window().expect("browser window"),
                "resize",
                move |_| {
                    update_width();
                },
            );

            move || drop(listener)
        });
    }

    // Auto compaction effect
    {
        let layout = props.layout.clone();
        let cols = props.cols;
        let ctype = props.compaction;
        let active_state = active.clone();

        use_effect_with(
            (cols, ctype, layout.clone()),
            move |(cols, ctype, layout)| {
                if active_state.is_none() {
                    let mut l = (**layout).clone();
                    let compactor: Box<dyn Compactor> = match *ctype {
                        CompactionType::Gravity => Box::new(RisingTideCompactor),
                        CompactionType::FreePlacement => Box::new(FreePlacementCompactor),
                    };
                    let engine = LayoutEngine::with_default_collision(compactor, *cols);
                    for item in l.iter_mut() {
                        if !item.is_static {
                            item.w = item.w.min(*cols);
                            item.x = item.x.max(0).min(*cols - item.w);
                        }
                    }
                    let old_layout = (**layout).clone();
                    engine.compact(&mut l);
                    if l != old_layout {
                        layout.set(l);
                    }
                }
                || ()
            },
        );
    }

    let mut max_y = 0;
    for item in (*props.layout).iter() {
        max_y = max_y.max(item.y + item.h);
    }
    let total_height = if active.is_some() {
        500.0
    } else {
        (max_y as f32 * (props.row_height + props.margin.1 as f32)).max(500.0)
    };

    let onpointermove = {
        let active = active.clone();
        let visual_delta = visual_delta.clone();
        let layout = props.layout.clone();
        let cols = props.cols;

        Callback::from(move |e: PointerEvent| {
            if let Some(interaction) = (*active).as_ref() {
                let coords = (e.client_x() as f32, e.client_y() as f32);
                visual_delta.set(Some(interaction.get_visual_delta(coords)));
                let mut new_layout = (*layout).clone();
                interaction.update(coords, &mut new_layout, cols);
                layout.set(new_layout);
            }
        })
    };

    let onpointerup = {
        let active = active.clone();
        let visual_delta = visual_delta.clone();

        Callback::from(move |e: PointerEvent| {
            if active.is_some() {
                let _pid = e.pointer_id();
                #[cfg(target_arch = "wasm32")]
                if let Some(el) = web_sys::window().and_then(|w| w.document()).and_then(|d| {
                    d.query_selector(".hadrone-container[data-active='true']")
                        .ok()
                        .flatten()
                }) {
                    let _ = el.release_pointer_capture(_pid);
                }
                active.set(None);
                visual_delta.set(None);
            }
        })
    };

    let style = format!(
        "position: relative; width: 100%; height: {}px; contain: layout; touch-action: none; user-select: none;",
        total_height
    );
    let container_class = "hadrone-container";

    html! {
        <div ref={container_ref} class={container_class} style={style}
             data-active={active.is_some().to_string()}
             role="application"
             aria-label="Draggable grid layout. Use Tab to reach widgets and resize handles. Arrow keys move the focused widget when keyboard nudge is enabled."
             onpointermove={onpointermove}
             onpointerup={onpointerup.clone()}
             onpointerleave={onpointerup.clone()}
             onpointercancel={onpointerup}
        >
            <style>
                { "
                .resize-handle { opacity: 0; pointer-events: none; transition: opacity 0.15s ease-in-out; }
                .grid-item:hover .resize-handle { opacity: 1; pointer-events: auto; }
                .hadrone-container[data-active=\"true\"] { cursor: grabbing !important; }
                .hadrone-container[data-active=\"true\"] .grid-item:not([data-active=\"true\"]) .resize-handle { opacity: 0 !important; pointer-events: none !important; }
                .grid-item[data-active=\"true\"] .resize-handle { opacity: 1 !important; pointer-events: auto !important; }
                .grid-item-inner:focus-visible { outline: 2px solid #2563eb; outline-offset: 2px; }
                .resize-handle:focus-visible { opacity: 1 !important; pointer-events: auto !important; outline: 2px solid #2563eb; outline-offset: 2px; }
                " }
            </style>

            { for (*props.layout).iter().map(|item| {
                let item_id = item.id.clone();
                let is_active = active.as_ref().is_some_and(|a| a.id == item_id);
                let start_rect = if is_active { active.as_ref().map(|a| a.start_rect) } else { None };
                let current_visual_delta = if is_active { *visual_delta } else { None };

                let render_cb = props.render_item.clone();
                let rendered_item = render_cb.emit(item.clone());
                let resize_handles: Vec<ResizeHandle> = item
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

                html! {
                    <GridItem
                        item={item.clone()}
                        cols={props.cols}
                        row_height={props.row_height}
                        margin={props.margin}
                        compaction={props.compaction}
                        keyboard_cell_nudge={props.keyboard_cell_nudge}
                        is_active={is_active}
                        start_rect={start_rect}
                        visual_delta={current_visual_delta}
                        active_state={active.clone()}
                        visual_delta_state={visual_delta.clone()}
                        layout={props.layout.clone()}
                        resize_handles={resize_handles}
                        container_width={*container_width}
                    >
                        { rendered_item }
                    </GridItem>
                }
            })}
        </div>
    }
}

#[derive(Clone, PartialEq, Properties)]
pub struct GridItemProps {
    pub item: LayoutItem,
    pub cols: i32,
    pub row_height: f32,
    pub margin: (i32, i32),
    pub compaction: CompactionType,
    pub keyboard_cell_nudge: bool,
    pub is_active: bool,
    pub start_rect: Option<(i32, i32, i32, i32)>,
    pub visual_delta: Option<(f32, f32, f32, f32)>,
    pub children: Children,
    pub active_state: UseStateHandle<Option<InteractionSession>>,
    pub visual_delta_state: UseStateHandle<Option<(f32, f32, f32, f32)>>,
    pub layout: UseStateHandle<Vec<LayoutItem>>,
    pub resize_handles: Vec<ResizeHandle>,
    pub container_width: f32,
}

#[function_component(GridItem)]
pub fn grid_item(props: &GridItemProps) -> Html {
    let col_width_pct = 100.0 / props.cols as f32;

    let (left_str, top_str, width_str, height_str) =
        if let (Some((dx, dy, dw, dh)), Some(sr)) = (props.visual_delta, props.start_rect) {
            let start_left_pct = sr.0 as f32 * col_width_pct;
            let start_top_px = sr.1 as f32 * (props.row_height + props.margin.1 as f32);
            let start_width_pct = sr.2 as f32 * col_width_pct;
            let start_height_px =
                sr.3 as f32 * props.row_height + (sr.3 as f32 - 1.0) * props.margin.1 as f32;
            (
                format!("calc({}% + {}px)", start_left_pct, dx),
                format!("{}px", start_top_px + dy),
                format!(
                    "calc({}% - {}px + {}px)",
                    start_width_pct, props.margin.0, dw
                ),
                format!("{}px", start_height_px + dh),
            )
        } else {
            (
                format!("{}%", props.item.x as f32 * col_width_pct),
                format!(
                    "{}px",
                    props.item.y as f32 * (props.row_height + props.margin.1 as f32)
                ),
                format!(
                    "calc({}% - {}px)",
                    props.item.w as f32 * col_width_pct,
                    props.margin.0
                ),
                format!(
                    "{}px",
                    props.item.h as f32 * props.row_height
                        + (props.item.h as f32 - 1.0) * props.margin.1 as f32
                ),
            )
        };

    let transform = if props.is_active {
        "scale(1.025) translate3d(0,0,0)"
    } else {
        "scale(1) translate3d(0,0,0)"
    };
    let z = if props.is_active { 100 } else { 0 };

    let style = format!(
        "position: absolute; left: {}; top: {}; width: {}; height: {}; z-index: {}; pointer-events: auto; transform: {}; transition: transform 0.15s ease-out; touch-action: none; user-select: none;",
        left_str, top_str, width_str, height_str, z, transform
    );

    // --- Drag handler ---
    let onpointerdown = {
        let active_state = props.active_state.clone();
        let visual_delta_state = props.visual_delta_state.clone();
        let item = props.item.clone();
        let cols = props.cols;
        let row_height = props.row_height;
        let margin = props.margin;
        let compaction = props.compaction;
        let cow = props.container_width;

        Callback::from(move |e: PointerEvent| {
            if !item.can_drag() {
                return;
            }
            let _pid = e.pointer_id();
            #[cfg(target_arch = "wasm32")]
            if let Some(el) = web_sys::window()
                .and_then(|w| w.document())
                .and_then(|d| d.query_selector(".hadrone-container").ok().flatten())
            {
                let _ = el.set_pointer_capture(_pid);
            }

            let start_mouse = (e.client_x() as f32, e.client_y() as f32);
            let session = InteractionSession {
                id: item.id.clone(),
                start_mouse,
                start_rect: (item.x, item.y, item.w, item.h),
                interaction_type: InteractionType::Drag,
                handle: ResizeHandle::SouthEast,
                col_width_px: cow / cols as f32,
                row_height_px: row_height,
                margin,
                container_padding: (0, 0),
                compaction,
                collision: CollisionStrategy::PushDown,
            };
            visual_delta_state.set(Some(session.get_visual_delta(start_mouse)));
            active_state.set(Some(session));
        })
    };

    let on_keydown = {
        let layout = props.layout.clone();
        let item_id = props.item.id.clone();
        let cols = props.cols;
        let compaction = props.compaction;
        let keyboard_cell_nudge = props.keyboard_cell_nudge;
        let active_state = props.active_state.clone();
        Callback::from(move |e: KeyboardEvent| {
            if !keyboard_cell_nudge || (*active_state).is_some() {
                return;
            }
            let (dx, dy) = match e.key().as_str() {
                "ArrowLeft" => (-1, 0),
                "ArrowRight" => (1, 0),
                "ArrowUp" => (0, -1),
                "ArrowDown" => (0, 1),
                _ => return,
            };
            e.prevent_default();
            e.stop_propagation();
            yew_apply_keyboard_cell_nudge(&layout, cols, compaction, &item_id, dx, dy);
        })
    };

    let aria_widget = format!("Widget {}, draggable grid item", props.item.id);
    let aria_grabbed = if props.is_active { "true" } else { "false" };

    // --- Resize handles ---
    let resize_handles_html: Vec<Html> = props.resize_handles.iter().map(|&handle| {
        let active_state = props.active_state.clone();
        let visual_delta_state = props.visual_delta_state.clone();
        let item = props.item.clone();
        let cols = props.cols;
        let row_height = props.row_height;
        let margin = props.margin;
        let compaction = props.compaction;
        let cow = props.container_width;

        let handle_style = match handle {
            ResizeHandle::SouthEast =>
                "bottom: -8px; right: -8px; cursor: nwse-resize; width: 40px; height: 40px; display: flex; align-items: flex-end; justify-content: flex-end; padding: 12px;",
            ResizeHandle::South =>
                "bottom: -8px; left: 30px; right: 30px; height: 16px; cursor: ns-resize; display: flex; justify-content: center; align-items: center;",
            ResizeHandle::East =>
                "top: 30px; bottom: 30px; right: -8px; width: 16px; cursor: ew-resize; display: flex; align-items: center; justify-content: center;",
            _ => "display: none;",
        };

        let aria_label = resize_handle_aria_label(handle);

        let on_resize_down = Callback::from(move |e: PointerEvent| {
            if !item.can_resize() {
                return;
            }
            let _pid = e.pointer_id();
            #[cfg(target_arch = "wasm32")]
            if let Some(el) = web_sys::window()
                .and_then(|w| w.document())
                .and_then(|d| d.query_selector(".hadrone-container").ok().flatten())
            {
                let _ = el.set_pointer_capture(_pid);
            }

            e.stop_propagation();
            let start_mouse = (e.client_x() as f32, e.client_y() as f32);
            let session = InteractionSession {
                id: item.id.clone(),
                start_mouse,
                start_rect: (item.x, item.y, item.w, item.h),
                interaction_type: InteractionType::Resize,
                handle,
                col_width_px: cow / cols as f32,
                row_height_px: row_height,
                margin,
                container_padding: (0, 0),
                compaction,
                collision: CollisionStrategy::PushDown,
            };
            visual_delta_state.set(Some(session.get_visual_delta(start_mouse)));
            active_state.set(Some(session));
        });

        html! {
            <div
                class="resize-handle"
                style={format!("position: absolute; touch-action: none; z-index: 20; {}", handle_style)}
                tabindex="0"
                role="button"
                aria-label={aria_label}
                onpointerdown={on_resize_down}
            >
                {if handle == ResizeHandle::SouthEast {
                    html! {
                        <svg width="14" height="14" viewBox="0 0 12 12" style="opacity: 0.4; pointer-events: none;">
                            <path d="M10 2 L10 10 L2 10 Z" fill="currentColor"/>
                        </svg>
                    }
                } else {
                    html! { <div style="width: 40px; height: 4px; background: #94a3b8; border-radius: 2px;"></div> }
                }}
            </div>
        }
    }).collect();

    html! {
        <div class="grid-item" style={style} data-active={props.is_active.to_string()}>
            <div
                class="grid-item-inner"
                style="width: 100%; height: 100%; position: relative;"
                tabindex="0"
                role="group"
                aria-label={aria_widget}
                aria-grabbed={aria_grabbed}
                onpointerdown={onpointerdown}
                onkeydown={on_keydown}
            >
                { for props.children.iter() }
            </div>
            { for resize_handles_html }
        </div>
    }
}
