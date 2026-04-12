use hadrone_core::{CompactionType, LayoutItem};
use hadrone_yew::GridLayout;
use yew::prelude::*;

#[function_component(App)]
fn app() -> Html {
    let layout = use_state(|| {
        vec![
            LayoutItem {
                id: "1".into(),
                x: 0,
                y: 0,
                w: 4,
                h: 2,
                ..Default::default()
            },
            LayoutItem {
                id: "2".into(),
                x: 4,
                y: 0,
                w: 4,
                h: 4,
                ..Default::default()
            },
            LayoutItem {
                id: "3".into(),
                x: 2,
                y: 2,
                w: 2,
                h: 2,
                ..Default::default()
            },
        ]
    });

    let render_item = Callback::from(|item: LayoutItem| {
        html! {
            <div style="width: 100%; height: 100%; display: flex; flex-direction: column; background: white; border: 1px solid #e2e8f0; border-radius: 12px; overflow: hidden; box-shadow: 0 4px 6px -1px rgb(0 0 0 / 0.1);">
                <div style="padding: 12px 16px; background: #f8fafc; border-bottom: 1px solid #e2e8f0; cursor: grab; font-weight: 800; color: #1e293b; font-size: 13px; display: flex; justify-content: space-between; align-items: center;">
                    <span>{ format!("WIDGET {}", item.id.to_uppercase()) }</span>
                </div>
                <div style="flex: 1; padding: 16px; display: flex; flex-direction: column; gap: 12px;">
                    <div style="display: grid; grid-template-columns: 1fr 1fr; gap: 8px; font-family: ui-monospace, monospace; font-size: 11px; color: #64748b;">
                        <div>{ format!("LOC: {},{}", item.x, item.y) }</div>
                        <div>{ format!("DIM: {}x{}", item.w, item.h) }</div>
                    </div>
                    <div style="flex: 1; min-height: 0; background: #f1f5f9; border: 1px solid #e2e8f0; border-radius: 8px; display: flex; align-items: center; justify-content: center; color: #94a3b8; font-size: 11px; font-weight: 700; text-transform: uppercase;">
                        { "Widget Content" }
                    </div>
                </div>
            </div>
        }
    });

    html! {
        <div style="padding: 24px; background: #f1f5f9; min-height: 100vh; font-family: system-ui, sans-serif; color: #1e293b;">
            <header style="margin-bottom: 32px; background: white; padding: 20px; border-radius: 12px; box-shadow: 0 1px 3px rgba(0,0,0,0.1); display: flex; justify-content: space-between; align-items: center;">
                <div>
                    <h1 style="margin: 0; font-size: 22px; font-weight: 800;">{ "Grid Engine Debugger" }</h1>
                    <div style="font-size: 12px; color: #64748b; margin-top: 4px;">{ "Platform: Yew • Cols: 12" }</div>
                </div>
                <div style="display: flex; gap: 10px;">
                    <button
                        style="padding: 8px 16px; background: #3b82f6; color: white; border: none; border-radius: 8px; font-weight: 600; cursor: pointer;"
                        onclick={
                            let layout = layout.clone();
                            Callback::from(move |_| {
                                let mut l = (*layout).clone();
                                let new_id = format!("w_{}", l.len() + 1);
                                l.push(LayoutItem { id: new_id, x: 0, y: 99, w: 4, h: 2, ..Default::default() });
                                layout.set(l);
                            })
                        }
                    >
                        { "Add Widget" }
                    </button>
                    <button
                        style="padding: 8px 16px; background: white; color: #ef4444; border: 1px solid #ef4444; border-radius: 8px; font-weight: 600; cursor: pointer;"
                        onclick={
                            let layout = layout.clone();
                            Callback::from(move |_| layout.set(vec![]))
                        }
                    >
                        { "Clear" }
                    </button>
                </div>
            </header>
            <div style="background: #fff; border-radius: 16px; padding: 20px; box-shadow: inset 0 2px 4px rgba(0,0,0,0.05); min-height: 600px;">
                <GridLayout
                    layout={layout.clone()}
                    cols={12}
                    row_height={100.0}
                    margin={(20, 20)}
                    compaction={CompactionType::Gravity}
                    keyboard_cell_nudge={true}
                    render_item={render_item}
                />
            </div>
        </div>
    }
}

fn main() {
    yew::Renderer::<App>::new().render();
}
