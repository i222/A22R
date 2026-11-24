use std::borrow::Cow;

use gpui_kit::App;

const ONEST_VARIABLE_FONT: &[u8] = include_bytes!("../assets/fonts/Onest[wght].ttf").as_slice();
const ROBOTO_FLEX_VARIABLE_FONT: &[u8] =
    include_bytes!("../assets/fonts/RobotoFlex[variable].ttf").as_slice();

/// Register the bundled comparison fonts with GPUI's text system.
///
/// The fonts are embedded so the product does not depend on a user-installed
/// copy or silently fall back to a different family on Ventura.
pub fn load_fonts(cx: &App) {
    cx.text_system()
        .add_fonts(vec![
            Cow::Borrowed(ONEST_VARIABLE_FONT),
            Cow::Borrowed(ROBOTO_FLEX_VARIABLE_FONT),
        ])
        .expect("failed to register the bundled typography fonts");
}
