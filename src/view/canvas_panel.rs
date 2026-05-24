use iced::{
    Point, Rectangle, Theme, Vector, mouse,
    widget::{
        Action,
        canvas::{self, Event, Fill, Frame, Geometry, Path, Program, Stroke},
    },
};

use crate::{
    app::Message,
    state::{HEX_SIZE, HexCoord, Layer, Tool},
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

pub struct HexCanvas<'a> {
    pub layers: &'a Vec<Layer>,
    pub tool: &'a Tool,
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
            Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left)) => {
                state.dragging = true;
                state.last_drag_pos = Some(cursor_pos);
                match self.tool {
                    Tool::Pan => Some(Action::capture()),

                    Tool::Paint => {
                        let coord = self.screen_to_hex(state, cursor_pos);
                        state.request_redraw();
                        Some(Action::publish(Message::PaintTile(coord)))
                    }
                    Tool::Erase => {
                        let coord = self.screen_to_hex(state, cursor_pos);
                        state.request_redraw();
                        Some(Action::publish(Message::EraseTile(coord)))
                    }
                }
            }

            Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Left)) => {
                state.dragging = false;
                state.last_drag_pos = None;
                None
            }

            Event::Mouse(mouse::Event::CursorMoved { .. }) => {
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
                            state.pan.0 += dx;
                            state.pan.1 += dy;
                            state.request_redraw();
                            Some(Action::request_redraw().and_capture())
                        }
                    },
                    Tool::Paint => {
                        let coord = self.screen_to_hex(state, cursor_pos);
                        state.request_redraw();
                        Some(Action::publish(Message::PaintTile(coord)))
                    }
                    Tool::Erase => {
                        let coord = self.screen_to_hex(state, cursor_pos);
                        state.request_redraw();
                        Some(Action::publish(Message::EraseTile(coord)))
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
        let pan = state.pan;
        let zoom = state.zoom;
        let hex_w = HEX_SIZE * 2.0;
        let hex_h = HEX_SIZE * (3.0_f32).sqrt();

        // Background
        // frame.fill_rectangle(Point::ORIGIN, bounds.size(), Color::from_rgb8(30, 32, 40));

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
                let (cx, cy) = coord.to_pixel();

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
                            style: canvas::Style::Solid(
                                theme
                                    .extended_palette()
                                    .background
                                    .base
                                    .text
                                    .scale_alpha(0.5),
                            ),
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
                    let (cx, cy) = coord.to_pixel();

                    frame.with_save(|frame| {
                        frame.translate(Vector::new(cx, cy));
                        frame.stroke(
                            &hex_path,
                            Stroke {
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
                            },
                        );
                    });
                }
            }
        }
    }

    fn screen_to_hex(&self, state: &CanvasState, screen: Point) -> HexCoord {
        let pan = state.pan;
        let zoom = state.zoom;

        let map_x = (screen.x - pan.0) / zoom;
        let map_y = (screen.y - pan.1) / zoom;
        HexCoord::from_pixel(map_x, map_y)
    }

    fn hex_path(&self) -> Path {
        let mut builder = canvas::path::Builder::new();
        for i in 0..6 {
            let angle = std::f32::consts::PI / 180.0 * (60.0 * i as f32);
            let px = HEX_SIZE * angle.cos();
            let py = HEX_SIZE * angle.sin();
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
