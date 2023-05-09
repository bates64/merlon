use egui::*;

/// Hyperlink to the support page.
/// Styled with a heart icon and a pink colour.
pub struct SupportLink;

const PINK: Color32 = Color32::from_rgb(0xea, 0x7a, 0xa1); // #ea7aa1
const URL: &'static str = "https://github.com/sponsors/nanaian";

impl Widget for SupportLink {
    fn ui(self, ui: &mut Ui) -> Response {
        let text = RichText::new(format!("{} Support development", egui_phosphor::HEART)).color(PINK); // FIXME icon
        let label = Label::new(text).sense(Sense::click());
        let (pos, text_galley, response) = label.layout_in_ui(ui);

        // Hand icon on hover
        if response.hovered() {
            ui.ctx().set_cursor_icon(CursorIcon::PointingHand);
        }

        // Draw underline on hover/focus
        if ui.is_rect_visible(response.rect) {
            let color = PINK;
            let visuals = ui.style().interact(&response);

            let underline = if response.hovered() || response.has_focus() {
                Stroke::new(visuals.fg_stroke.width, color)
            } else {
                Stroke::NONE
            };

            ui.painter().add(epaint::TextShape {
                pos,
                galley: text_galley.galley,
                override_text_color: Some(color),
                underline,
                angle: 0.0,
            });
        }

        // On click, open URL
        if response.clicked() {
            let modifiers = ui.ctx().input(|i| i.modifiers);
            ui.ctx().output_mut(|o| {
                o.open_url = Some(egui::output::OpenUrl {
                    url: URL.to_string(),
                    new_tab: modifiers.any(),
                });
            });
        }
        if response.middle_clicked() {
            ui.ctx().output_mut(|o| {
                o.open_url = Some(egui::output::OpenUrl {
                    url: URL.to_string(),
                    new_tab: true,
                });
            });
        }

        response
    }
}
