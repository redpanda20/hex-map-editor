mod gpu;

use std::cell::{Cell, RefCell};
use std::sync::Arc;

use iced::{
    Color, Element, Event, Length, Point, Rectangle, Task, Vector, mouse, touch,
    widget::{Action, shader},
};

use gpu::{DrawCommand, HexMapPrimitive, push_hex_fill, push_hex_stroke, quad_vertices};

#[derive(Debug, Clone, Copy)]
pub enum CanvasEvent {
    PointerPressed { at: HexCoord },
    PointerMoved { from: HexCoord, to: HexCoord },
    PointerReleased,
}

impl CanvasEvent {
    pub fn into_task(self, current_layer: &Option<LayerId>, tool: &Tool) -> Task<Message> {
        let command: Message = match (*current_layer, tool, self) {
            (Some(layer), Tool::Paint, CanvasEvent::PointerPressed { at }) => {
                PaintTile { layer, coord: at }.into()
            }

            (Some(layer), Tool::Paint, CanvasEvent::PointerMoved { from: _, to }) => {
                PaintTile { layer, coord: to }.into()
            }

            (Some(layer), Tool::Erase, CanvasEvent::PointerPressed { at }) => {
                EraseTile { layer, coord: at }.into()
            }

            (Some(layer), Tool::Erase, CanvasEvent::PointerMoved { from: _, to }) => {
                EraseTile { layer, coord: to }.into()
            }

            (Some(layer), Tool::Fill, CanvasEvent::PointerPressed { at }) => {
                BucketFill { layer, from: at }.into()
            }

            _ => return Task::none(),
        };
        Task::done(command)
    }
}

use crate::domain::layer::LayerInnerImpl;
use crate::{
    app::Message,
    domain::{
        HexCoord, RenderTarget, Scene, Tool,
        assets::AssetStore,
        edit::{BucketFill, EraseTile, PaintTile},
        id::{ImageId, LayerId},
        layer::overlay::HexGridOverlay,
    },
};

const HEX_SIZE: f32 = 16.0;

/// Accent color used for the cursor-hover hex outline.
///
/// The old `canvas::Program::draw` received a `&Theme` and pulled
/// `theme.extended_palette().primary.base.color` from it; `shader::Program::draw`
/// does not receive a theme at all, so this is hardcoded for now. If you want
/// this theme-aware again, thread a `Color` into `HexCanvas`/`canvas_panel`
/// from wherever `view()` has access to the active `Theme`.
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

pub struct HexCanvas<'a> {
    pub scene: &'a Scene,
    pub tool: Tool,
}

#[derive(Debug)]
pub struct CanvasState {
    // Cache of the map's draw commands, keyed by `Scene::revision`. Reused
    // (by `Arc` pointer) across frames whenever the scene hasn't changed, so
    // the GPU pipeline can skip re-uploading vertex/texture buffers on pure
    // pan/zoom/hover frames - see `HexMapPipeline::upload`.
    cached_commands: RefCell<Option<Arc<Vec<DrawCommand>>>>,
    cached_layers_revision: Cell<u64>,
    dragging: bool,
    last_drag_pos: Option<Point>,
    translation: Vector,
    zoom: f32,
}

impl Default for CanvasState {
    fn default() -> Self {
        Self {
            cached_commands: RefCell::new(None),
            cached_layers_revision: Cell::new(0),
            dragging: false,
            last_drag_pos: None,
            translation: Vector::new(0.0, 0.0),
            zoom: 1.0,
        }
    }
}

impl<'a> shader::Program<CanvasEvent> for HexCanvas<'a> {
    type State = CanvasState;
    type Primitive = HexMapPrimitive;

