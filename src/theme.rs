use iced::{
    Background, Color, Element, Length, Shadow, Theme, Vector,
    border::Border,
    color,
    overlay::menu,
    theme::Palette,
    widget::{Container, button, checkbox, container, pick_list, slider, space, text_input},
};

// Surfaces
pub const BG: Color = color!(0x0F1117);
pub const SURFACE: Color = color!(0x1E1E1E);
pub const GRID: Color = color!(0x2D2D2D);
pub const TRANSPARENT: Color = Color::TRANSPARENT;

// Accent (primary actions)
pub const ACCENT: Color = color!(0xD4A574);
pub const ACCENT_HOVER: Color = color!(0xDDB588);
pub const ACCENT_PRESSED: Color = color!(0xC4956A);
pub const ON_ACCENT: Color = color!(0x1A1206);

// Slate (secondary actions)
pub const SLATE: Color = color!(0x4A5568);
pub const SLATE_HOVER: Color = color!(0x566176);
pub const SLATE_PRESSED: Color = color!(0x414A5A);

// Status
pub const SUCCESS: Color = color!(0x6B9E6D);
pub const WARNING: Color = color!(0xD9A84B);
pub const DANGER: Color = color!(0xA84B4B);
pub const DANGER_HOVER: Color = color!(0xB85A5A);
pub const DANGER_PRESSED: Color = color!(0x964141);

// Text
pub const TEXT: Color = color!(0xE8E8E8);
pub const TEXT_2: Color = color!(0xA0A0A0);
pub const TEXT_3: Color = color!(0x707070);

// Translucent overlays
pub const SELECTION: Color = color!(0xD4A574, 0.35);
pub const BACKDROP: Color = color!(0x0F1117, 0.6);
pub const MENU_SHADOW: Color = color!(0x000000, 0.5);
pub const MODAL_SHADOW: Color = color!(0x000000, 0.65);

// ---------------------------------------------------------------------------

pub fn theme() -> Theme {
    Theme::custom(
        "Arena".to_string(),
        Palette {
            background: BG,
            text: TEXT,
            primary: ACCENT,
            success: SUCCESS,
            warning: WARNING,
            danger: DANGER,
        },
    )
}

fn filled(
    bg: Option<Color>,
    fg: Color,
    hover: Color,
    press: Color,
    s: button::Status,
) -> button::Style {
    let mut base = button::Style {
        background: bg.map(Background::from),
        text_color: fg,
        border: Border {
            radius: 4.0.into(),
            ..Border::default()
        },
        ..button::Style::default()
    };

    match s {
        button::Status::Active => {}
        button::Status::Hovered => base.background = Some(hover.into()),
        button::Status::Pressed => base.background = Some(press.into()),
        button::Status::Disabled => {
            base.background = Some(GRID.into());
            base.text_color = TEXT_3
        }
    };

    base
}

pub fn button_primary(_: &Theme, s: button::Status) -> button::Style {
    filled(Some(ACCENT), ON_ACCENT, ACCENT_HOVER, ACCENT_PRESSED, s)
}
pub fn button_secondary(_: &Theme, s: button::Status) -> button::Style {
    filled(Some(SLATE), TEXT, SLATE_HOVER, SLATE_PRESSED, s)
}
pub fn button_danger(_: &Theme, s: button::Status) -> button::Style {
    filled(Some(DANGER), TEXT, DANGER_HOVER, DANGER_PRESSED, s)
}
pub fn button_ghost(_: &Theme, s: button::Status) -> button::Style {
    filled(None, TEXT, GRID, SLATE, s)
}

/// Button that opens a `popup_menu`
pub fn button_menu_trigger(_: &Theme, _: button::Status) -> button::Style {
    button::Style {
        text_color: TEXT,
        border: Border {
            radius: 4.0.into(),
            ..Border::default()
        },
        ..button::Style::default()
    }
}

/// Item inside a popup menu
pub fn button_menu_item(_: &Theme, s: button::Status) -> button::Style {
    let mut base = button::Style {
        text_color: TEXT,
        border: Border {
            radius: 4.0.into(),
            ..Border::default()
        },
        ..button::Style::default()
    };

    match s {
        button::Status::Active => {}
        button::Status::Hovered => base.background = Some(GRID.into()),
        button::Status::Pressed => base.background = Some(SLATE.into()),
        button::Status::Disabled => base.text_color = TEXT_3,
    };

    base
}

pub fn tool_button(_: &Theme, status: button::Status) -> button::Style {
    let mut base = button::Style {
        border: Border {
            radius: 4.0.into(),
            ..Border::default()
        },
        ..button::Style::default()
    };

    if matches!(status, button::Status::Active) {
        base.text_color = TEXT;
    } else {
        base.background = Some(GRID.into());
        base.text_color = ACCENT;
    }

    base
}

pub fn tool_marker<'a, M: 'a>(active: bool) -> Container<'a, M> {
    container(space())
        .width(2)
        .height(Length::Fill)
        .style(move |_| container::Style {
            background: active.then(|| ACCENT.into()),
            ..container::Style::default()
        })
}

