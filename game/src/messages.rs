use fyrox::scene::node::Node;
use crate::{Handle, Vector3};
use gilrs::ev::EventType;

pub enum Message {
    Hit {
        damage: i32,
        knockback: Vector3<f32>,
        body: Handle<Node>,
    },
    Controller {
        event: EventType,

    },
}