    fn draw(
        &self,
        state: &CanvasState,
        cursor: mouse::Cursor,
        bounds: Rectangle,
    ) -> HexMapPrimitive {
        let current_revision = self.scene.revision();

        {
            let mut cached = state.cached_commands.borrow_mut();
            if cached.is_none() || state.cached_layers_revision.get() != current_revision {
                *cached = Some(Arc::new(self.build_base_commands(state, bounds)));
                state.cached_layers_revision.set(current_revision);
            }
        }
        let base = Arc::clone(state.cached_commands.borrow().as_ref().unwrap());

        let overlay = self.build_overlay_commands(state, cursor.position_in(bounds), bounds);

        HexMapPrimitive {
            base,
            overlay: Arc::new(overlay),
            translation: state.translation,
            zoom: state.zoom,
        }
    }

    fn update(
        &self,
        state: &mut CanvasState,
        event: &Event,
        bounds: Rectangle,
        cursor: mouse::Cursor,
    ) -> Option<Action<CanvasEvent>> {
        let Some(cursor_pos) = cursor.position_in(bounds) else {
            state.dragging = false;
            state.last_drag_pos = None;
            return None;
        };

        match event {
            Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left))
            | Event::Touch(touch::Event::FingerPressed { .. }) => {
                state.dragging = true;
                state.last_drag_pos = Some(cursor_pos);

                if self.tool == Tool::Pan {
                    return Some(Action::capture());
                }

                let coord = self.screen_to_hex(state, cursor_pos);
                Some(Action::publish(CanvasEvent::PointerPressed { at: coord }).and_capture())
            }

            Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Left))
            | Event::Touch(touch::Event::FingerLifted { .. })
            | Event::Touch(touch::Event::FingerLost { .. }) => {
                state.dragging = false;
                state.last_drag_pos = None;

                if self.tool == Tool::Pan {
                    return None;
                }

                Some(Action::publish(CanvasEvent::PointerReleased).and_capture())
            }

            Event::Mouse(mouse::Event::CursorMoved { .. })
            | Event::Touch(touch::Event::FingerMoved { .. }) => {
                if !state.dragging {
                    return Some(Action::request_redraw());
                }

                let last = state.last_drag_pos;
                state.last_drag_pos = Some(cursor_pos);
                let last = last?;

                if self.tool == Tool::Pan {
                    state.translation.x += cursor_pos.x - last.x;
                    state.translation.y += cursor_pos.y - last.y;

                    return Some(Action::request_redraw().and_capture());
                }

                let last_coord = self.screen_to_hex(state, last);
                let coord = self.screen_to_hex(state, cursor_pos);

                let message = CanvasEvent::PointerMoved {
                    from: last_coord,
                    to: coord,
                };

                Some(Action::publish(message).and_capture())
            }

            Event::Mouse(mouse::Event::WheelScrolled { delta }) => {
                let delta = match delta {
                    mouse::ScrollDelta::Lines { x, y } => (x + y) * 20.0,
                    mouse::ScrollDelta::Pixels { x, y } => x + y,
                };
                state.zoom = f32::clamp(state.zoom + delta * 0.01, 0.4, 10.0);

                Some(Action::request_redraw().and_capture())
            }

            _ => None,
        }
    }

    fn mouse_interaction(
        &self,
        state: &CanvasState,
        bounds: Rectangle,
        cursor: mouse::Cursor,
    ) -> mouse::Interaction {
        if !cursor.is_over(bounds) {
            return mouse::Interaction::None;
        }

        match self.tool {
            Tool::Pan if state.dragging => mouse::Interaction::Grabbing,
            Tool::Pan => mouse::Interaction::Grab,
            Tool::Paint | Tool::Erase | Tool::Fill => mouse::Interaction::Crosshair,
        }
    }
}

impl<'a> HexCanvas<'a> {
    /// Builds ordered draw commands for every visible layer.
    /// Updated only when `Scene` cache is invalidated.
    fn build_base_commands(&self, state: &CanvasState, bounds: Rectangle) -> Vec<DrawCommand> {
        let inv_scale = 1.0 / HEX_SIZE / state.zoom;
        let relative_bounds =
            Rectangle::with_size(bounds.size()) * inv_scale - state.translation * inv_scale;
        let mut target = GpuRenderTarget::new(relative_bounds, &self.scene.assets);

        for layer in self.scene.get_visible_layers() {
            layer.draw(&mut target);
        }

        target.finish()
    }

