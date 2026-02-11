use ratatui::style::Color;

#[derive(Debug, Clone, Copy)]
pub struct Theme {
    pub name: &'static str,
    // Chrome
    pub border: Color,
    pub tab_active: Color,
    pub tab_inactive: Color,
    pub status: Color,
    pub status_error: Color,
    pub help_text: Color,
    pub cursor: Color,
    // Content
    pub text: Color,
    pub text_dim: Color,
    pub selected: Color,
    pub moving: Color,
    // Task states
    pub state_todo: Color,
    pub state_ondeck: Color,
    pub state_inprogress: Color,
    pub state_done: Color,
    // Backlog tree
    pub category: Color,
    pub project: Color,
    // Dialogs
    pub dialog_border: Color,
    pub dialog_text: Color,
    pub dialog_placeholder: Color,
}

pub const DEFAULT: Theme = Theme {
    name: "Default",
    border: Color::DarkGray,
    tab_active: Color::Yellow,
    tab_inactive: Color::Gray,
    status: Color::Green,
    status_error: Color::Red,
    help_text: Color::DarkGray,
    cursor: Color::Yellow,
    text: Color::Gray,
    text_dim: Color::DarkGray,
    selected: Color::White,
    moving: Color::Magenta,
    state_todo: Color::Red,
    state_ondeck: Color::Rgb(100, 149, 237),
    state_inprogress: Color::Yellow,
    state_done: Color::Green,
    category: Color::Yellow,
    project: Color::Cyan,
    dialog_border: Color::Yellow,
    dialog_text: Color::White,
    dialog_placeholder: Color::DarkGray,
};

pub const DRACULA: Theme = Theme {
    name: "Dracula",
    border: Color::Rgb(68, 71, 90),       // comment
    tab_active: Color::Rgb(189, 147, 249), // purple
    tab_inactive: Color::Rgb(98, 114, 164),
    status: Color::Rgb(80, 250, 123),      // green
    status_error: Color::Rgb(255, 85, 85), // red
    help_text: Color::Rgb(98, 114, 164),
    cursor: Color::Rgb(255, 184, 108),     // orange
    text: Color::Rgb(248, 248, 242),       // foreground
    text_dim: Color::Rgb(98, 114, 164),
    selected: Color::Rgb(248, 248, 242),
    moving: Color::Rgb(255, 121, 198),     // pink
    state_todo: Color::Rgb(255, 85, 85),   // red
    state_ondeck: Color::Rgb(139, 233, 253), // cyan
    state_inprogress: Color::Rgb(255, 184, 108), // orange
    state_done: Color::Rgb(80, 250, 123),  // green
    category: Color::Rgb(189, 147, 249),   // purple
    project: Color::Rgb(139, 233, 253),    // cyan
    dialog_border: Color::Rgb(189, 147, 249),
    dialog_text: Color::Rgb(248, 248, 242),
    dialog_placeholder: Color::Rgb(98, 114, 164),
};

pub const CATPPUCCIN_MOCHA: Theme = Theme {
    name: "Catppuccin Mocha",
    border: Color::Rgb(88, 91, 112),       // surface2
    tab_active: Color::Rgb(203, 166, 247), // mauve
    tab_inactive: Color::Rgb(127, 132, 156),
    status: Color::Rgb(166, 227, 161),     // green
    status_error: Color::Rgb(243, 139, 168), // red
    help_text: Color::Rgb(127, 132, 156),
    cursor: Color::Rgb(249, 226, 175),     // yellow
    text: Color::Rgb(205, 214, 244),       // text
    text_dim: Color::Rgb(127, 132, 156),
    selected: Color::Rgb(205, 214, 244),
    moving: Color::Rgb(245, 194, 231),     // pink
    state_todo: Color::Rgb(243, 139, 168), // red
    state_ondeck: Color::Rgb(137, 180, 250), // blue
    state_inprogress: Color::Rgb(249, 226, 175), // yellow
    state_done: Color::Rgb(166, 227, 161), // green
    category: Color::Rgb(203, 166, 247),   // mauve
    project: Color::Rgb(148, 226, 213),    // teal
    dialog_border: Color::Rgb(203, 166, 247),
    dialog_text: Color::Rgb(205, 214, 244),
    dialog_placeholder: Color::Rgb(127, 132, 156),
};

pub const SOLARIZED_LIGHT: Theme = Theme {
    name: "Solarized Light",
    border: Color::Rgb(147, 161, 161),     // base1
    tab_active: Color::Rgb(108, 113, 196), // violet
    tab_inactive: Color::Rgb(147, 161, 161),
    status: Color::Rgb(133, 153, 0),       // green
    status_error: Color::Rgb(220, 50, 47), // red
    help_text: Color::Rgb(147, 161, 161),
    cursor: Color::Rgb(181, 137, 0),       // yellow
    text: Color::Rgb(101, 123, 131),       // base00
    text_dim: Color::Rgb(147, 161, 161),   // base1
    selected: Color::Rgb(7, 54, 66),       // base02
    moving: Color::Rgb(211, 54, 130),      // magenta
    state_todo: Color::Rgb(220, 50, 47),   // red
    state_ondeck: Color::Rgb(38, 139, 210), // blue
    state_inprogress: Color::Rgb(181, 137, 0), // yellow
    state_done: Color::Rgb(133, 153, 0),   // green
    category: Color::Rgb(108, 113, 196),   // violet
    project: Color::Rgb(42, 161, 152),     // cyan
    dialog_border: Color::Rgb(108, 113, 196),
    dialog_text: Color::Rgb(7, 54, 66),
    dialog_placeholder: Color::Rgb(147, 161, 161),
};

const ALL_THEMES: &[Theme] = &[DEFAULT, DRACULA, CATPPUCCIN_MOCHA, SOLARIZED_LIGHT];

impl Theme {
    pub fn all() -> &'static [Theme] {
        ALL_THEMES
    }

    pub fn by_name(name: &str) -> usize {
        ALL_THEMES
            .iter()
            .position(|t| t.name == name)
            .unwrap_or(0)
    }
}
