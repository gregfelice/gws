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

pub const GRUVBOX_DARK: Theme = Theme {
    name: "Gruvbox Dark",
    border: Color::Rgb(80, 73, 69),        // bg3
    tab_active: Color::Rgb(215, 153, 33),  // yellow
    tab_inactive: Color::Rgb(146, 131, 116), // gray
    status: Color::Rgb(152, 151, 26),      // green
    status_error: Color::Rgb(204, 36, 29), // red
    help_text: Color::Rgb(146, 131, 116),
    cursor: Color::Rgb(215, 153, 33),      // yellow
    text: Color::Rgb(235, 219, 178),       // fg
    text_dim: Color::Rgb(146, 131, 116),   // gray
    selected: Color::Rgb(251, 241, 199),   // fg0
    moving: Color::Rgb(177, 98, 134),      // purple
    state_todo: Color::Rgb(204, 36, 29),   // red
    state_ondeck: Color::Rgb(69, 133, 136), // aqua
    state_inprogress: Color::Rgb(215, 153, 33), // yellow
    state_done: Color::Rgb(152, 151, 26),  // green
    category: Color::Rgb(254, 128, 25),    // orange
    project: Color::Rgb(69, 133, 136),     // aqua
    dialog_border: Color::Rgb(215, 153, 33),
    dialog_text: Color::Rgb(235, 219, 178),
    dialog_placeholder: Color::Rgb(146, 131, 116),
};

pub const NORD: Theme = Theme {
    name: "Nord",
    border: Color::Rgb(76, 86, 106),       // nord3
    tab_active: Color::Rgb(136, 192, 208), // nord8 (frost)
    tab_inactive: Color::Rgb(76, 86, 106), // nord3
    status: Color::Rgb(163, 190, 140),     // nord14 (green)
    status_error: Color::Rgb(191, 97, 106), // nord11 (red)
    help_text: Color::Rgb(76, 86, 106),
    cursor: Color::Rgb(235, 203, 139),     // nord13 (yellow)
    text: Color::Rgb(216, 222, 233),       // nord4 (snow storm)
    text_dim: Color::Rgb(76, 86, 106),     // nord3
    selected: Color::Rgb(236, 239, 244),   // nord6
    moving: Color::Rgb(180, 142, 173),     // nord15 (purple)
    state_todo: Color::Rgb(191, 97, 106),  // nord11 (red)
    state_ondeck: Color::Rgb(129, 161, 193), // nord9 (frost)
    state_inprogress: Color::Rgb(235, 203, 139), // nord13 (yellow)
    state_done: Color::Rgb(163, 190, 140), // nord14 (green)
    category: Color::Rgb(136, 192, 208),   // nord8
    project: Color::Rgb(143, 188, 187),    // nord7 (frost)
    dialog_border: Color::Rgb(136, 192, 208),
    dialog_text: Color::Rgb(216, 222, 233),
    dialog_placeholder: Color::Rgb(76, 86, 106),
};

pub const TOKYO_NIGHT: Theme = Theme {
    name: "Tokyo Night",
    border: Color::Rgb(56, 62, 90),        // comment
    tab_active: Color::Rgb(122, 162, 247), // blue
    tab_inactive: Color::Rgb(86, 95, 137), // dark5
    status: Color::Rgb(158, 206, 106),     // green
    status_error: Color::Rgb(247, 118, 142), // red
    help_text: Color::Rgb(86, 95, 137),
    cursor: Color::Rgb(224, 175, 104),     // yellow
    text: Color::Rgb(192, 202, 245),       // foreground
    text_dim: Color::Rgb(86, 95, 137),     // dark5
    selected: Color::Rgb(192, 202, 245),
    moving: Color::Rgb(187, 154, 247),     // purple
    state_todo: Color::Rgb(247, 118, 142), // red
    state_ondeck: Color::Rgb(125, 207, 255), // cyan
    state_inprogress: Color::Rgb(224, 175, 104), // yellow
    state_done: Color::Rgb(158, 206, 106), // green
    category: Color::Rgb(122, 162, 247),   // blue
    project: Color::Rgb(125, 207, 255),    // cyan
    dialog_border: Color::Rgb(122, 162, 247),
    dialog_text: Color::Rgb(192, 202, 245),
    dialog_placeholder: Color::Rgb(86, 95, 137),
};

pub const ROSE_PINE: Theme = Theme {
    name: "Rosé Pine",
    border: Color::Rgb(110, 106, 134),     // muted
    tab_active: Color::Rgb(196, 167, 231), // iris
    tab_inactive: Color::Rgb(110, 106, 134), // muted
    status: Color::Rgb(156, 207, 216),     // foam
    status_error: Color::Rgb(235, 111, 146), // love
    help_text: Color::Rgb(110, 106, 134),
    cursor: Color::Rgb(246, 193, 119),     // gold
    text: Color::Rgb(224, 222, 244),       // text
    text_dim: Color::Rgb(110, 106, 134),   // muted
    selected: Color::Rgb(224, 222, 244),
    moving: Color::Rgb(235, 111, 146),     // love
    state_todo: Color::Rgb(235, 111, 146), // love
    state_ondeck: Color::Rgb(156, 207, 216), // foam
    state_inprogress: Color::Rgb(246, 193, 119), // gold
    state_done: Color::Rgb(156, 207, 216), // foam
    category: Color::Rgb(196, 167, 231),   // iris
    project: Color::Rgb(234, 154, 151),    // rose
    dialog_border: Color::Rgb(196, 167, 231),
    dialog_text: Color::Rgb(224, 222, 244),
    dialog_placeholder: Color::Rgb(110, 106, 134),
};

const ALL_THEMES: &[Theme] = &[
    DEFAULT,
    DRACULA,
    CATPPUCCIN_MOCHA,
    SOLARIZED_LIGHT,
    GRUVBOX_DARK,
    NORD,
    TOKYO_NIGHT,
    ROSE_PINE,
];

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
