//! A22 semantic theme adapter for GPUI Kit.
//!
//! The product owns the palette and the backend owns its projection onto
//! `gpui-component::Theme`. This keeps application render code independent of
//! the component library's evolving token names.

use gpui_kit::component::ActiveTheme;
use gpui_kit::{
    component::{
        theme::{Theme as KitTheme, ThemeMode},
        Size as KitSize,
    },
    px, rgb, AnyView, App, Global, Hsla, Rgba, SharedString, UpdateGlobal, Window,
};

trait RgbaHsla {
    fn hsla(self) -> Hsla;
}

impl RgbaHsla for Rgba {
    fn hsla(self) -> Hsla {
        self.into()
    }
}

pub const PRIMARY_LIGHT_ID: &str = "a22-primary-light";
pub const PRIMARY_DARK_ID: &str = "a22-primary-dark";

const PRIMARY_RAMP: [u32; 10] = [
    0xf7f3fa, 0xeee8f3, 0xe2d7ea, 0xd2c1de, 0xbca4cd, 0xa68bbc, 0x705697, 0x614c82, 0x503d6d,
    0x3e2f53,
];
const WARM_GRAY_RAMP: [u32; 10] = [
    0xfaf9f7, 0xf4f2ef, 0xeae7e2, 0xded9d2, 0xc9c2b9, 0xada49a, 0x83786d, 0x5e554e, 0x3f3934,
    0x292521,
];

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Color(Hsla);

impl Color {
    pub fn hsla(self) -> Hsla {
        self.0
    }

    pub fn alpha(self, alpha: f32) -> Self {
        Self(self.0.alpha(alpha))
    }

