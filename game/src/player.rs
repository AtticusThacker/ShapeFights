//this module contains the data structure + implementation for the Player script, which handles:
// player state
// player actions
// health
use crate::*;

use gilrs::Axis;

#[derive(Visit, Reflect, Debug, Clone, Default, PartialEq)]
pub enum PlayerState {
    #[default]
    Idle,
    Charging,
    Dead,
    Riposting,
    //the field holds the number of frames the player is into the action
    Attacking(i32),
    Hit(i32),
    Parry(i32),
}

#[derive(Visit, Reflect, Debug, Clone, Default)]
pub struct Player{
    pub class: Class,
    pub state: PlayerState,
    pub weapon: Handle<Node>,
    pub cooldown: i32,
    pub facing: Vector3<f32>, //z axis should always be 0.0 here!
    pub health: u32,
    pub charges: i32,
}

impl_component_provider!(Player,);

impl TypeUuidProvider for Player {
    // Returns unique script id for serialization needs.
    fn type_uuid() -> Uuid {
        uuid!("c5671d19-9f1a-4286-8486-add4ebaadaec")
    }
}

impl ScriptTrait for Player {
    // Called once at initialization.
    fn on_init(&mut self, _context: &mut ScriptContext) {}
    
    // Put start logic - it is called when every other script is already initialized.
    fn on_start(&mut self, context: &mut ScriptContext) { 
        context.message_dispatcher.subscribe_to::<Message>(context.handle);
        //self.class.clone().startup(self, context);

        //tell game to update health
        if let Some(game) = context.plugins[0].cast_mut::<Game>() {
            game.phealthchanged = true;
        }
    }

    // Called whenever there is an event from OS (mouse click, keypress, etc.)
    fn on_os_event(&mut self, _event: &Event<()>, _context: &mut ScriptContext) {}

    // Called every frame at fixed rate of 60 FPS.
    fn on_update(&mut self, context: &mut ScriptContext) {
        //update the various states 
        match self.state {
            PlayerState::Dead => return(),
            PlayerState::Attacking(frame) => {self.class.clone().cont_attack(self, frame, context)},
            PlayerState::Hit(frame) => {self.class.clone().cont_hit(self, frame, context)},

            PlayerState::Charging => {self.class.clone().charging(self, context)},
            PlayerState::Parry(frame) => {self.class.clone().cont_parry(self, frame, context)},
            PlayerState::Riposting => {self.class.clone().riposting(self, context)}
            _ => (),
        }

        match self.class {
            Class::Barbarian if self.cooldown == 8 => {
                if let Some(rigid_body) = context.scene.graph[context.handle.clone()].cast_mut::<RigidBody>(){
                    rigid_body.set_lin_vel(Vector2::new(0.0, 0.0));
                }
            }
            _ => {},
        };

        self.cooldown += 1;
        //make the player face towards the facing vector
        Self::update_look(self.facing.clone(), &mut context.scene.graph[context.handle.clone()]);

    }






    fn on_message(&mut self,
        message: &mut dyn ScriptMessagePayload,
        ctx: &mut ScriptMessageContext,
    ) {
        match self.state {
            PlayerState::Dead => return(),
            _ => {}
        }

        if let Some(message) = message.downcast_ref::<Message>(){
            match message {
                Controller{event} => {
                    match event {
                        // put the various controller events here, as well as calls to
                        //the correct class methods-- player has a class field now!
                        AxisChanged(axis, value, _code) => self.moveplayer(axis, value, ctx),
                        ButtonPressed(button, _) => {
                            match button {
                                RightTrigger => self.class.clone().start_melee_attack(self, ctx),
                                LeftTrigger => self.class.clone().projectiles(self, ctx),
                                RightThumb => self.class.clone().parry(self, ctx),
                                _ => (),
                            }},
                        // ButtonPressed(RightTrigger, _) => self.class.clone().start_melee_attack(self, ctx),
                        // //projectiles
                        // ButtonPressed(LeftTrigger, _) => self.class.clone().projectiles(self, ctx),
                        // //parrying
                        // ButtonPressed(RightThumb, _) => self.class.clone().parry(self, ctx),
                        _ => (),
                    }
                },

                Hit{damage: dam, knockback: knock, body: bod, sender: send} => {
                    self.class.clone().takehit(self, dam.clone(), knock.clone(), bod.clone(), send.clone(), ctx);
                },
                Parried{} => {self.class.clone().parried(self, ctx)},
            }
        }
    }

    // Returns unique script ID for serialization needs.
    fn id(&self) -> Uuid {
        Self::type_uuid()
    }
}


impl Player {
    
    pub fn moveplayer(&mut self, axis: &Axis, value: &f32, ctx: &mut ScriptMessageContext) {
        if let Some(rigid_body) = ctx.scene.graph[ctx.handle.clone()].cast_mut::<RigidBody>() {
            match (axis, &self.class, self.state.clone()) {
                (_, _, PlayerState::Hit(_)) => {}, //cant move when hit
                (_, _, PlayerState::Charging) => {} //cant change direction while charging

                (g::Axis::LeftStickX, Class::Barbarian, _) => {rigid_body.set_lin_vel(Vector2::new(-value*Class::BARBSPD, rigid_body.lin_vel().y));},
                (g::Axis::LeftStickX, Class::Rogue, _) => {rigid_body.set_lin_vel(Vector2::new(-value*Class::ROGSPD, rigid_body.lin_vel().y));},
                (g::Axis::LeftStickX, Class::Wizard, _) => {rigid_body.set_lin_vel(Vector2::new(-value*Class::WIZSPD, rigid_body.lin_vel().y));},
                (g::Axis::LeftStickX, Class::Fighter, _) => {rigid_body.set_lin_vel(Vector2::new(-value*Class::FIGSPD, rigid_body.lin_vel().y));},
            
                (g::Axis::LeftStickY, Class::Barbarian, _) => {rigid_body.set_lin_vel(Vector2::new(rigid_body.lin_vel().x, value*Class::BARBSPD));},
                (g::Axis::LeftStickY, Class::Rogue, _) => {rigid_body.set_lin_vel(Vector2::new(rigid_body.lin_vel().x, value*Class::ROGSPD));},
                (g::Axis::LeftStickY, Class::Wizard, _) => {rigid_body.set_lin_vel(Vector2::new(rigid_body.lin_vel().x, value*Class::WIZSPD));},
                (g::Axis::LeftStickY, Class::Fighter, _) => {rigid_body.set_lin_vel(Vector2::new(rigid_body.lin_vel().x, value*Class::FIGSPD));},

                //can't turn while attacking or parrying
                (g::Axis::RightStickX, _, PlayerState::Attacking(_)) => {},
                (g::Axis::RightStickY, _, PlayerState::Attacking(_)) => {},
                (g::Axis::RightStickX, _, PlayerState::Parry(_)) => {},
                (g::Axis::RightStickY, _, PlayerState::Parry(_)) => {},

                (g::Axis::RightStickX, _, _) if (value.clone() != 0.0) => {self.facing.x = -*value;},
                (g::Axis::RightStickY, _, _) if (value.clone() != 0.0) => {self.facing.y = *value;},
                _ => (),
            }
        } else {println!("didn't get rigidbody");} 
    }

    pub fn update_look(facing: Vector3<f32>, node: &mut Node) {
        node.local_transform_mut().set_rotation(UnitQuaternion::face_towards(&Vector3::z_axis(), &facing));
    }




}