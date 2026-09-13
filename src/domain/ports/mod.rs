mod inspect;
mod render;

pub use render::RenderTarget;

#[cfg(test)]
pub(crate) use render::MockRenderer;
