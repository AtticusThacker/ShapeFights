use fyrox::scene::node::Node;
use crate::{Handle, Vector3};
use gilrs::ev::EventType;

pub enum Message {
    Hit {
        damage: u32,
        knockback: Vector3<f32>,
        body: Handle<Node>,
        sender: Handle<Node>,
    },
    Controller {
        event: EventType,

    },
    Parried {
        //can eventually hold boolean to tell projectiles to be reflected
    }
}