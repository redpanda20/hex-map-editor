//! The hex-map canvas widget: an `iced::widget::shader` program backed by
//! a small custom wgpu renderer.
//!
//! - [`gpu`] - the wgpu adapter (pipelines, textures, vertex geometry).
//! - [`render_target`] - implements `domain::RenderTarget` by turning
//!   layer draw calls into `gpu::DrawCommand`s.
//! - [`state`] - `CanvasState` (cross-frame cache + drag state), and
//!   `Camera` (pan/zoom, and coordinate conversions).
//! - [`program`] - the `shader::Program` impl tying the above together:
//!   per-frame `draw`, input `update`, cursor styling.

mod gpu;
mod program;
mod render_target;
mod state;

use iced::{Color, Element, Length, Task};

use crate::app::Message;
use crate::domain::edit::{BucketFill, EraseTile, PaintTile};
use crate::domain::id::LayerId;
use crate::domain::{HexCoord, Scene, Tool};

/// World-space size (in pixels, before pan/zoom) of one hex,
/// measured center-to-corner.
const HEX_SIZE: f32 = 16.0;

/// Accent color used for the cursor-hover hex outline.
/// TODO: Replace with iced theme.primary derivative
const CURSOR_HEX_COLOR: Color = Color {
    r: 0.35,
    g: 0.55,
    b: 0.95,
    a: 1.0,
};

pub fn canvas_panel<'a>(scene: &'a Scene, tool: Tool) -> Element<'a, Message> {
    let hex_canvas = HexCanvas { scene, tool };

    let element: Element<'_, CanvasEvent> = iced::widget::shader(hex_canvas)
        .width(Length::Fill)
        .height(Length::Fill)
        .into();

    element.map(Message::Canvas)
}

struct HexCanvas<'a> {
    pub scene: &'a Scene,
    pub tool: Tool,
}

#[derive(Debug, Clone, Copy)]
pub enum CanvasEvent {
    PointerPressed { at: HexCoord },
    PointerMoved { from: HexCoord, to: HexCoord },
    PointerReleased,
}

impl CanvasEvent {
    pub fn into_task(self, current_layer: &Option<LayerId>, tool: &Tool) -> Task<Message> {
        // No layer selected, or no tool that cares about this event
        let Some(layer) = *current_layer else {
            return Task::none();
        };

        let message: Message = match (tool, self) {
            (Tool::Paint, CanvasEvent::PointerPressed { at })
            | (Tool::Paint, CanvasEvent::PointerMoved { to: at, .. }) => {
                PaintTile { layer, coord: at }.into()
            }

            (Tool::Erase, CanvasEvent::PointerPressed { at })
            | (Tool::Erase, CanvasEvent::PointerMoved { to: at, .. }) => {
                EraseTile { layer, coord: at }.into()
            }

            (Tool::Fill, CanvasEvent::PointerPressed { at }) => {
                BucketFill { layer, from: at }.into()
            }

            _ => return Task::none(),
        };

        Task::done(message)
    }
}
