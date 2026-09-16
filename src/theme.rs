use iced::theme::{
    Palette,
    palette::{Extended, Pair},
};

pub mod token {
    use iced::{Color, color};

    // Surfaces
    pub const SURFACE_0: Color = color!(0x14171C); // app shell background
    pub const SURFACE_1: Color = color!(0x1B1F26); // docked panels: Canvas, Layer stack, Inspector
    pub const SURFACE_2: Color = color!(0x232833); // raised elements: list rows, dropdowns
    pub const SURFACE_3: Color = color!(0x2A303C); // overlays: modals, popovers
    pub const SURFACE_HOVER: Color = color!(0x2E3542);
    pub const SURFACE_SELECTED: Color = color!(0x33302A); // bronze-tinted selection wash

    // Text
    pub const TEXT_PRIMARY: Color = color!(0xEDEFF2);
    pub const TEXT_SECONDARY: Color = color!(0xA8B0BD);
    pub const TEXT_TERTIARY: Color = color!(0x6B7280);
    pub const TEXT_INVERSE: Color = color!(0x14171C); // text set on filled bronze

    // Borders
    pub const BORDER_SUBTLE: Color = color!(0x2C323D);
    pub const BORDER_DEFAULT: Color = color!(0x3A4150);
    pub const BORDER_STRONG: Color = color!(0x4C5568);

    // Bronze — primary accent scale
    pub const BRONZE_100: Color = color!(0xF0DEC4);
    pub const BRONZE_300: Color = color!(0xCCA476);
    pub const BRONZE_400: Color = color!(0xB8875C); // accent text/icons on dark surfaces
    pub const BRONZE_500: Color = color!(0xA6733F); // primary accent
    pub const BRONZE_600: Color = color!(0x8C5F32); // pressed/active
    pub const BRONZE_700: Color = color!(0x6F4A27); // text on light bronze fills

    // Semantic
    pub const SUCCESS: Color = color!(0x5FA87A);
    pub const WARNING: Color = color!(0xD4A72C);
    pub const DANGER: Color = color!(0xD9564C);
    pub const INFO: Color = color!(0x5B8DD9);
}

use token::*;

pub const PALETTE: Palette = Palette {
    background: SURFACE_1,
    text: TEXT_PRIMARY,

    // primary buttons, active tool state, selection, focus rings.
    primary: BRONZE_500,

    // confirmation toasts, valid field state.
    success: SUCCESS,

    // caution toasts, fields nearing a bound.
    warning: WARNING,

    // destructive actions, error toasts, invalid state.
    danger: DANGER,
};

pub fn extended_fn(palette: Palette) -> Extended {
    let mut extended = Extended::generate(palette);

    // Adjust "weak" primary pair to match bronze-400 exactly.
    extended.primary.weak = Pair {
        color: BRONZE_400,
        text: TEXT_INVERSE,
    };

    extended
}