    pub fn contrasting(self) -> Self {
        let luminance = self.0.l * (1.0 - self.0.s * 0.35);
        if luminance > 0.5 {
            Self(rgb(0x292521).hsla())
        } else {
            Self(rgb(0xffffff).hsla())
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ColorName {
    Gray,
    Violet,
    Red,
    Yellow,
    Green,
    Blue,
    Cyan,
    Orange,
    Pink,
    Teal,
}

impl ColorName {
    pub const ALL: [Self; 10] = [
        Self::Gray,
        Self::Violet,
        Self::Red,
        Self::Yellow,
        Self::Green,
        Self::Blue,
        Self::Cyan,
        Self::Orange,
        Self::Pink,
        Self::Teal,
    ];

    pub fn label(self) -> &'static str {
        match self {
            Self::Gray => "gray",
            Self::Violet => "violet",
            Self::Red => "red",
            Self::Yellow => "yellow",
            Self::Green => "green",
            Self::Blue => "blue",
            Self::Cyan => "cyan",
            Self::Orange => "orange",
            Self::Pink => "pink",
            Self::Teal => "teal",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ColorScheme {
    Light,
    Dark,
}

impl ColorScheme {
    pub fn is_dark(self) -> bool {
        matches!(self, Self::Dark)
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub enum Size {
    Xs,
    Sm,
    #[default]
    Md,
    Lg,
    Xl,
}

impl Size {
    pub(crate) fn kit(self) -> KitSize {
        match self {
            // GPUI Kit's control scale is one step denser than the frozen
            // A22 scale. Offset it so A22 xs/sm/md retain their old
            // typography and approximate control heights.
            Self::Xs => KitSize::Small,
            Self::Sm => KitSize::Medium,
            Self::Md => KitSize::Large,
            Self::Lg | Self::Xl => KitSize::Large,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Variant {
    Default,
    Outline,
    Subtle,
    Primary,
    Danger,
    Success,
    Warning,
    Info,
    Link,
    Text,
    Ghost,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FontFamily {
    Onest,
    RobotoFlex,
}

impl FontFamily {
    pub const fn name(self) -> &'static str {
        match self {
            Self::Onest => "Onest",
            Self::RobotoFlex => "Roboto Flex",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TypographySettings {
    pub font_family: FontFamily,
    pub line_height: f32,
}

impl Default for TypographySettings {
    fn default() -> Self {
        Self {
            font_family: FontFamily::Onest,
            line_height: 1.5,
        }
    }
}

impl Global for TypographySettings {}

#[derive(Clone)]
pub struct Theme {
    pub scheme: ColorScheme,
    pub font_family: SharedString,
    pub line_height: f32,
    pub default_radius: f32,
    body: Color,
    inner: KitTheme,
}

impl Theme {
    fn from_inner(inner: &KitTheme, line_height: f32) -> Self {
        let dark = inner.is_dark();
        Self {
            scheme: if dark {
                ColorScheme::Dark
            } else {
                ColorScheme::Light
            },
            font_family: inner.font_family.clone(),
            line_height,
            default_radius: f32::from(inner.radius),
            body: Color(if dark {
                rgb(0x292521).hsla()
            } else {
                rgb(0xfbfaf8).hsla()
            }),
            inner: inner.clone(),
        }
    }

    fn color_from_ramp(name: ColorName, shade: usize) -> Color {
        let value = match name {
            ColorName::Gray => WARM_GRAY_RAMP[shade.min(9)],
            ColorName::Violet => PRIMARY_RAMP[shade.min(9)],
            ColorName::Red => [
                0xfff5f5, 0xffe3e3, 0xffc9c9, 0xffa8a8, 0xff8787, 0xff6666, 0xfa5252, 0xf03e3e,
                0xc92a2a, 0xa51111,
            ][shade.min(9)],
            ColorName::Yellow => [
                0xfff9db, 0xffec99, 0xffd948, 0xffc225, 0xfab005, 0xe67700, 0xd9480f, 0xbf360c,
                0x9a2c0b, 0x7f1d1d,
            ][shade.min(9)],
            ColorName::Green => [
                0xebfbee, 0xb2f2bb, 0x8ce99a, 0x69db7c, 0x40c057, 0x2f9e44, 0x237b35, 0x1b5e20,
                0x14532d, 0x0b3d20,
            ][shade.min(9)],
            ColorName::Blue => [
                0xe7f5ff, 0xa5d8ff, 0x74c0fc, 0x4dabf7, 0x339af0, 0x228be6, 0x1971c2, 0x1864ab,
                0x145591, 0x0b3d91,
            ][shade.min(9)],
            ColorName::Cyan => [
                0xe3fafc, 0x99e9f2, 0x66d9e8, 0x3bc9db, 0x22b8cf, 0x15aabf, 0x1098ad, 0x0c8599,
                0x0b7285, 0x065666,
            ][shade.min(9)],
            ColorName::Orange => [
                0xfff4e6, 0xffd8a8, 0xffc078, 0xffa94d, 0xff922b, 0xfd7e14, 0xe8590c, 0xd9480f,
                0xc2410c, 0x9a3412,
            ][shade.min(9)],
            ColorName::Pink => [
                0xffe3e3, 0xffc9c9, 0xffa8a8, 0xff8787, 0xff6b6b, 0xf06595, 0xd6336c, 0xc2255c,
                0xa61e4d, 0x86123d,
            ][shade.min(9)],
            ColorName::Teal => [
                0xe6fcf5, 0x96f2d7, 0x63e6be, 0x38d9a9, 0x20c997, 0x12b886, 0x0ca678, 0x099268,
                0x087f5b, 0x065f46,
            ][shade.min(9)],
        };
        Color(rgb(value).hsla())
    }

    pub fn body(&self) -> Color {
        self.body
    }

    pub fn surface(&self) -> Color {
        Color(self.inner.group_box)
    }

    pub fn surface_hover(&self) -> Color {
        Color(self.inner.accent)
    }

    pub fn text(&self) -> Color {
        Color(self.inner.foreground)
    }

    pub fn dimmed(&self) -> Color {
        Color(self.inner.muted_foreground)
    }

    pub fn border(&self) -> Color {
        Color(self.inner.border)
    }

    pub fn primary(&self) -> Color {
        Color(self.inner.primary)
    }

    pub fn success(&self) -> Color {
        Color(self.inner.success)
    }

    pub fn warning(&self) -> Color {
        Color(self.inner.warning)
    }

    pub fn danger(&self) -> Color {
        Color(self.inner.danger)
    }

    pub fn info(&self) -> Color {
        Color(self.inner.info)
    }

    pub fn color(&self, name: ColorName, shade: usize) -> Color {
        Self::color_from_ramp(name, shade)
    }

    pub fn font_size(&self, size: Size) -> f32 {
        match size {
            Size::Xs => 12.0,
            Size::Sm => 14.0,
            Size::Md => 16.0,
            Size::Lg => 18.0,
            Size::Xl => 20.0,
        }
    }

    pub fn radius(&self, radius: f32) -> f32 {
        radius
    }
}

fn apply_palette(theme: &mut KitTheme) {
    let dark = theme.is_dark();
    let primary = if dark {
        rgb(0xa68bbc).hsla()
    } else {
        rgb(0x705697).hsla()
    };
    let background = if dark {
        rgb(0x292521).hsla()
    } else {
        rgb(0xfbfaf8).hsla()
    };
    let surface = if dark {
        rgb(0x3f3934).hsla()
    } else {
        rgb(0xffffff).hsla()
    };
    let foreground = if dark {
        rgb(0xfaf9f7).hsla()
    } else {
        rgb(0x292521).hsla()
    };
    let muted = if dark {
        rgb(0xc9c2b9).hsla()
    } else {
        rgb(0x83786d).hsla()
    };
    let border = if dark {
        rgb(0x83786d).hsla()
    } else {
        rgb(0xded9d2).hsla()
    };
    let primary_hover = if dark {
        rgb(0xbca4cd).hsla()
    } else {
        rgb(0x614c82).hsla()
    };
    let primary_active = if dark {
        rgb(0xd2c1de).hsla()
    } else {
        rgb(0x503d6d).hsla()
    };
    let primary_foreground = if dark {
        rgb(0x292521).hsla()
    } else {
        rgb(0xffffff).hsla()
    };
    let secondary = if dark {
        rgb(0x5e554e).hsla()
    } else {
        rgb(0xf4f2ef).hsla()
    };
    let secondary_hover = if dark {
        rgb(0x83786d).hsla()
    } else {
        rgb(0xeae7e2).hsla()
    };
    let secondary_active = if dark {
        rgb(0xada49a).hsla()
    } else {
        rgb(0xded9d2).hsla()
    };
    let danger = if dark {
        rgb(0xff8787).hsla()
    } else {
        rgb(0xfa5252).hsla()
    };
    let danger_hover = if dark {
        rgb(0xfa5252).hsla()
    } else {
        rgb(0xf03e3e).hsla()
    };
    let danger_active = if dark {
        rgb(0xf03e3e).hsla()
    } else {
        rgb(0xc92a2a).hsla()
    };
    let danger_foreground = if dark {
        rgb(0x292521).hsla()
    } else {
        rgb(0xffffff).hsla()
    };
    let success = if dark {
        rgb(0x69db7c).hsla()
    } else {
        rgb(0x40c057).hsla()
    };
    let success_hover = if dark {
        rgb(0x8ce99a).hsla()
    } else {
        rgb(0x2f9e44).hsla()
    };
    let success_active = if dark {
        rgb(0xb2f2bb).hsla()
    } else {
        rgb(0x237b35).hsla()
    };
    let warning = if dark {
        rgb(0xffd43b).hsla()
    } else {
        rgb(0xfab005).hsla()
    };
    let warning_hover = if dark {
        rgb(0xffec99).hsla()
    } else {
        rgb(0xe67700).hsla()
    };
    let warning_active = if dark {
        rgb(0xfff9db).hsla()
    } else {
        rgb(0xd9480f).hsla()
    };
    let info = if dark {
        rgb(0x66d9e8).hsla()
    } else {
        rgb(0x15aabf).hsla()
    };
    let info_hover = if dark {
        rgb(0x99e9f2).hsla()
    } else {
        rgb(0x1098ad).hsla()
    };
    let info_active = if dark {
        rgb(0xc5f6fa).hsla()
    } else {
        rgb(0x0c8599).hsla()
    };

    // GPUI Kit uses `background` as the input/editor surface. Keep the
    // A22 body as a separate adapter token so fields remain white (or warm
    // dark surface) while the page keeps the old warm-gray canvas.
    theme.background = surface;
    theme.foreground = foreground;
    theme.group_box = surface;
    theme.popover = surface;
    theme.popover_foreground = foreground;
    theme.primary = primary;
    theme.primary_hover = primary_hover;
    theme.primary_active = primary_active;
    theme.primary_foreground = primary_foreground;
    theme.button_primary = primary;
    theme.button_primary_hover = primary_hover;
    theme.button_primary_active = primary_active;
    theme.button_primary_foreground = primary_foreground;
    theme.secondary = secondary;
    theme.secondary_hover = secondary_hover;
    theme.secondary_active = secondary_active;
    theme.secondary_foreground = foreground;
    theme.button = secondary;
    theme.button_hover = secondary_hover;
    theme.button_active = secondary_active;
    theme.button_foreground = foreground;
    theme.button_secondary = secondary;
    theme.button_secondary_hover = secondary_hover;
    theme.button_secondary_active = secondary_active;
    theme.button_secondary_foreground = foreground;
    theme.accent = secondary;
    theme.accent_foreground = foreground;
    theme.muted = secondary;
    theme.border = border;
    theme.input = border;
    theme.muted_foreground = muted;
    theme.ring = primary;
    theme.success = success;
    theme.success_hover = success_hover;
    theme.success_active = success_active;
    theme.success_foreground = primary_foreground;
    theme.button_success = success;
    theme.button_success_hover = success_hover;
    theme.button_success_active = success_active;
    theme.button_success_foreground = primary_foreground;
    theme.warning = warning;
    theme.warning_hover = warning_hover;
    theme.warning_active = warning_active;
    theme.warning_foreground = if dark {
        rgb(0x292521).hsla()
    } else {
        foreground
    };
    theme.button_warning = warning;
    theme.button_warning_hover = warning_hover;
    theme.button_warning_active = warning_active;
    theme.button_warning_foreground = if dark {
        rgb(0x292521).hsla()
    } else {
        foreground
    };
    theme.danger = danger;
    theme.danger_hover = danger_hover;
    theme.danger_active = danger_active;
    theme.danger_foreground = danger_foreground;
    theme.button_danger = danger;
    theme.button_danger_hover = danger_hover;
    theme.button_danger_active = danger_active;
    theme.button_danger_foreground = danger_foreground;
    theme.info = info;
    theme.info_hover = info_hover;
    theme.info_active = info_active;
    theme.info_foreground = primary_foreground;
    theme.button_info = info;
    theme.button_info_hover = info_hover;
    theme.button_info_active = info_active;
    theme.button_info_foreground = primary_foreground;
    theme.link = primary;
    theme.link_hover = primary_hover;
    theme.link_active = primary_active;
    theme.selection = primary.alpha(0.22);
    theme.colors.list = surface;
    theme.list_hover = secondary;
    theme.list_active = primary.alpha(0.12);
    theme.list_active_border = primary;
    theme.list_even = background;
    theme.table = surface;
    theme.table_hover = secondary;
    theme.table_active = primary.alpha(0.12);
    theme.table_active_border = primary;
    theme.table_even = background;
    theme.slider_bar = border;
    theme.slider_thumb = surface;
    theme.switch = secondary_hover;
    theme.switch_thumb = surface;
    theme.scrollbar = background;
    theme.scrollbar_thumb = muted;
    theme.scrollbar_thumb_hover = primary;
    theme.caret = primary;
    theme.font_size = px(14.0);
    theme.mono_font_size = px(13.0);
    theme.font_family = "Onest".into();
    theme.radius = gpui_kit::px(4.0);
    theme.radius_lg = gpui_kit::px(16.0);
    theme.shadow = false;
    theme.focus_ring = false;
}

pub fn primary_light_theme() -> Theme {
    let mut inner = KitTheme {
        mode: ThemeMode::Light,
        ..Default::default()
    };
    apply_palette(&mut inner);
    Theme::from_inner(&inner, TypographySettings::default().line_height)
}

pub fn primary_dark_theme() -> Theme {
    let mut inner = KitTheme {
        mode: ThemeMode::Dark,
        ..Default::default()
    };
    apply_palette(&mut inner);
    Theme::from_inner(&inner, TypographySettings::default().line_height)
}

pub fn install_primary_theme_manager(cx: &mut App) {
    cx.set_global(TypographySettings::default());
    crate::fonts::load_fonts(cx);
    KitTheme::change(ThemeMode::Light, None, cx);
    KitTheme::update(cx, apply_palette);
}

pub struct ThemeManager;

impl ThemeManager {
    pub fn set_font_family(font_family: FontFamily, cx: &mut App) {
        TypographySettings::update_global(cx, |settings, _| {
            settings.font_family = font_family;
        });
        KitTheme::update(cx, |theme| {
            theme.font_family = font_family.name().into();
        });
    }

    pub fn set_line_height(line_height: f32, cx: &mut App) {
        let line_height = match line_height {
            value if (value - 1.0).abs() < f32::EPSILON => 1.0,
            value if (value - 1.25).abs() < f32::EPSILON => 1.25,
            _ => 1.5,
        };
        TypographySettings::update_global(cx, |settings, _| {
            settings.line_height = line_height;
        });
        cx.refresh_windows();
    }

    pub fn toggle(cx: &mut App) {
        let font_family = cx
            .try_global::<TypographySettings>()
            .map(|settings| settings.font_family)
            .unwrap_or(FontFamily::Onest);
        let mode = if cx.theme().is_dark() {
            ThemeMode::Light
        } else {
            ThemeMode::Dark
        };
        KitTheme::change(mode, None, cx);
        KitTheme::update(cx, |theme| {
            apply_palette(theme);
            theme.font_family = font_family.name().into();
        });
    }
}

pub fn theme(cx: &App) -> Theme {
    let line_height = cx
        .try_global::<TypographySettings>()
        .map(|settings| settings.line_height)
        .unwrap_or_else(|| TypographySettings::default().line_height);
    Theme::from_inner(cx.theme(), line_height)
}

pub fn tooltip(
    label: impl Into<SharedString>,
) -> impl Fn(&mut Window, &mut App) -> AnyView + 'static {
    let label = label.into();
    move |window, cx| gpui_kit::component::tooltip::Tooltip::new(label.clone()).build(window, cx)
}