pub fn choice_marker<'a, M: 'a>(active: bool) -> Container<'a, M> {
    container(space())
        .height(2)
        .width(Length::Fill)
        .style(move |_| container::Style {
            background: active.then(|| ACCENT.into()),
            ..container::Style::default()
        })
}

pub fn panel(_: &Theme) -> container::Style {
    container::Style {
        background: Some(SURFACE.into()),
        border: Border {
            color: GRID,
            width: 1.0,
            radius: 6.0.into(),
        },
        ..container::Style::default()
    }
}

pub fn focused_panel(_: &Theme) -> container::Style {
    container::Style {
        background: Some(GRID.into()),
        border: Border::default().rounded(6.0),
        ..container::Style::default()
    }
}

// Modal: dialog container plus a dimmed backdrop.
pub fn modal(_: &Theme) -> container::Style {
    container::Style {
        background: Some(SURFACE.into()),
        text_color: Some(TEXT),
        border: Border {
            color: GRID,
            width: 1.0,
            radius: 8.0.into(),
        },
        shadow: Shadow {
            color: MODAL_SHADOW,
            offset: Vector::new(0.0, 12.0),
            blur_radius: 40.0,
        },
        ..container::Style::default()
    }
}
pub fn backdrop(_: &Theme) -> container::Style {
    container::Style {
        background: Some(BACKDROP.into()),
        ..container::Style::default()
    }
}

fn edge(color: Color) -> Border {
    Border {
        color,
        width: 1.0,
        radius: 4.0.into(),
    }
}

/// Closed pick list. Pair with `.menu_style(dropdown_menu)`
pub fn dropdown(_: &Theme, status: pick_list::Status) -> pick_list::Style {
    let (line, handle) = match status {
        pick_list::Status::Active => (GRID, TEXT_2),
        pick_list::Status::Hovered => (SLATE, TEXT),
        pick_list::Status::Opened { .. } => (ACCENT, ACCENT),
    };
    pick_list::Style {
        text_color: TEXT,
        placeholder_color: TEXT_3,
        handle_color: handle,
        background: BG.into(),
        border: edge(line),
    }
}

/// Overlay menu.
pub fn dropdown_menu(_: &Theme) -> menu::Style {
    menu::Style {
        background: SURFACE.into(),
        border: edge(GRID),
        text_color: TEXT,
        selected_text_color: ACCENT,
        selected_background: GRID.into(),
        shadow: Shadow {
            color: MENU_SHADOW,
            offset: Vector::new(0.0, 4.0),
            blur_radius: 12.0,
        },
    }
}

/// Checkbox styling
pub fn check(_: &Theme, status: checkbox::Status) -> checkbox::Style {
    let (on, hot, off) = match status {
        checkbox::Status::Active { is_checked } => (is_checked, false, false),
        checkbox::Status::Hovered { is_checked } => (is_checked, true, false),
        checkbox::Status::Disabled { is_checked } => (is_checked, false, true),
    };
    let fill = match (on, off, hot) {
        (true, true, _) => GRID,
        (true, false, true) => ACCENT_HOVER,
        (true, false, false) => ACCENT,
        _ => BG,
    };
    checkbox::Style {
        background: fill.into(),
        icon_color: if off { TEXT_3 } else { BG },
        border: edge(if on && !off {
            fill
        } else if hot {
            ACCENT
        } else if off {
            GRID
        } else {
            SLATE
        }),
        text_color: Some(if off { TEXT_3 } else { TEXT }),
    }
}

/// Text input styling
pub fn input(_: &Theme, status: text_input::Status) -> text_input::Style {
    let line = match status {
        text_input::Status::Active | text_input::Status::Disabled => GRID,
        text_input::Status::Hovered => SLATE,
        text_input::Status::Focused { .. } => ACCENT,
    };
    text_input::Style {
        background: BG.into(),
        border: edge(line),
        icon: TEXT_2,
        placeholder: TEXT_3,
        value: if matches!(status, text_input::Status::Disabled) {
            TEXT_3
        } else {
            TEXT
        },
        selection: SELECTION,
    }
}

/// Slider styling
pub fn range(_: &Theme, status: slider::Status) -> slider::Style {
    let handle = match status {
        slider::Status::Active => ACCENT,
        slider::Status::Hovered => ACCENT_HOVER,
        slider::Status::Dragged => ACCENT_PRESSED,
    };
    slider::Style {
        rail: slider::Rail {
            backgrounds: (ACCENT.into(), GRID.into()),
            width: 4.0,
            border: Border {
                radius: 2.0.into(),
                ..Border::default()
            },
        },
        handle: slider::Handle {
            shape: slider::HandleShape::Circle { radius: 7.0 },
            background: handle.into(),
            border_width: 0.0,
            border_color: TRANSPARENT,
        },
    }
}

// Toasts: panel-like container with a 3px status bar on the left.

pub fn toast_marker<'a, M: 'a>(colour: Color) -> Element<'a, M> {
    container(space())
        .width(3)
        .height(Length::Fill)
        .style(move |_| container::Style {
            background: Some(colour.into()),
            ..container::Style::default()
        })
        .into()
}
pub fn toast(_: &Theme) -> container::Style {
    container::Style {
        background: Some(SURFACE.into()),
        text_color: Some(TEXT),
        border: edge(GRID),
        ..container::Style::default()
    }
}
