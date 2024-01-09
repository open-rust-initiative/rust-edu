use godot::engine::{ISprite2D, Sprite2D, Texture2D};

use godot::prelude::*;

#[derive(GodotClass)]
#[class(base=Sprite2D)]
pub struct OnsImage {
    id: i64,
    #[base]
    base: Base<Sprite2D>,
}

#[godot_api]
impl ISprite2D for OnsImage {
    fn init(base: Base<Self::Base>) -> Self {
        Self { id: 0, base }
    }
}

#[godot_api]
impl OnsImage {
    #[func]
    pub fn set_id(&mut self, id: Variant) {
        if let Ok(id) = id.try_to::<i64>() {
            self.id = id;
            godot_print!("id set to {}", self.id);
        }
    }
    #[func]
    pub fn on_click(&mut self) {
        if let Some(mut parent) = self.base.get_parent() {
            godot_print!("image {} pressed", self.id);
            parent.call("finish_image".into(), &[Variant::from(self.id)]);
        }
    }
    #[func]
    pub fn set_texture(&mut self, texture: Gd<Texture2D>) {
        self.base.set_texture(texture.into());
        if let Some(mut parent) = self.base.get_parent() {
            parent.call("finish_image".into(), &[Variant::from(self.id)]);
        }
    }

    #[func]
    pub fn set_modulate(&mut self, color: Color) {
        self.base.set_modulate(color);
        if let Some(mut parent) = self.base.get_parent() {
            parent.call("finish_image".into(), &[Variant::from(self.id)]);
        }
    }

    #[func]
    pub fn set_visible(&mut self, visible: bool) {
        self.base.set_visible(visible);
        if let Some(mut parent) = self.base.get_parent() {
            parent.call("finish_image".into(), &[Variant::from(self.id)]);
        }
    }

    #[func]
    pub fn set_position(&mut self, position: Vector2) {
        self.base.set_position(position);
        if let Some(mut parent) = self.base.get_parent() {
            parent.call("finish_image".into(), &[Variant::from(self.id)]);
        }
    }

    #[func]
    pub fn set_scale(&mut self, scale: Vector2) {
        self.base.set_scale(scale);
        if let Some(mut parent) = self.base.get_parent() {
            parent.call("finish_image".into(), &[Variant::from(self.id)]);
        }
    }
}
