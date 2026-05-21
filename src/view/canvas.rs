use iced::{
    Color, Point, Rectangle, Theme, Vector, mouse,
    widget::canvas::{self, Event, Fill, Frame, Geometry, Path, Program, Stroke},
};

use crate::{
    app::Message,
    state::{HexCoord, Layer},
};

#[derive(Debug)]
pub struct CanvasState {
    cache: canvas::Cache,
    dragging: bool,
    last_drag_pos: Option<Point>,
    pan: (f32, f32),
    zoom: f32,
}

impl Default for CanvasState {
    fn default() -> Self {
        Self {
            cache: Default::default(),
            dragging: false,
            last_drag_pos: None,
            pan: (0.0, 0.0),
            zoom: 1.0,
        }
    }
}

impl CanvasState {
    pub fn request_redraw(&mut self) {
        self.cache.clear();
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Tool {
    Paint,
    Erase,
    Pan,
}

pub struct CanvasSettings {
    pub tool: Tool,
    pub hex_size: f32,
}

pub struct HexCanvas<'a> {
    pub layers: &'a Vec<Layer>,
    pub settings: &'a CanvasSettings,
}

impl<'a> Program<Message> for HexCanvas<'a> {
    type State = CanvasState;

    fn draw(
        &self,
        state: &CanvasState,
        renderer: &iced::Renderer,
        _theme: &Theme,
        bounds: Rectangle,
        _cursor: mouse::Cursor,
    ) -> Vec<Geometry> {
        let geometry = state.cache.draw(renderer, bounds.size(), |frame| {
            self.draw_map(state, frame, bounds);
        });
        vec![geometry]
    }

    fn update(
        &self,
        state: &mut CanvasState,
        event: Event,
        bounds: Rectangle,
        cursor: mouse::Cursor,
    ) -> (canvas::event::Status, Option<Message>) {
        let Some(cursor_pos) = cursor.position_in(bounds) else {
            state.dragging = false;
            state.last_drag_pos = None;
            return (canvas::event::Status::Ignored, None);
        };

        match event {
            Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left)) => {
                state.dragging = true;
                state.last_drag_pos = Some(cursor_pos);
                match self.settings.tool {
                    Tool::Pan => (canvas::event::Status::Captured, None),

                    Tool::Paint => {
                        let coord = self.screen_to_hex(state, cursor_pos);
                        state.request_redraw();
                        (
                            canvas::event::Status::Captured,
                            Some(Message::PaintTile(coord)),
                        )
                    }
                    Tool::Erase => {
                        let coord = self.screen_to_hex(state, cursor_pos);
                        state.request_redraw();
                        (
                            canvas::event::Status::Captured,
                            Some(Message::EraseTile(coord)),
                        )
                    }
                }
            }

            Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Left)) => {
                state.dragging = false;
                state.last_drag_pos = None;
                (canvas::event::Status::Ignored, None)
            }

            Event::Mouse(mouse::Event::CursorMoved { .. }) => {
                if !state.dragging {
                    return (canvas::event::Status::Ignored, None);
                };

                let last = state.last_drag_pos;
                state.last_drag_pos = Some(cursor_pos);

                match self.settings.tool {
                    Tool::Pan => match last {
                        None => (canvas::event::Status::Ignored, None),
                        Some(last) => {
                            let dx = cursor_pos.x - last.x;
                            let dy = cursor_pos.y - last.y;
                            state.pan.0 += dx;
                            state.pan.1 += dy;
                            state.request_redraw();
                            (canvas::event::Status::Captured, None)
                        }
                    },
                    Tool::Paint => {
                        state.request_redraw();
                        (
                            canvas::event::Status::Captured,
                            Some(Message::PaintTile(self.screen_to_hex(state, cursor_pos))),
                        )
                    }
                    Tool::Erase => {
                        state.request_redraw();
                        (
                            canvas::event::Status::Captured,
                            Some(Message::EraseTile(self.screen_to_hex(state, cursor_pos))),
                        )
                    }
                }
            }

            Event::Mouse(mouse::Event::WheelScrolled { delta }) => {
                let delta = match delta {
                    mouse::ScrollDelta::Lines { x, y } => (x + y) * 20.0,
                    mouse::ScrollDelta::Pixels { x, y } => x + y,
                };
                state.zoom = f32::clamp(state.zoom + delta * 0.01, 0.1, 10.0);
                state.request_redraw();
                (canvas::event::Status::Captured, None)
            }

            _ => (canvas::event::Status::Ignored, None),
        }
    }

    fn mouse_interaction(
        &self,
        state: &CanvasState,
        _bounds: Rectangle,
        _cursor: mouse::Cursor,
    ) -> mouse::Interaction {
        match self.settings.tool {
            Tool::Pan if state.dragging => mouse::Interaction::Grabbing,
            Tool::Pan => mouse::Interaction::Grab,
            Tool::Paint => mouse::Interaction::Crosshair,
            Tool::Erase => mouse::Interaction::Crosshair,
        }
    }
}

