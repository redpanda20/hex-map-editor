#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Tool {
    Paint,
    Erase,
    Pan,
    Fill,
}

impl Default for Tool {
    fn default() -> Self {
        Tool::Pan
    }
}
