use skia_safe::Color4f;

// -- Fonts ------------------------------------------------------------------

pub const FONT_FAMILY_TEXT: &str = "Cascadia Code NF";
pub const FONT_FAMILY_ICON: &str = "sketchybar-app-font";

// -- Bar dimensions ---------------------------------------------------------

pub const BAR_HEIGHT: f32 = 36.0;
pub const BAR_PADDING_H: f32 = 10.0;
pub const BAR_PADDING_V: f32 = 2.0;

// -- Widget pill styles -----------------------------------------------------

pub const WIDGET_BG: Color4f = Color4f::new(1.0, 1.0, 1.0, 0.4); // #ffffff66
pub const WIDGET_BORDER: Color4f = Color4f::new(0.0, 0.0, 0.0, 0.133); // #00000022
pub const WIDGET_BORDER_WIDTH: f32 = 1.0;
pub const WIDGET_RADIUS: f32 = 6.0;
pub const WIDGET_PADDING_H: f32 = 6.0;
#[allow(dead_code)]
pub const WIDGET_PADDING_V: f32 = 2.0;

pub const FOCUSED_BG: Color4f = Color4f::new(0.867, 0.667, 0.8, 0.4); // #DDAACC66
pub const FOCUSED_BORDER: Color4f = Color4f::new(0.867, 0.667, 0.8, 0.4);
pub const MONITOR_DIVIDER_COLOR: Color4f = Color4f::new(0.0, 0.0, 0.0, 0.3);
pub const MONITOR_DIVIDER_WIDTH: f32 = 1.0;
pub const MONITOR_DIVIDER_MARGIN: f32 = 6.0;

#[allow(dead_code)]
pub const BAR_BORDER: Color4f = Color4f::new(0.0, 0.0, 0.0, 0.133); // #00000022
#[allow(dead_code)]
pub const BAR_BORDER_WIDTH: f32 = 1.0;
#[allow(dead_code)]
pub const BAR_RADIUS: f32 = 8.0;

// -- Text colors ------------------------------------------------------------

pub const TEXT_COLOR: Color4f = Color4f::new(0.0, 0.0, 0.0, 1.0); // #000000

// -- Font sizes (Tailwind-style scale) --------------------------------------

pub const FONT_SIZE_2XS: f32 = 8.0; // label / extra small
pub const FONT_SIZE_XS: f32 = 10.0;
pub const FONT_SIZE_SM: f32 = 14.0; // default body
pub const FONT_SIZE_BASE: f32 = 16.0; // time / primary values
pub const FONT_SIZE_LG: f32 = 18.0; // icon
#[allow(dead_code)]
pub const FONT_SIZE_XL: f32 = 20.0;
pub const FONT_SIZE_2XL: f32 = 24.0; // battery icon

// -- Spacing ----------------------------------------------------------------

pub const SECTION_SPACING: f32 = 8.0;
pub const WIDGET_SPACING: f32 = 4.0;
pub const INNER_SPACING: f32 = 4.0;
pub const ICON_ADVANCE_TRIM: f32 = 6.0;
