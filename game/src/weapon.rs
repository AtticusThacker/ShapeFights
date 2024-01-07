//This module handles the player's weapon
use crate::*;
use fyrox::scene::transform::Transform;

#[derive(Visit, Reflect, Default, Debug, Clone)]
pub struct Weapon {
    pub player: Handle<Node>,
    pub class: Class,
}

impl_component_provider!(Weapon);

impl TypeUuidProvider for Weapon {
    fn type_uuid() -> Uuid {
        uuid!("bf0f9804-56cb-4a2e-beba-93d75371a568")
    }
}

impl ScriptTrait for Weapon {
    fn on_init(&mut self, _context: &mut ScriptContext) {
        // Put initialization logic here.

    }
    
    fn on_start(&mut self, context: &mut ScriptContext) {
        // subscribe to messages
        context.message_dispatcher.subscribe_to::<Message>(context.handle);

        //setup the correct positioning and visibility of the weapon:

        let offset = match self.class {
            Class::Barbarian => 1.0,
            Class::Fighter => 1.0,
            _ => 0.75,
        };

        context.scene.graph[context.handle.clone()].set_visibility(false);
        //change the local position of the weapon
        if let Some(weapon) = context.scene.graph[context.handle.clone()].cast_mut::<RigidBody>() {
            let axis = Vector3::z_axis();
            //the transform encodes essentially all position information
            let mut starting_transform = Transform::identity();
            //first, change its rotation angle to pi/4 radians (45 degrees)
            starting_transform.set_rotation(UnitQuaternion::from_axis_angle(&axis, -(std::f32::consts::FRAC_PI_2)))
                //these should always be negatives of each other in x and y coords.
                //this sets the position relative to the player
                .set_position(Vector3::new(0.0, offset,0.0))
                //this sets the position of the rotation pivot (the thing it rotates around) to the center of the player
                .set_rotation_pivot(Vector3::new(0.0,-offset,0.0));
            
            weapon.set_local_transform(starting_transform);
        }
    }

    fn on_os_event(&mut self, _event: &Event<()>, _context: &mut ScriptContext) {
        // Respond to OS events here.
    }

    fn on_update(&mut self, ctx: &mut ScriptContext) {
        //get the player state
        let mut state = PlayerState::Idle;
        //get the player state
        if let Some(script) = ctx.scene.graph[self.player].try_get_script::<Player>() {
            state = script.state.clone();
        }
        match state {
            PlayerState::Attacking(frame) => {self.cont_attack(frame, ctx)},
            _ => (),
        }
    }

    ///reads in messages
    fn on_message(&mut self, message: &mut dyn ScriptMessagePayload, ctx: &mut ScriptMessageContext) {
        //downcast the message to the message type we define in messages.rs
        if let Some(message) = message.downcast_ref::<Message>(){
            match message {
                Message::Attack{s} if *s => self.start_melee_attack(ctx),
                Message::Attack{s} if !*s => self.restore_weapon_mes(ctx),
                _ => (),
            }
        }
    }

    fn id(&self) -> Uuid {
        Self::type_uuid()
    }
}

impl Weapon {

    ///restores a weapon to its default position and settings
    pub fn restore_weapon(&self, ctx: &mut ScriptContext) {
        
        if let Some(rigid_body) = ctx.scene.graph[ctx.handle].cast_mut::<RigidBody>(){
            //make weapon invisible
            rigid_body.set_visibility(false);
            //return weapon to starting rotation 
            rigid_body.local_transform_mut()
                .set_rotation(UnitQuaternion::from_axis_angle(&Vector3::z_axis(), -(std::f32::consts::FRAC_PI_2)));
        }
        // find collider
        if let Some((chandle, _)) = ctx.scene.graph.find(ctx.handle, &mut |c| c.is_collider2d()) {
            let collider = ctx.scene.graph[chandle.clone()].as_collider2d_mut();
            //make sure it's a sensor again
            if !collider.is_sensor() {
                collider.set_is_sensor(true);
            }
        }
    }

    ///restores a weapon to its default position and settings, from a message context
    pub fn restore_weapon_mes(&self, ctx: &mut ScriptMessageContext) {
        //make weapon invisible
        if let Some(rigid_body) = ctx.scene.graph[ctx.handle].cast_mut::<RigidBody>(){
            rigid_body.set_visibility(false);
            //return weapon to starting rotation 
            rigid_body.local_transform_mut()
                .set_rotation(UnitQuaternion::from_axis_angle(&Vector3::z_axis(), -(std::f32::consts::FRAC_PI_2)));
        }
        // find collider
        if let Some((chandle, _)) = ctx.scene.graph.find(ctx.handle, &mut |c| c.is_collider2d()) {
            let collider = ctx.scene.graph[chandle.clone()].as_collider2d_mut();
            //make sure it's a sensor again
            if !collider.is_sensor() {
                collider.set_is_sensor(true);
            }
        }

    }

