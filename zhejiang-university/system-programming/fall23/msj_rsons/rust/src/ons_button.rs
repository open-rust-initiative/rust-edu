use godot::engine::{ITextureButton, TextureButton};

use godot::prelude::*;

#[derive(GodotClass)]
#[class(base=TextureButton)]
pub struct OnsButton {
    id: i64,
    #[base]
    base: Base<TextureButton>,
}

#[godot_api]
impl ITextureButton for OnsButton {
    fn init(base: Base<Self::Base>) -> Self {
        Self { id: 0, base }
    }
}

#[godot_api]
impl OnsButton {
    #[func]
    pub fn set_id(&mut self, id: Variant) {
        if let Ok(id) = id.try_to::<i64>() {
            self.id = id;
            godot_print!("id set to {}", self.id);
            let callable = self.base.callable("on_button_down");
            self.base.connect("button_down".into(), callable);
        }
    }
    #[func]
    pub fn on_button_down(&mut self) {
        if let Some(mut parent) = self.base.get_parent() {
            godot_print!("button {} pressed", self.id);
            parent.call("finish_button".into(), &[Variant::from(self.id)]);
        }
    }
}
