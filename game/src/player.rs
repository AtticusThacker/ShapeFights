//this module contains the data structure + implementation for the Player script, which handles:
// player state
// player actions
// health
use crate::*;

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
    pub weapon: Option<Handle<Node>>,
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
        self.class.clone().startup(self, context);
    }

    // Called whenever there is an event from OS (mouse click, keypress, etc.)
    fn on_os_event(&mut self, _event: &Event<()>, _context: &mut ScriptContext) {}

    // Called every frame at fixed rate of 60 FPS.
    fn on_update(&mut self, context: &mut ScriptContext) {
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
        Class::update_look(self.facing.clone(), &mut context.scene.graph[context.handle.clone()]);

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
                        AxisChanged(axis, value, _code) => self.class.clone().moveplayer(self, axis, value, ctx),
                        //must clone class for any method that takes a 'self' as well.
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