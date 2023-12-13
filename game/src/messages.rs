use fyrox::scene::node::Node;
use crate::Handle;
use gilrs::ev::EventType;

pub enum Message {
    Hit {
        damage: i32,
        knockback: f32,
    },
    Controller {
        event: EventType,

    },
}