impl<'a> HexCanvas<'a> {
    fn draw_map(&self, state: &CanvasState, frame: &mut Frame, bounds: Rectangle) {
        let pan = state.pan;
        let zoom = state.zoom;
        let hex_w = self.settings.hex_size * 2.0;
        let hex_h = self.settings.hex_size * (3.0_f32).sqrt();

        // Background
        frame.fill_rectangle(Point::ORIGIN, bounds.size(), Color::from_rgb8(30, 32, 40));

        frame.translate(Vector::new(pan.0, pan.1));
        frame.scale(zoom);

        // Compute visible bounds in map-space for culling
        let inv_zoom = 1.0 / zoom;
        let map_x0 = (-pan.0) * inv_zoom;
        let map_y0 = (-pan.1) * inv_zoom;
        let map_x1 = map_x0 + bounds.width * inv_zoom;
        let map_y1 = map_y0 + bounds.height * inv_zoom;

        // Precalcuate path for hexagon
        let hex_path = self.hex_path();

        for layer in self.layers.iter() {
            if !layer.visible {
                continue;
            }
            for &coord in &layer.tiles {
                let (cx, cy) = coord.to_pixel(self.settings.hex_size);

                // Cull tiles outside the visible map-space rectangle
                if cx + hex_w < map_x0
                    || cx - hex_w > map_x1
                    || cy + hex_h < map_y0
                    || cy - hex_h > map_y1
                {
                    continue;
                }

                frame.with_save(|frame| {
                    frame.translate(Vector::new(cx, cy));

                    frame.fill(
                        &hex_path,
                        Fill {
                            style: canvas::Style::Solid(layer.color),
                            rule: canvas::fill::Rule::NonZero,
                        },
                    );

                    frame.stroke(
                        &hex_path,
                        Stroke {
                            style: canvas::Style::Solid(Color::from_rgba(1.0, 1.0, 1.0, 0.2)),
                            width: 0.5,
                            ..Stroke::default()
                        },
                    );
                });
            }
        }

        {
            let col_min = (map_x0 / (hex_w * 0.75)).floor() as i32 - 1;
            let col_max = (map_x1 / (hex_w * 0.75)).ceil() as i32 + 1;
            let row_min = (map_y0 / hex_h).floor() as i32 - 1;
            let row_max = (map_y1 / hex_h).ceil() as i32 + 1;

            for col in col_min..=col_max {
                for row in row_min..=row_max {
                    let coord = HexCoord::new(col, row);
                    let (cx, cy) = coord.to_pixel(self.settings.hex_size);

                    frame.with_save(|frame| {
                        frame.translate(Vector::new(cx, cy));
                        frame.stroke(
                            &hex_path,
                            Stroke {
                                style: canvas::Style::Solid(Color::from_rgba(1.0, 1.0, 1.0, 0.06)),
                                width: 0.5,
                                ..Stroke::default()
                            },
                        );
                    });
                }
            }
        }
    }

    fn screen_to_hex(&self, state: &CanvasState, screen: Point) -> HexCoord {
        let hex_size = self.settings.hex_size;
        let pan = state.pan;
        let zoom = state.zoom;

        let map_x = (screen.x - pan.0) / zoom;
        let map_y = (screen.y - pan.1) / zoom;
        HexCoord::from_pixel(map_x, map_y, hex_size)
    }

    fn hex_path(&self) -> Path {
        let mut builder = canvas::path::Builder::new();
        for i in 0..6 {
            let angle = std::f32::consts::PI / 180.0 * (60.0 * i as f32);
            let px = self.settings.hex_size * angle.cos();
            let py = self.settings.hex_size * angle.sin();
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