    ///starts a melee attack
    /// called when player messages weapon that a valid attack can be made
    pub fn start_melee_attack(&self, ctx: &mut ScriptMessageContext) {
        if let Some(weapon) = ctx.scene.graph[ctx.handle].cast_mut::<RigidBody>(){
            weapon.set_visibility(true);
        }
    }

    pub fn cont_attack(&self, frame: i32, ctx: &mut ScriptContext) {

        //this doesn't work, maybe later find a good way to boost barbarian attack damage out of a charge
        // let barbdam = match script.state {
        //     PlayerState::Charging => 2 + Class::BARBDAM,
        //     _ => Class::BARBDAM,
        // };
        
        //check if the weapon is visible; if it isn't, then return 
        // (we've been parried)
        if !ctx.scene.graph[ctx.handle].visibility() {return;}
        
        //match for attack constants
        let (interval, lag, spd, dam, knock) = match self.class {
            Class::Barbarian => (Class::BARBINT, Class::BARBLAG, Class::BARBWEPSPD, Class::BARBDAM, Class::BARBKNOCK),
            Class::Rogue => (Class::ROGINT, Class::ROGLAG, Class::ROGWEPSPD, Class::ROGDAM, Class::ROGKNOCK),
            Class::Wizard => (Class::WIZINT, Class::WIZLAG, Class::WIZWEPSPD, Class::WIZDAM, Class::WIZKNOCK),
            Class::Fighter => (Class::FIGINT, Class::FIGLAG, Class::FIGWEPSPD, Class::FIGDAM, Class::FIGKNOCK),
        };

        //while in the attack
        if frame <= interval {
            //continue the swing
            if let Some(weapon) = ctx.scene.graph[ctx.handle].cast_mut::<RigidBody>(){
                //rotate the weapon equal to the weapon speed constant
                let currotation = weapon.local_transform().rotation().clone();
                weapon.local_transform_mut().set_rotation(currotation.append_axisangle_linearized(
                    &(&Vector3::z() * spd)));
            }
            //check for hits
            //let mut hit = (false, self.player);
            //find the collider of the weapon
            if let Some((collider_handle, colnode)) = ctx.scene.graph.find(ctx.handle, &mut |c| c.is_collider2d()) {
                for i in colnode.as_collider2d().intersects(&ctx.scene.graph.physics2d) {
                    //I think a very persistent bug in a previous version of this code arose from 
                    //sending the hit message to the wrong side of the interaction; I'm still
                    //trying to figure out how these intersection pairs work.
                    let other_collider_parent = if i.collider1 == collider_handle {
                        ctx.scene.graph[i.collider2].parent()
                    } else {
                        ctx.scene.graph[i.collider1].parent()
                    };
                    if other_collider_parent == self.player {
                        //stop hitting yourself
                        return;
                    }
                    let parent_node = &ctx.scene.graph[other_collider_parent.clone()];
                    if parent_node.script().is_some() {
                        if matches!(self.class, Class::Fighter) {
                            if let Some(script) = ctx.scene.graph[other_collider_parent].try_get_script::<Player>() {
                                match script.state {
                                    PlayerState::Hit(_) => (),
                                    PlayerState::Dead(_) => (),
                                    _ => {
                                        //tell fighters to increase their charge on a successful hit
                                        ctx.message_sender.send_to_target(self.player,
                                            Message::Charges{i: 1}
                                        );
                                    }
                                }
                            }
                        }

                        let mut knockvec = Vector3::new(1.0,1.0, 1.0);
                        //get the knockback vector
                        if let Some(script) = ctx.scene.graph[self.player].try_get_script::<Player>(){
                            knockvec = script.facing.clone();
                            knockvec.set_magnitude(knock);
                        }


                        ctx.message_sender.send_to_target(other_collider_parent,
                            Message::Hit{
                                damage: dam,
                                knockback: knockvec,
                                sender: ctx.handle,
                            }
                        )  
                    }
                }
            }

        } else if frame < interval + lag {
            //if we're in end lag, don't touch the weapon, just advance the frame
        } else {
            //attack is over
        }
    }



}

//todo: 
// parry fn changes weapon stuff
// takehit (should only be called when a parry is successful!) should return a "parried" message
// on_message should call a parried fn when recieve a parried message
// on_parried fn should get rid of sword 


//barbarian attacks dont benefit from charge