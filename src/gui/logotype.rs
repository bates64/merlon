use egui::*;
use egui_extras::RetainedImage;
use serde::{Deserialize, Serialize};

const PNG_BYTES: &[u8] = include_bytes!("../../assets/logo/logotype.png");

/// Merlon logotype. Displays a heading if the image fails to load.
#[derive(Default)]
pub struct Logotype {
    state: State,
}

enum State {
    Unloaded,
    Failed,
    Loaded {
        image: RetainedImage,
        texture_id: TextureId,
    }
}

impl Logotype {
    pub fn load_if_first_attempt(&mut self, ctx: &Context) {
        if let State::Unloaded = self.state {
            self.state = match RetainedImage::from_image_bytes("logotype", PNG_BYTES) {
                Ok(image) => State::Loaded {
                    texture_id: image.texture_id(ctx),
                    image,
                },
                Err(err) => {
                    log::error!("failed loading logotype image: {}", err);
                    State::Failed
                }
            };
        }
    }
}

impl Widget for &mut Logotype {
    fn ui(self, ui: &mut Ui) -> Response {
        match &self.state {
            State::Loaded { image, texture_id } => {
                ui.image(*texture_id, image.size_vec2())
            }
            _ => ui.heading("Merlon"),
        }
    }
}

// Always serialize None so that the logotype is not saved to disk.
impl Serialize for Logotype {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_none()
    }
}

// Deserialize to default, regardless of input.
impl<'de> Deserialize<'de> for Logotype {
    fn deserialize<D: serde::Deserializer<'de>>(_: D) -> Result<Self, D::Error> {
        Ok(Self::default())
    }
}

impl Default for State {
    fn default() -> Self {
        State::Unloaded
    }
}
