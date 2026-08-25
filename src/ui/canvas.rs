use std::cell::Cell;

use iced::{
    Element, Length, Point, Rectangle, Task, Theme, Vector,
    advanced::image::Handle,
    mouse, touch,
    widget::{
        Action,
        canvas::{self, Event, Fill, Frame, Geometry, Path, Program, Stroke},
    },
};

#[derive(Debug, Clone, Copy)]
pub enum CanvasEvent {
    PointerPressed { at: HexCoord },
    PointerMoved { from: HexCoord, to: HexCoord },
    PointerReleased,
}

impl CanvasEvent {
    pub fn into_task(self, current_layer: &Option<LayerId>, tool: &Tool) -> Task<Message> {
        let command: Box<dyn EditCommand> = match (*current_layer, tool, self) {
            (Some(layer), Tool::Paint, CanvasEvent::PointerPressed { at }) => {
                Box::new(PaintTile { layer, coord: at })
            }
            (Some(layer), Tool::Paint, CanvasEvent::PointerMoved { from: _, to }) => {
                Box::new(PaintTile { layer, coord: to })
            }

            (Some(layer), Tool::Erase, CanvasEvent::PointerPressed { at }) => {
                Box::new(EraseTile { layer, coord: at })
            }
            (Some(layer), Tool::Erase, CanvasEvent::PointerMoved { from: _, to }) => {
                Box::new(EraseTile { layer, coord: to })
            }

            (Some(layer), Tool::Fill, CanvasEvent::PointerPressed { at }) => {
                Box::new(BucketFill { layer, from: at })
            }

            _ => return Task::none(),
        };
        Task::done(Message::Scene(command))
    }
}

use crate::{
    app::Message,
    domain::{
        EditCommand, HexCoord, RenderTarget, Scene, Tool,
        assets::{AssetStore, ImageAsset},
        edit::{BucketFill, EraseTile, PaintTile},
        id::{ImageId, LayerId},
        layer::overlay::HexGridOverlay,
    },
};

const HEX_SIZE: f32 = 16.0;

pub fn canvas_panel<'a>(scene: &'a Scene, tool: Tool) -> Element<'a, Message> {
    let hex_canvas = HexCanvas { scene, tool };

    let element: Element<'_, CanvasEvent> = iced::widget::canvas(hex_canvas)
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
    cache: canvas::Cache,
    // Tracks the `Layers` revision that `cache` was last drawn from. Wrapped
    // in a `Cell` so it can be updated from `Program::draw`, which only
    // hands us a `&CanvasState` (see `HexCanvas::draw`).
    cached_layers_revision: Cell<u64>,
    dragging: bool,
    last_drag_pos: Option<Point>,
    translation: Vector,
    zoom: f32,
}

impl Default for CanvasState {
    fn default() -> Self {
        Self {
            cache: Default::default(),
            cached_layers_revision: Cell::new(0),
            dragging: false,
            last_drag_pos: None,
            translation: Vector::new(0.0, 0.0),
            zoom: 1.0,
        }
    }
}

impl<'a> Program<CanvasEvent> for HexCanvas<'a> {
    type State = CanvasState;

