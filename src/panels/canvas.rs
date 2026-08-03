use iced::{
    Element, Length, Point, Rectangle, Theme, Vector, mouse, touch,
    widget::{
        Action,
        canvas::{self, Event, Fill, Frame, Geometry, Path, Program, Stroke},
    },
};

use crate::{
    app::Message,
    state::{HexCoord, LayerMessage, Layers, Tool, hexes_in_range},
};

pub fn canvas_panel<'a>(layers: &'a Layers, tool: &'a Tool) -> Element<'a, Message> {
    let hex_canvas = HexCanvas {
        layers,
        tool,
        hex_size: 16.0,
    };

    iced::widget::canvas(hex_canvas)
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
}

pub struct HexCanvas<'a> {
    pub layers: &'a Layers,
    pub tool: &'a Tool,
    pub hex_size: f32,
}

#[derive(Debug)]
pub struct CanvasState {
    cache: canvas::Cache,
    dragging: bool,
    last_drag_pos: Option<Point>,
    translation: Vector,
    zoom: f32,
}

impl Default for CanvasState {
    fn default() -> Self {
        Self {
            cache: Default::default(),
            dragging: false,
            last_drag_pos: None,
            translation: Vector::new(0.0, 0.0),
            zoom: 1.0,
        }
    }
}

impl CanvasState {
    pub fn request_redraw(&mut self) {
        self.cache.clear();
    }
}

impl<'a> Program<Message> for HexCanvas<'a> {
    type State = CanvasState;

    fn draw(
        &self,
        state: &CanvasState,
        renderer: &iced::Renderer,
        theme: &Theme,
        bounds: Rectangle,
        _cursor: mouse::Cursor,
    ) -> Vec<Geometry> {
        let geometry = state.cache.draw(renderer, bounds.size(), |frame| {
            self.draw_map(state, theme, frame, bounds);
        });
        vec![geometry]
    }

    fn update(
        &self,
        state: &mut CanvasState,
        event: &Event,
        bounds: Rectangle,
        cursor: mouse::Cursor,
    ) -> Option<Action<Message>> {
        if !cursor.is_over(bounds) {
            state.dragging = false;
            state.last_drag_pos = None;
            return None;
        }

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
                match self.tool {
                    Tool::Pan => Some(Action::capture()),

                    Tool::Paint => {
                        let coord = self.screen_to_hex(state, cursor_pos);
                        state.request_redraw();

                        Some(Action::publish(Message::LayerEvent(
                            LayerMessage::PaintHex(coord),
                        )))
                    }
                    Tool::Erase => {
                        let coord = self.screen_to_hex(state, cursor_pos);
                        state.request_redraw();
                        Some(Action::publish(Message::LayerEvent(
                            LayerMessage::EraseHex(coord),
                        )))
                    }
                }
            }

            Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Left))
            | Event::Touch(touch::Event::FingerLifted { .. })
            | Event::Touch(touch::Event::FingerLost { .. }) => {
                state.dragging = false;
                state.last_drag_pos = None;
                None
            }

            Event::Mouse(mouse::Event::CursorMoved { .. })
            | Event::Touch(touch::Event::FingerMoved { .. }) => {
                if !state.dragging {
                    return None;
                };

                let last = state.last_drag_pos;
                state.last_drag_pos = Some(cursor_pos);

                match self.tool {
                    Tool::Pan => match last {
                        None => None,
                        Some(last) => {
                            let dx = cursor_pos.x - last.x;
                            let dy = cursor_pos.y - last.y;
                            state.translation.x += dx;
                            state.translation.y += dy;
                            state.request_redraw();
                            Some(Action::request_redraw().and_capture())
                        }
                    },
                    Tool::Paint => {
                        let coord = self.screen_to_hex(state, cursor_pos);
                        state.request_redraw();
                        Some(Action::publish(Message::LayerEvent(
                            LayerMessage::PaintHex(coord),
                        )))
                    }
                    Tool::Erase => {
                        let coord = self.screen_to_hex(state, cursor_pos);
                        state.request_redraw();
                        Some(Action::publish(Message::LayerEvent(
                            LayerMessage::EraseHex(coord),
                        )))
                    }
                }
            }

            Event::Mouse(mouse::Event::WheelScrolled { delta }) => {
                let delta = match delta {
                    mouse::ScrollDelta::Lines { x, y } => (x + y) * 20.0,
                    mouse::ScrollDelta::Pixels { x, y } => x + y,
                };
                state.zoom = f32::clamp(state.zoom + delta * 0.01, 0.4, 10.0);
                state.request_redraw();
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
            return mouse::Interaction::Idle;
        }

        match self.tool {
            Tool::Pan if state.dragging => mouse::Interaction::Grabbing,
            Tool::Pan => mouse::Interaction::Grab,
            Tool::Paint => mouse::Interaction::Crosshair,
            Tool::Erase => mouse::Interaction::Crosshair,
        }
    }
}

impl<'a> HexCanvas<'a> {
    fn draw_map(&self, state: &CanvasState, theme: &Theme, frame: &mut Frame, bounds: Rectangle) {
        frame.translate(state.translation);
        frame.scale(state.zoom);

        // Compute hex bounds in map-space for culling
        let inv_zoom = 1.0 / state.zoom;
        let inv_hex_w = 1.0 / (self.hex_size * 1.5);
        let inv_hex_h = 1.0 / (self.hex_size * (3.0_f32).sqrt());

        let col_min = (-state.translation.x * inv_hex_w * inv_zoom).floor() as i32;
        let col_max = col_min + (bounds.width * inv_hex_w * inv_zoom).ceil() as i32;

        let row_min = (-state.translation.y * inv_hex_h * inv_zoom).floor() as i32;
        let row_max = row_min + (bounds.height * inv_hex_h * inv_zoom).ceil() as i32;

        let hex_path = &self.hex_path();

        // Draw grid layers
        for layer in self.layers.get_visible_layers() {
            let coords = hexes_in_range(col_min, col_max, row_min, row_max);

            layer.draw(frame, coords, |frame, hex, colour| {
                let centre = hex.to_pixel(self.hex_size);
                frame.with_save(|frame| {
                    frame.translate(centre);
                    frame.fill(hex_path, Fill::from(colour));
                })
            });
        }

        // Draw grid overlay
        let grid_stroke = Stroke {
            style: canvas::Style::Solid(
                theme
                    .extended_palette()
                    .background
                    .base
                    .text
                    .scale_alpha(0.1),
            ),
            width: 1.0,
            ..Stroke::default()
        };

        let coords = hexes_in_range(col_min, col_max, row_min, row_max);
        for hex in coords {
            let centre = hex.to_pixel(self.hex_size);
            frame.with_save(|frame| {
                frame.translate(centre);
                frame.stroke(&hex_path, grid_stroke);
            })
        }
    }

    fn screen_to_hex(&self, state: &CanvasState, screen: Point) -> HexCoord {
        let translation = state.translation;
        let zoom = state.zoom;

        let map_x = (screen.x - translation.x) / zoom;
        let map_y = (screen.y - translation.y) / zoom;

        HexCoord::from_pixel(map_x, map_y, self.hex_size)
    }

    fn hex_path(&self) -> Path {
        let mut builder = canvas::path::Builder::new();
        for i in 0..6 {
            let angle = std::f32::consts::PI / 180.0 * (60.0 * i as f32);
            let px = self.hex_size * angle.cos();
            let py = self.hex_size * angle.sin();
            if i == 0 {
                builder.move_to(Point::new(px, py));
            } else {
                builder.line_to(Point::new(px, py));
            }
        }
        builder.close();
        builder.build()
    }
}
