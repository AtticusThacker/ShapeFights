use fyrox::scene::node::Node;
use crate::Handle;
use gilrs::ev::EventType;

pub enum Message {
    Damage {
        actor: Handle<Node>,
        attacker: Handle<Node>,
    },
    Controller {
        event: EventType,

    },
}