    fn draw(
        &self,
        state: &CanvasState,
        renderer: &iced::Renderer,
        theme: &Theme,
        bounds: Rectangle,
        cursor: mouse::Cursor,
    ) -> Vec<Geometry> {
        // Invalidate render cache if the state of Layers has changed.
        let current_revision = self.scene.revision();
        if state.cached_layers_revision.get() != current_revision {
            state.cache.clear();
            state.cached_layers_revision.set(current_revision);
        }

        // Cache map drawing
        let map = state.cache.draw(renderer, bounds.size(), |frame| {
            self.draw_map(state, theme, frame, bounds);
        });

        // Get cursor pos. Otherwise just draw map by itself
        let Some(cursor_pos) = cursor.position_in(bounds) else {
            return vec![map];
        };

        // Don't draw hex indicator if user is panning
        if Tool::Pan == self.tool {
            return vec![map];
        }

        let mouse = self.draw_cursor_hex(renderer, theme, bounds, state, cursor_pos);
        vec![map, mouse]
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

                    state.cache.clear();
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

                state.cache.clear();
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
    fn draw_cursor_hex(
        &self,
        renderer: &iced::Renderer,
        theme: &Theme,
        bounds: Rectangle,
        state: &CanvasState,
        cursor_pos: Point,
    ) -> Geometry {
        let mut frame = Frame::new(renderer, bounds.size());

        let hex = self.screen_to_hex(state, cursor_pos);
        let coord = hex.to_cartesian() * HEX_SIZE * state.zoom + state.translation;

        frame.translate(coord);
        frame.scale(state.zoom);

        let stroke = Stroke {
            style: canvas::Style::Solid(theme.extended_palette().primary.base.color),
            width: 2.0 / state.zoom,
            ..Stroke::default()
        };

        frame.stroke(&hex_path(HEX_SIZE), stroke);
        frame.into_geometry()
    }

    fn draw_map(&self, state: &CanvasState, _theme: &Theme, frame: &mut Frame, bounds: Rectangle) {
        frame.translate(state.translation);
        frame.scale(state.zoom);

        let inv_scale = 1.0 / HEX_SIZE / state.zoom;
        let relative_bounds =
            Rectangle::with_size(bounds.size()) * inv_scale - state.translation * inv_scale;

        let mut target = CanvasRenderTarget {
            frame,
            bounds: relative_bounds,
            assets: &self.scene.assets,
        };

        let mut layers = self.scene.get_visible_layers();
        let overlay = HexGridOverlay::new_light();
        layers.push(&overlay);

        for layer in layers {
            layer.draw(&mut target);
        }
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

fn hex_path(hex_size: f32) -> Path {
    let mut builder = canvas::path::Builder::new();
    for i in 0..6 {
        let angle = std::f32::consts::PI / 180.0 * (60.0 * i as f32);
        let px = hex_size * angle.cos();
        let py = hex_size * angle.sin();
        if i == 0 {
            builder.move_to(Point::new(px, py));
        } else {
            builder.line_to(Point::new(px, py));
        }
    }
    builder.close();
    builder.build()
}

struct CanvasRenderTarget<'a> {
    frame: &'a mut Frame,
    bounds: Rectangle,
    assets: &'a AssetStore,
}

impl<'a> RenderTarget for CanvasRenderTarget<'a> {
    fn hex_to_point(&self, coord: &HexCoord) -> Point {
        let point = coord.to_cartesian();

        Point::new(point.x * HEX_SIZE, point.y * HEX_SIZE)
    }

    fn get_bounds(&self) -> Rectangle {
        self.bounds
    }

    fn fill_polygon(&mut self, point: &Point, fill: iced::Color) {
        let path = hex_path(HEX_SIZE);

        self.frame.with_save(|frame| {
            frame.translate(Vector::new(point.x, point.y));
            frame.fill(&path, Fill::from(fill));
        });
    }

    fn stroke_polygon(&mut self, point: &Point, colour: iced::Color) {
        let path = hex_path(HEX_SIZE);

        self.frame.with_save(|frame| {
            frame.translate(Vector::new(point.x, point.y));

            let stroke = Stroke::default().with_color(colour);
            frame.stroke(&path, stroke);
        });
    }

    fn draw_image(&mut self, bounds: Rectangle, image: ImageId, opacity: f32) {
        if let Some(ImageAsset {
            data,
            width,
            height,
        }) = self.assets.image(image).cloned()
        {
            let handle = Handle::from_rgba(width, height, data);
            let image = iced::advanced::image::Image::new(handle).opacity(opacity);
            self.frame.draw_image(bounds, image);
        }
    }
}
