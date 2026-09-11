use std::cell::RefCell;
use std::sync::Arc;

use iced::{Point, Rectangle, Vector};

use super::HEX_SIZE;
use super::gpu::DrawCommand;
use crate::domain::HexCoord;

const MIN_ZOOM: f32 = 0.4;
const MAX_ZOOM: f32 = 10.0;

/// Pan (`translation`) and zoom applied on top of screen space.
#[derive(Debug, Clone, Copy)]
pub struct Camera {
    pub translation: Vector,
    pub zoom: f32,
}

impl Default for Camera {
    fn default() -> Self {
        Self {
            translation: Vector::new(0.0, 0.0),
            zoom: 1.0,
        }
    }
}

impl Camera {
    pub fn pan_by(&mut self, delta: Vector) {
        self.translation += delta
    }

    pub fn zoom_by(&mut self, delta: f32) {
        self.zoom = (self.zoom + delta).clamp(MIN_ZOOM, MAX_ZOOM);
    }

    /// Screen space -> pixel-world space.
    fn screen_to_pixel_world(&self, screen: Point) -> Vector {
        Vector {
            x: (screen.x - self.translation.x) / self.zoom,
            y: (screen.y - self.translation.y) / self.zoom,
        }
    }

    /// Screen space -> hex-cartesian space, ready for `HexCoord::from_cartesian`.
    pub fn screen_to_hex(&self, screen: Point) -> HexCoord {
        HexCoord::from_cartesian(self.screen_to_pixel_world(screen) / HEX_SIZE)
    }

    /// The visible viewport, in hex-cartesian space - what `GpuRenderTarget`
    /// is constructed with so layers can cull against it.
    pub fn visible_hex_bounds(&self, viewport: Rectangle) -> Rectangle {
        let inv_scale = 1.0 / (HEX_SIZE * self.zoom);
        Rectangle::with_size(viewport.size()) * inv_scale - self.translation * inv_scale
    }
}

/// A cached build of the map's draw commands, tagged with the
/// `Scene::revision()` it was built from.
#[derive(Debug)]
struct CachedCommands {
    revision: u64,
    commands: Arc<Vec<DrawCommand>>,
}

#[derive(Debug)]
pub(crate) struct CanvasState {
    // Private command cache.
    cache: RefCell<Option<CachedCommands>>,
    /// The pointer's last known position while a press is held.
    pub drag_anchor: Option<Point>,
    pub camera: Camera,
}

impl Default for CanvasState {
    fn default() -> Self {
        Self {
            cache: RefCell::new(None),
            drag_anchor: None,
            camera: Camera::default(),
        }
    }
}

impl CanvasState {
    pub fn is_dragging(&self) -> bool {
        self.drag_anchor.is_some()
    }

    /// Returns the base draw commands for `revision`.
    ///
    /// Provides the cached commands if the revision matches,
    /// otherwise calls `build` to produce (and cache) a fresh set.
    pub fn base_commands(
        &self,
        revision: u64,
        build: impl FnOnce() -> Vec<DrawCommand>,
    ) -> Arc<Vec<DrawCommand>> {
        let mut cache = self.cache.borrow_mut();

        if let Some(cached) = cache.as_ref()
            && cached.revision == revision
        {
            return Arc::clone(&cached.commands);
        }

        let commands = Arc::new(build());
        *cache = Some(CachedCommands {
            revision,
            commands: Arc::clone(&commands),
        });
        commands
    }
}
