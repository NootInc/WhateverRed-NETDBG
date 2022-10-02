use egui::{style::Margin, Color32, FontDefinitions, FontFamily, FontId, Stroke, Style, TextStyle};

pub fn get_fonts() -> FontDefinitions {
    let mut fonts = egui::FontDefinitions::default();
    fonts.font_data.insert(
        "Helvetica".to_owned(),
        egui::FontData::from_static(include_bytes!("assets/fonts/Helvetica.ttf")),
    );
    fonts.font_data.insert(
        "Iosevka NF".to_owned(),
        egui::FontData::from_static(include_bytes!("assets/fonts/Iosevka NF.ttf")),
    );
    fonts.font_data.insert(
        "Symbol".to_owned(),
        egui::FontData::from_static(include_bytes!("assets/fonts/Symbol.ttf")),
    );
    fonts.font_data.insert(
        "Apple Symbols".to_owned(),
        egui::FontData::from_static(include_bytes!("assets/fonts/Apple Symbols.ttf")),
    );

    let ent = fonts
        .families
        .entry(egui::FontFamily::Proportional)
        .or_default();
    ent.insert(0, "Helvetica".to_owned());
    ent.insert(1, "Symbol".to_owned());
    ent.insert(2, "Apple Symbols".to_owned());

    let ent = fonts
        .families
        .entry(egui::FontFamily::Monospace)
        .or_default();
    ent.insert(0, "Iosevka NF".to_owned());
    ent.insert(1, "Symbol".to_owned());
    ent.insert(2, "Apple Symbols".to_owned());
    fonts
}

pub fn fix_style(style: &mut Style) {
    style.spacing.menu_margin = Margin::same(6.0);
    style.spacing.window_margin = Margin::same(12.0);
    style.spacing.item_spacing = [12.0, 12.0].into();
    style.spacing.icon_spacing = 6.0;
    style.text_styles = [
        (
            TextStyle::Heading,
            FontId::new(20.0, FontFamily::Proportional),
        ),
        (
            TextStyle::Name("Heading2".into()),
            FontId::new(10.0, FontFamily::Proportional),
        ),
        (
            TextStyle::Name("Context".into()),
            FontId::new(14.0, FontFamily::Proportional),
        ),
        (TextStyle::Body, FontId::new(14.0, FontFamily::Proportional)),
        (
            TextStyle::Monospace,
            FontId::new(14.0, FontFamily::Proportional),
        ),
        (
            TextStyle::Button,
            FontId::new(14.0, FontFamily::Proportional),
        ),
        (
            TextStyle::Small,
            FontId::new(10.0, FontFamily::Proportional),
        ),
    ]
    .into();
    style.visuals.widgets.noninteractive.fg_stroke = Stroke::new(1.0, Color32::WHITE);
    style.visuals.widgets.noninteractive.bg_fill =
        Color32::from_rgba_premultiplied(0x10, 0x10, 0x10, 0xFF);
    style.visuals.widgets.noninteractive.bg_stroke = Stroke::new(
        1.0,
        Color32::from_rgba_premultiplied(0x44, 0x44, 0x44, 0xFF),
    );
}
