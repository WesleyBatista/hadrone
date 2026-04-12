use crate::{CollisionStrategy, CompactionType, LayoutItem, ResizeHandle, layout_engine};

/// The type of user interaction currently active.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum InteractionType {
    /// Moving an item across the grid.
    Drag,
    /// Changing an item's dimensions.
    Resize,
}

/// Captures the state of an active drag or resize operation.
/// This struct handles the high-frequency sub-pixel tracking and
/// periodic grid snapping logic.
#[derive(Debug, Clone, PartialEq)]
pub struct InteractionSession {
    /// ID of the item being interacted with.
    pub id: String,
    /// Whether this is a drag or resize.
    pub interaction_type: InteractionType,
    /// Initial mouse coordinates (X, Y) when the interaction started.
    pub start_mouse: (f32, f32),
    /// Initial position and dimensions (X, Y, W, H) in grid units.
    pub start_rect: (i32, i32, i32, i32),
    /// The resize handle being used (only relevant for Resize).
    pub handle: ResizeHandle,
    /// Current rendered width of a single column in pixels.
    pub col_width_px: f32,
    /// Current rendered height of a single row in pixels.
    pub row_height_px: f32,
    /// Margin between items (X, Y) in pixels.
    pub margin: (i32, i32),
    /// Padding inside the container around the grid content (left, top), in px.
    /// Added to [`Self::get_visual_rect`] output; pointer deltas remain unchanged.
    pub container_padding: (i32, i32),
    /// The compaction strategy to apply during the interaction.
    pub compaction: CompactionType,
    /// Collision handling during the interaction (usually [`CollisionStrategy::PushDown`]).
    pub collision: CollisionStrategy,
}

impl InteractionSession {
    /// Returns the raw pixel displacement for smooth rendering
    pub fn get_smooth_offset(&self, current_mouse: (f32, f32)) -> (f32, f32) {
        (
            current_mouse.0 - self.start_mouse.0,
            current_mouse.1 - self.start_mouse.1,
        )
    }

    /// Performs a single-pass interaction update:
    /// Mouse Delta -> Handle-Aware Rect -> Grid Logic
    pub fn update(&self, current_mouse: (f32, f32), layout: &mut Vec<LayoutItem>, cols: i32) {
        let (dx, dy) = self.get_smooth_offset(current_mouse);

        let grid_dx = (dx / self.col_width_px).round() as i32;
        let grid_dy = (dy / (self.row_height_px + self.margin.1 as f32)).round() as i32;

        let engine = layout_engine(self.compaction, self.collision, cols);

        match self.interaction_type {
            InteractionType::Drag => {
                engine.move_element(
                    layout,
                    &self.id,
                    self.start_rect.0 + grid_dx,
                    self.start_rect.1 + grid_dy,
                );
            }
            InteractionType::Resize => {
                let (mut nx, mut ny, mut nw, mut nh) = self.start_rect;

                match self.handle {
                    ResizeHandle::East => {
                        nw += grid_dx;
                    }
                    ResizeHandle::West => {
                        nx += grid_dx;
                        nw -= grid_dx;
                    }
                    ResizeHandle::South => {
                        nh += grid_dy;
                    }
                    ResizeHandle::North => {
                        ny += grid_dy;
                        nh -= grid_dy;
                    }
                    ResizeHandle::SouthEast => {
                        nw += grid_dx;
                        nh += grid_dy;
                    }
                    ResizeHandle::SouthWest => {
                        nx += grid_dx;
                        nw -= grid_dx;
                        nh += grid_dy;
                    }
                    ResizeHandle::NorthEast => {
                        ny += grid_dy;
                        nh -= grid_dy;
                        nw += grid_dx;
                    }
                    ResizeHandle::NorthWest => {
                        nx += grid_dx;
                        nw -= grid_dx;
                        ny += grid_dy;
                        nh -= grid_dy;
                    }
                }

                engine.resize_element(layout, &self.id, nx, ny, nw, nh, Some(self.handle));
            }
        }
    }

    /// Returns the continuous pixel-rect for rendering the active element.
    /// This prevents the 'Snap-Flicker' by using start_rect as the base instead of the current layout.
    pub fn get_visual_rect(&self, current_mouse: (f32, f32)) -> (f32, f32, f32, f32) {
        let (dx, dy) = self.get_smooth_offset(current_mouse);
        let (pad_x, pad_y) = (
            self.container_padding.0 as f32,
            self.container_padding.1 as f32,
        );

        // Convert start grid-rect to pixels
        let mut x = self.start_rect.0 as f32 * self.col_width_px + pad_x;
        let mut y = self.start_rect.1 as f32 * (self.row_height_px + self.margin.1 as f32) + pad_y;
        let mut w = (self.start_rect.2 as f32 * self.col_width_px) - self.margin.0 as f32;
        let mut h = self.start_rect.3 as f32 * self.row_height_px
            + (self.start_rect.3 as f32 - 1.0) * self.margin.1 as f32;

        match self.interaction_type {
            InteractionType::Drag => {
                x += dx;
                y += dy;
            }
            InteractionType::Resize => match self.handle {
                ResizeHandle::East => {
                    w += dx;
                }
                ResizeHandle::West => {
                    x += dx;
                    w -= dx;
                }
                ResizeHandle::South => {
                    h += dy;
                }
                ResizeHandle::North => {
                    y += dy;
                    h -= dy;
                }
                ResizeHandle::SouthEast => {
                    w += dx;
                    h += dy;
                }
                ResizeHandle::SouthWest => {
                    x += dx;
                    w -= dx;
                    h += dy;
                }
                ResizeHandle::NorthEast => {
                    y += dy;
                    h -= dy;
                    w += dx;
                }
                ResizeHandle::NorthWest => {
                    x += dx;
                    w -= dx;
                    y += dy;
                    h -= dy;
                }
            },
        }

        (x, y, w, h)
    }

    /// Returns the pure mouse displacement delta (dx, dy, dw, dh)
    /// to be combined with native CSS calc() for frame-perfect rendering.
    pub fn get_visual_delta(&self, current_mouse: (f32, f32)) -> (f32, f32, f32, f32) {
        let (dx, dy) = self.get_smooth_offset(current_mouse);
        let mut out_dx = 0.0;
        let mut out_dy = 0.0;
        let mut out_dw = 0.0;
        let mut out_dh = 0.0;

        match self.interaction_type {
            InteractionType::Drag => {
                out_dx = dx;
                out_dy = dy;
            }
            InteractionType::Resize => match self.handle {
                ResizeHandle::East => {
                    out_dw = dx;
                }
                ResizeHandle::West => {
                    out_dx = dx;
                    out_dw = -dx;
                }
                ResizeHandle::South => {
                    out_dh = dy;
                }
                ResizeHandle::North => {
                    out_dy = dy;
                    out_dh = -dy;
                }
                ResizeHandle::SouthEast => {
                    out_dw = dx;
                    out_dh = dy;
                }
                ResizeHandle::SouthWest => {
                    out_dx = dx;
                    out_dw = -dx;
                    out_dh = dy;
                }
                ResizeHandle::NorthEast => {
                    out_dy = dy;
                    out_dh = -dy;
                    out_dw = dx;
                }
                ResizeHandle::NorthWest => {
                    out_dx = dx;
                    out_dw = -dx;
                    out_dy = dy;
                    out_dh = -dy;
                }
            },
        }
        (out_dx, out_dy, out_dw, out_dh)
    }
}
