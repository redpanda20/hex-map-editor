#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum Tool {
    Paint,
    Erase,
    #[default]
    Pan,
    Fill,
}
