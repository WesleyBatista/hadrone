use hadrone_core::interaction::{InteractionSession, InteractionType};
use hadrone_core::{
    CollisionStrategy, CompactionType, FreePlacementCompactor, LayoutEngine, LayoutItem,
    ResizeHandle, RisingTideCompactor,
};
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub struct GridEngineWasm {
    layout: Vec<LayoutItem>,
    cols: i32,
    compaction: CompactionType,
    session: Option<InteractionSession>,
}

#[wasm_bindgen]
impl GridEngineWasm {
    #[wasm_bindgen(constructor)]
    pub fn new(cols: i32, compaction_str: &str) -> Self {
        let compaction = match compaction_str {
            "FreePlacement" => CompactionType::FreePlacement,
            _ => CompactionType::Gravity,
        };
        Self {
            layout: Vec::new(),
            cols,
            compaction,
            session: None,
        }
    }

    #[wasm_bindgen]
    pub fn set_layout(&mut self, items: JsValue) -> Result<(), JsValue> {
        self.layout = serde_wasm_bindgen::from_value(items)?;
        self.compact();
        Ok(())
    }

    #[wasm_bindgen]
    pub fn get_layout(&self) -> Result<JsValue, JsValue> {
        serde_wasm_bindgen::to_value(&self.layout).map_err(|e| e.into())
    }

    fn compact(&mut self) {
        let compactor: Box<dyn hadrone_core::Compactor> = match self.compaction {
            CompactionType::Gravity => Box::new(RisingTideCompactor),
            CompactionType::FreePlacement => Box::new(FreePlacementCompactor),
        };
        let engine = LayoutEngine::with_default_collision(compactor, self.cols);
        engine.compact(&mut self.layout);
    }

    #[allow(clippy::too_many_arguments)]
    #[wasm_bindgen]
    pub fn start_interaction(
        &mut self,
        id: &str,
        interaction_type: &str,
        handle_str: &str,
        mouse_x: f32,
        mouse_y: f32,
        col_width_px: f32,
        row_height_px: f32,
        margin_x: i32,
        margin_y: i32,
        padding_x: i32,
        padding_y: i32,
    ) {
        if let Some(item) = self.layout.iter().find(|i| i.id == id) {
            let i_type = match interaction_type {
                "Resize" => InteractionType::Resize,
                _ => InteractionType::Drag,
            };

            let handle = match handle_str {
                "East" => ResizeHandle::East,
                "West" => ResizeHandle::West,
                "North" => ResizeHandle::North,
                "South" => ResizeHandle::South,
                "NorthEast" => ResizeHandle::NorthEast,
                "NorthWest" => ResizeHandle::NorthWest,
                "SouthEast" => ResizeHandle::SouthEast,
                "SouthWest" => ResizeHandle::SouthWest,
                _ => ResizeHandle::SouthEast,
            };

            self.session = Some(InteractionSession {
                id: id.to_string(),
                interaction_type: i_type,
                start_mouse: (mouse_x, mouse_y),
                start_rect: (item.x, item.y, item.w, item.h),
                handle,
                col_width_px,
                row_height_px,
                margin: (margin_x, margin_y),
                container_padding: (padding_x, padding_y),
                compaction: self.compaction,
                collision: CollisionStrategy::PushDown,
            });
        }
    }

    #[wasm_bindgen]
    pub fn update_interaction(&mut self, mouse_x: f32, mouse_y: f32) -> Result<JsValue, JsValue> {
        if let Some(session) = &self.session {
            let mut virtual_layout = self.layout.clone();
            session.update((mouse_x, mouse_y), &mut virtual_layout, self.cols);

            // Return updated layout while interacting
            serde_wasm_bindgen::to_value(&virtual_layout).map_err(|e| e.into())
        } else {
            self.get_layout()
        }
    }

    #[wasm_bindgen]
    pub fn end_interaction(&mut self, mouse_x: f32, mouse_y: f32) {
        if let Some(session) = self.session.take() {
            session.update((mouse_x, mouse_y), &mut self.layout, self.cols);
        }
    }
}