    /// Builds draw commands for overlays.
    /// Updated every frame to keep overlay responsive and sharp.
    fn build_overlay_commands(
        &self,
        state: &CanvasState,
        cursor_pos: Option<Point>,
        bounds: Rectangle,
    ) -> Vec<DrawCommand> {
        let inv_scale = 1.0 / HEX_SIZE / state.zoom;
        let relative_bounds =
            Rectangle::with_size(bounds.size()) * inv_scale - state.translation * inv_scale;
        let mut target = GpuRenderTarget::new(relative_bounds, &self.scene.assets);

        // Draw hex grid overlay
        let overlay = HexGridOverlay::new_light(1.5 / state.zoom);
        overlay.draw(&mut target);

        // Draw cursor highlight
        if let Some(cursor_pos) = cursor_pos {
            let hex = self.screen_to_hex(state, cursor_pos);
            let world = hex.to_cartesian() * HEX_SIZE;
            let center = Point::new(world.x, world.y);

            target.stroke_polygon(&center, CURSOR_HEX_COLOR, 2.0 / state.zoom);
        }

        target.finish()
    }

    fn screen_to_hex(&self, state: &CanvasState, screen: Point) -> HexCoord {
        let translation = state.translation;
        let zoom = state.zoom;

        let x = (screen.x - translation.x) / zoom;
        let y = (screen.y - translation.y) / zoom;
        let map_vec = Vector { x, y };

        HexCoord::from_cartesian(map_vec / HEX_SIZE)
    }
}

/// Data that `RenderTarget` is implemented against.
///
/// Batches draw calls to preserve layer order.
struct GpuRenderTarget<'a> {
    bounds: Rectangle,
    assets: &'a AssetStore,
    commands: Vec<DrawCommand>,
    current_mesh: Vec<gpu::MeshVertex>,
}

impl<'a> GpuRenderTarget<'a> {
    fn new(bounds: Rectangle, assets: &'a AssetStore) -> Self {
        Self {
            bounds,
            assets,
            commands: Vec::new(),
            current_mesh: Vec::new(),
        }
    }

    fn flush_mesh(&mut self) {
        if !self.current_mesh.is_empty() {
            self.commands
                .push(DrawCommand::Mesh(std::mem::take(&mut self.current_mesh)));
        }
    }

    fn finish(mut self) -> Vec<DrawCommand> {
        self.flush_mesh();
        self.commands
    }
}

impl RenderTarget for GpuRenderTarget<'_> {
    fn hex_to_point(&self, coord: &HexCoord) -> Point {
        let point = coord.to_cartesian();
        Point::new(point.x * HEX_SIZE, point.y * HEX_SIZE)
    }

    fn get_bounds(&self) -> Rectangle {
        self.bounds
    }

    fn fill_polygon(&mut self, point: &Point, fill: Color) {
        push_hex_fill(&mut self.current_mesh, *point, HEX_SIZE, fill);
    }

    fn stroke_polygon(&mut self, point: &Point, colour: Color, width: f32) {
        push_hex_stroke(&mut self.current_mesh, *point, HEX_SIZE, colour, width);
    }

    fn draw_image(&mut self, bounds: Rectangle, image: ImageId, opacity: f32) {
        self.flush_mesh();

        let Some(handle) = self.assets.image_data(image) else {
            return;
        };

        let raw = match handle {
            iced::advanced::image::Handle::Rgba {
                width,
                height,
                pixels,
                ..
            } => Arc::new(gpu::RawImage {
                width: *width,
                height: *height,
                pixels: pixels.as_ref().to_vec(),
            }),
            _ => {
                return;
            }
        };

        self.commands.push(DrawCommand::Image {
            image,
            vertices: quad_vertices(bounds, opacity),
            raw,
        });
    }
}
