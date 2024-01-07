use fyrox::scene::node::Node;
use crate::{Handle, Vector3};
use gilrs::ev::EventType;

pub enum Message {
    Hit {
        damage: u32,
        knockback: Vector3<f32>,
        sender: Handle<Node>,
    },
    Controller {
        event: EventType,

    },
    Parried {
        //can eventually hold boolean to tell projectiles to be reflected
    },
    //from a player to a weapon: indicates that a valid attack can be made and should start,
    //or that an attack is over and should end
    Attack{
        s: bool,
    },
    //tell a weapon to start parrying
    Start_Parry{

    },
    //when a player recieves this, they change their "charges" field by the amount inside
    Charges{
        i: i32
    }
}