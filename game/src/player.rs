//this module contains the data structure + implementation for the Player script, which handles:
// player state
// player actions
// health
use crate::*;

use fyrox::script::ScriptMessage;
use gilrs::Axis;

#[derive(Visit, Reflect, Debug, Clone, Default, PartialEq)]
pub enum PlayerState {
    #[default]
    Idle,
    Charging,
    Dead(i32),
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

        //setting up the "facing chevron"
        let chevron = create_facing_chevron(self.facing.clone(), context);

        context.scene.graph.link_nodes(chevron, context.handle);
    }

    // Called whenever there is an event from OS (mouse click, keypress, etc.)
    fn on_os_event(&mut self, _event: &Event<()>, _context: &mut ScriptContext) {}

    // Called every frame at fixed rate of 60 FPS.
    fn on_update(&mut self, context: &mut ScriptContext) {
        //update the various states 
        match self.state {
            PlayerState::Dead(_) => return(), //later, we can use this for a respawn coundown
            PlayerState::Attacking(frame) => {self.check_attack(frame, context)},
            PlayerState::Hit(frame) => {self.cont_hit(frame, context)},

            PlayerState::Charging => {self.class.clone().charging(self, context)},
            PlayerState::Parry(frame) => {self.cont_parry(frame, context)},
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
            PlayerState::Dead(_) => return(),
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
                                RightTrigger => self.start_melee_attack(ctx),
                                LeftTrigger => self.projectiles(ctx),
                                RightThumb => self.parry(ctx),
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

                Hit{damage: dam, knockback: knock, sender: send} => {
                    self.takehit(dam.clone(), knock.clone(), send.clone(), ctx);
                },
                Parried{} => {
                    //self.class.clone().parried(self, ctx)
                },
                Charges{i} => {self.charges += i},
                _ => (),
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

    ///checks if a melee attack can be made, and if so sends a message to weapon
    pub fn start_melee_attack(&mut self, ctx: &mut ScriptMessageContext) {
        //check if the player is in a valid state to start an attack
        let atk = match self.state {
            PlayerState::Idle => true,
            PlayerState::Charging => true,
            _ => false
        };
        
        if atk {
            self.state = PlayerState::Attacking(1);
            ctx.message_sender.send_to_target(self.weapon,
                Message::Attack{s: true});
        }
    }

    ///checks if an attack should continue or end, 
    /// and messages the weapon to stop the attack if it should end
    pub fn check_attack(&mut self, frame: i32, ctx: &mut ScriptContext) {
        let (interval, lag) = match self.class {
            Class::Barbarian => (Class::BARBINT, Class::BARBLAG),
            Class::Rogue => (Class::ROGINT, Class::ROGLAG),
            Class::Wizard => (Class::WIZINT, Class::WIZLAG),
            Class::Fighter => (Class::FIGINT, Class::FIGLAG),
        };
        //while in the attack
        if frame <= interval {
            //advance the current frame
            self.state = PlayerState::Attacking(frame+1)
        } else if frame < interval + lag {
            //if we're in end lag, don't touch the weapon, just advance the frame
            self.state = PlayerState::Attacking(frame+1)
        } else {
            //attack is over
            self.state = PlayerState::Idle;
            ctx.message_sender.send_to_target(self.weapon, Message::Attack{s: false});
        }
    }

    /// called when the player has been hit by an attack.
    pub fn takehit(&mut self, dam: u32, knock: Vector3<f32>, _send: Handle<Node>, ctx: &mut ScriptMessageContext) {
        //if currently hit or dead, return
        match self.state {
            PlayerState::Hit(_) => return,
            PlayerState::Dead(_) => return,
            _=> (),
        }
        //tell game to update health
        if let Some(game) = ctx.plugins[0].cast_mut::<Game>() {
            game.phealthchanged = true;
        }

        //take damage, die if necessary
        if self.health <= dam {
            self.die(ctx);
            self.health = 0;
            return;
        } else {
            self.health -= dam;
            //set status to Hit
            self.state = PlayerState::Hit(0);
        }
        //take knockback
        if let Some(rigid_body) = ctx.scene.graph[ctx.handle.clone()].cast_mut::<RigidBody>() {
            rigid_body.set_lin_vel(Vector2::new(knock.x, knock.y));

        }
        //tell weapon to vanish
        ctx.message_sender.send_to_target(self.weapon,
            Message::Attack{s: false}
        );

    }

    pub fn die(&mut self, context: &mut ScriptMessageContext) {
        self.state = PlayerState::Dead(100); //respawn time later?
        context.message_sender.send_to_target(self.weapon,
            Message::Attack{s: false}
        );
        context.scene.graph[context.handle].set_enabled(false);
        context.scene.graph[context.handle].set_visibility(false);
    }

    ///called every frame while the player is hit
    pub fn cont_hit(&mut self, frame: i32, context: &mut ScriptContext) {
        if frame < Class::HITDUR {
            //if player is still stunlocked
            let v = context.scene.graph[context.handle.clone()].global_visibility();
            context.scene.graph[context.handle.clone()].set_visibility(!v);
            
            self.state = PlayerState::Hit(frame+1);
            //otherwise, 
        } else {
            context.scene.graph[context.handle.clone()].set_visibility(true);
            self.state = PlayerState::Idle;
        }
    }

    ///called when the player starts a parry
    pub fn parry(&mut self, ctx: &mut ScriptMessageContext) {
        //check if player can parry
        match self.state {
            PlayerState::Idle => (),
            _ => {return;}
        }

        //change state to parrying
        self.state = PlayerState::Parry(0);

        //tell weapon to start parrying
        ctx.message_sender.send_to_target(self.weapon, 
            Message::Start_Parry{}
        );
    }

    pub fn cont_parry(&mut self, frame: i32, ctx: &mut ScriptContext) {
        if frame == 16 {
            //put blade away
            ctx.message_sender.send_to_target(self.weapon, 
                Message::Attack{s: false}
            );
            self.state = PlayerState::Parry(frame+1);
        } else if frame == 28 {
            self.state = PlayerState::Idle;
        }  else {
            self.state = PlayerState::Parry(frame+1);
        }
    }

    pub fn projectiles(&mut self, ctx: &mut ScriptMessageContext) {
        match self.class{
            Class::Barbarian => {
                //self.start_charge(script, ctx); 
                return;
            }
            Class::Rogue => {
                //self.riposte(script, ctx); 
                return;}
            Class::Fighter if self.charges > 0 => {self.charges -= 1;}
            Class::Fighter => {return;},
            Class::Wizard => {},
        };
        
        
        if self.cooldown > Class::RCOOL && self.state == PlayerState::Idle {
            //create projectile
            let proj = create_projectile(self.facing, ctx);
            // set its script
            set_script(&mut ctx.scene.graph[proj.clone()], 
                        Projectile{facing: self.facing.clone(), hit: false, life: 120}
                        );

            self.cooldown = 0
        }
    }

}


//todo: 
//start parry fn should do player parry things
// cont parry fn should advance frames and end the parry, send weapn message when parry ends
// 
// projectiles eventually
//respawning eventually?