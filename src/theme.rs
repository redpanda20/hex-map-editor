use iced::theme::{
    Palette,
    palette::{Background, Danger, Extended, Pair, Primary, Secondary, Success, Warning},
};

#[allow(unused)]
pub mod raw {
    use iced::{Color, color};

    // App surface
    pub const APP_BG: Color = color!(0x141618);
    pub const APP_PANEL: Color = color!(0x1B1D1F);
    pub const APP_SURFACE: Color = color!(0x212326);
    pub const APP_SURFACE_2: Color = color!(0x26282B);
    pub const APP_BORDER: Color = color!(0x33363A);
    pub const APP_BORDER_STRONG: Color = color!(0x45484C);
    pub const APP_BORDER_SUBTLE: Color = color!(0x2A2D30);

    // Text
    pub const TEXT_PRIMARY: Color = color!(0xE5E4E0);
    pub const TEXT_SECONDARY: Color = color!(0xC9CBC7);
    pub const TEXT_MUTED: Color = color!(0x8C948E);
    pub const TEXT_FAINT: Color = color!(0x6D716F);

    // Brand accent (warm amber — primary actions + Tile layer type only)
    pub const ACCENT: Color = color!(0xEF9F27);
    pub const ACCENT_BG: Color = color!(0x3A3025);
    pub const ACCENT_INK: Color = color!(0x412402);
    pub const ACCENT_TINT: Color = color!(0xFAEEDA);

    // Layer-type colors — fixed mapping, never reassigned
    pub const LAYER_TILE: Color = color!(0xEF9F27);
    pub const LAYER_TILE_BG: Color = color!(0x3A3025);
    pub const LAYER_IMAGE: Color = color!(0x8C948E);
    pub const LAYER_IMAGE_BG: Color = color!(0x26282B);
    pub const LAYER_IMAGETILE: Color = color!(0x5DCAA5);
    pub const LAYER_IMAGETILE_BG: Color = color!(0x1B2420);
    pub const LAYER_TEXT: Color = color!(0xD4537E);
    pub const LAYER_TEXT_BG: Color = color!(0x2B1B21);
    pub const LAYER_NOISE: Color = color!(0x7F77DD);
    pub const LAYER_NOISE_BG: Color = color!(0x211E2A);

    // Status colors
    pub const SUCCESS: Color = color!(0x5AAD6B);
    pub const SUCCESS_BG: Color = color!(0x1C2A20);
    pub const SUCCESS_BORDER: Color = color!(0x2E4A38);
    pub const SUCCESS_STRONG: Color = color!(0x3F8F55);
    pub const SUCCESS_INK: Color = color!(0xEFFAF1);

    pub const WARNING: Color = color!(0xC99A3D);
    pub const WARNING_BG: Color = color!(0x2C2417);
    pub const WARNING_BORDER: Color = color!(0x4A3A1E);
    pub const WARNING_STRONG: Color = color!(0xA67B22);
    pub const WARNING_INK: Color = color!(0xFBF1DE);

    pub const DANGER: Color = color!(0xC4685F);
    pub const DANGER_BG: Color = color!(0x2A1D1B);
    pub const DANGER_BORDER: Color = color!(0x4A2E29);
    pub const DANGER_STRONG: Color = color!(0xB8483F);
    pub const DANGER_INK: Color = color!(0xFCEDEB);
}

pub const PALETTE: Palette = Palette {
    background: raw::APP_PANEL,
    text: raw::TEXT_PRIMARY,
    primary: raw::ACCENT,
    success: raw::SUCCESS,
    warning: raw::WARNING,
    danger: raw::DANGER_STRONG,
};

pub const EXTENDED: Extended = Extended {
    background: Background {
        base: Pair {
            color: raw::APP_BG,
            text: raw::TEXT_PRIMARY,
        },
        weakest: Pair {
            color: raw::APP_PANEL,
            text: raw::TEXT_PRIMARY,
        },
        weaker: Pair {
            color: raw::APP_SURFACE,
            text: raw::TEXT_PRIMARY,
        },
        weak: Pair {
            color: raw::APP_SURFACE_2,
            text: raw::TEXT_SECONDARY,
        },
        neutral: Pair {
            color: raw::APP_BORDER_SUBTLE,
            text: raw::TEXT_SECONDARY,
        },
        strong: Pair {
            color: raw::APP_BORDER,
            text: raw::TEXT_PRIMARY,
        },
        stronger: Pair {
            color: raw::APP_BORDER_STRONG,
            text: raw::TEXT_PRIMARY,
        },
        strongest: Pair {
            color: raw::TEXT_FAINT,
            text: raw::APP_BG,
        },
    },

    primary: Primary {
        base: Pair {
            color: raw::ACCENT,
            text: raw::ACCENT_INK,
        },
        weak: Pair {
            color: raw::ACCENT_BG,
            text: raw::ACCENT_TINT,
        },
        strong: Pair {
            color: raw::ACCENT_TINT,
            text: raw::ACCENT_INK,
        },
    },

    secondary: Secondary {
        base: Pair {
            color: raw::LAYER_IMAGE,
            text: raw::APP_BG,
        },
        weak: Pair {
            color: raw::LAYER_IMAGE_BG,
            text: raw::TEXT_MUTED,
        },
        strong: Pair {
            color: raw::TEXT_SECONDARY,
            text: raw::APP_BG,
        },
    },

    success: Success {
        base: Pair {
            color: raw::SUCCESS,
            text: raw::SUCCESS_INK,
        },
        weak: Pair {
            color: raw::SUCCESS_BG,
            text: raw::SUCCESS,
        },
        strong: Pair {
            color: raw::SUCCESS_STRONG,
            text: raw::SUCCESS_INK,
        },
    },

    warning: Warning {
        base: Pair {
            color: raw::WARNING,
            text: raw::WARNING_INK,
        },
        weak: Pair {
            color: raw::WARNING_BG,
            text: raw::WARNING,
        },
        strong: Pair {
            color: raw::WARNING_STRONG,
            text: raw::WARNING_INK,
        },
    },

    danger: Danger {
        base: Pair {
            color: raw::DANGER,
            text: raw::DANGER_INK,
        },
        weak: Pair {
            color: raw::DANGER_BG,
            text: raw::DANGER,
        },
        strong: Pair {
            color: raw::DANGER_STRONG,
            text: raw::DANGER_INK,
        },
    },

    is_dark: true,
};
