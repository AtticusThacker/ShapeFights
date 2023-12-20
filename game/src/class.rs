use crate::{Visit, Reflect, Visitor, VisitResult, FieldInfo, 
    RigidBodyType, PlayerState, 
    PlayerState::{Attacking, Idle, Hit},
    Player, Projectile, set_script};
use std::collections::{HashMap, HashSet};
use fyrox::{

    core::{
        pool::Handle,
        algebra::{Vector2, Vector3},
        reflect::prelude::*,
        uuid::{uuid, Uuid},
        visitor::prelude::*, TypeUuidProvider,
        futures::executor::block_on,
    },
    gui::{message::UiMessage,core::algebra::UnitQuaternion,},
    plugin::{Plugin, PluginConstructor, PluginContext, PluginRegistrationContext},
    asset::manager::ResourceManager,
    event::{ElementState, Event, WindowEvent},
    keyboard::KeyCode,
    impl_component_provider,
    resource::texture::Texture,
    scene::{
        dim2::{
            rectangle::{Rectangle, RectangleBuilder}, 
            rigidbody::{RigidBody, RigidBodyBuilder}, 
            collider::{
                ColliderShape, 
                ColliderBuilder,
                CuboidShape,
                TriangleShape,
            },
            joint,
            joint::*
        },
        node::{Node},
        Scene, SceneLoader, SceneContainer,
        graph::{Graph},
        base::BaseBuilder,
        transform::{TransformBuilder, Transform},

        //rigidbody::RigidBodyType,
    },
    script::{ScriptContext, ScriptTrait, ScriptMessageSender, 
        ScriptMessagePayload, ScriptMessageContext},

    engine::ScriptedScene,
};
use std::path::Path;
use gilrs as g;
use gilrs::{
    Gilrs,
    Event as gEvent,
    EventType::*,
    EventType, 
    Axis,
    Button,
    Button::RightTrigger,
};
use fyrox::script::Script;

use crate::messages::{
    Message,
    Message::{Controller, Hit as MessHit, Parried},
};

#[derive(Visit, Reflect, Debug, Clone, Default)]
pub enum Class {
    Barbarian,
    Rogue,
    Wizard,
    #[default]
    Fighter,
}

impl Class {
    //speed of normal movement
    const BARBSPD:f32 = 2.5;
    const ROGSPD:f32 = 5.0;
    const WIZSPD:f32 = 2.5;
    const FIGSPD:f32 = 4.0;

    //shape of weapon (each number is half of the length of one of the sides)
    const BARBWEP:CuboidShape = CuboidShape{half_extents: Vector2::new(0.2,0.35)};
    const ROGWEP:CuboidShape = CuboidShape{half_extents: Vector2::new(0.1,0.3)};
    const WIZWEP:CuboidShape = CuboidShape{half_extents: Vector2::new(0.05,0.3)};
    const FIGWEP:CuboidShape = CuboidShape{half_extents: Vector2::new(0.1,0.45)};

    //number of frames in melee attack
    const BARBINT:i32 = 15;
    const ROGINT:i32 = 15;
    const WIZINT:i32 = 15;
    const FIGINT:i32 = 15;

    //frames of end-lag after melee attack
    const BARBLAG:i32 = 12;
    const ROGLAG:i32 = 12;
    const WIZLAG:i32 = 12;
    const FIGLAG:i32 = 12;

    //number of radians the melee attack should move per frame
    const BARBWEPSPD:f32 = std::f32::consts::PI/20.0;
    const ROGWEPSPD:f32 = std::f32::consts::PI/20.0;
    const WIZWEPSPD:f32 = std::f32::consts::PI/20.0;
    const FIGWEPSPD:f32 = std::f32::consts::PI/20.0;

    //damage done by each class in melee
    const BARBDAM:u32 = 4;
    const ROGDAM:u32 = 2;
    const WIZDAM:u32 = 0;
    const FIGDAM:u32 = 3;

    //knockback done by each class in melee
    const BARBKNOCK:f32 = 3.0;
    const ROGKNOCK:f32 = 3.0;
    const WIZKNOCK:f32 = 3.0;
    const FIGKNOCK:f32 = 3.0;

    //ranged attack speed scalar
    const RATKSPD:f32 = 4.5;

    //special attack speed cooldown (in frames)
    const RCOOL:i32 = 60;
    const CCOOL:i32 = 300;

    //charge length (frames)
    const CHARLEN:i32 = 8;

    //hitstun duration (frames)
    const HITDUR: i32 = 30;

    pub fn startup(&self, script: &mut Player, context: &mut ScriptContext) {

        //setting up the "facing chevron"
        //let mut trans = context.scene.graph[context.handle.clone()].local_transform().clone();
        let mut trans = Transform::identity();
        let mut off = script.facing.clone();
        off.set_magnitude(0.3);
        trans.offset(off);
        let chevron = RigidBodyBuilder::new(BaseBuilder::new().with_children(&[
            RectangleBuilder::new(
                BaseBuilder::new().with_local_transform(
                    TransformBuilder::new()
                        // Size of the rectangle is defined only by scale.
                        .with_local_scale(Vector3::new(0.25,-0.25,0.1))
                        .build()
                )
            )
            .with_texture(context.resource_manager.request::<Texture, _>("data/White_chevron.png"))
            .build(&mut context.scene.graph),
            ColliderBuilder::new(BaseBuilder::new())
                    .with_shape(fyrox::scene::dim2::collider::ColliderShape::Triangle(TriangleShape{
                        a: Vector2::new(0.0,0.25),
                        b: Vector2::new(-0.15,0.0),
                        c: Vector2::new(0.15,0.0),
                    }))
                    .with_sensor(true)
                    .build(&mut context.scene.graph),
            ])
            .with_local_transform(trans)
        )
        .with_body_type(RigidBodyType::KinematicPositionBased)
        .build(&mut context.scene.graph);

        context.scene.graph.link_nodes(chevron, context.handle);
        
        //setting up melee weapon
        if let Some(rigid_body) = context.scene.graph[context.handle.clone()].cast_mut::<RigidBody>() {
            let weapontype = match self {
                Class::Barbarian => Self::BARBWEP,
                Class::Rogue => Self::ROGWEP,
                Class::Wizard => Self::WIZWEP,
                Class::Fighter => Self::FIGWEP,

            };
            let weapon = RigidBodyBuilder::new(BaseBuilder::new().with_children(&[
                RectangleBuilder::new(
                    BaseBuilder::new().with_local_transform(
                        TransformBuilder::new()
                            // Size of the rectangle is defined only by scale.
                            .with_local_scale(Vector3::new(weapontype.half_extents[0].clone()*2.0, weapontype.half_extents[1].clone()*2.0,1.0))
                            .build()
                    )
                )
                    .with_texture(context.resource_manager.request::<Texture, _>("data/white_rectangle.png"))
                    .build(&mut context.scene.graph),
                // Rigid body must have at least one collider
                ColliderBuilder::new(BaseBuilder::new())
                    .with_shape(ColliderShape::Cuboid(weapontype))
                    .with_sensor(true)
                    .build(&mut context.scene.graph),
                
                ]))
            .with_body_type(RigidBodyType::KinematicPositionBased)
            .build(&mut context.scene.graph);


            //set the player's weapon field to this node we've just made
            script.weapon = Some(weapon.clone());

            let offset = match self {
                Class::Barbarian => 1.0,
                Class::Fighter => 1.0,
                _ => 0.75,
            };

            context.scene.graph[weapon.clone()].set_visibility(false);
            //set weapon to be a child of the player
            context.scene.graph.link_nodes(weapon, context.handle);
            //change the local position of the weapon
            if let Some(weapon) = context.scene.graph[weapon.clone()].cast_mut::<RigidBody>() {
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






            //NOTE: I don't think joints are the right thing for this
            // //create joint
            // JointBuilder::new(BaseBuilder::new())
            //     .with_body1(context.handle)
            //     .with_body2(weapon)
            //     .with_params(JointParams::BallJoint(BallJoint {
            //     limits_enabled: false,
            //     limits_angles: Default::default(),
            // }))
            // .build(&mut context.scene.graph);

        }
    }
    








    pub fn moveplayer(&self, script: &mut Player, axis: &Axis, value: &f32, ctx: &mut ScriptMessageContext) {
        if let Some(rigid_body) = ctx.scene.graph[ctx.handle.clone()].cast_mut::<RigidBody>() {
            match (axis, self, script.state.clone()) {
                (_, _, PlayerState::Hit(_)) => {}, //cant move when hit
                (_, _, PlayerState::Charging) => {} //cant change direction while charging

                (g::Axis::LeftStickX, Class::Barbarian, _) => {rigid_body.set_lin_vel(Vector2::new(-value*Self::BARBSPD, rigid_body.lin_vel().y));},
                (g::Axis::LeftStickX, Class::Rogue, _) => {rigid_body.set_lin_vel(Vector2::new(-value*Self::ROGSPD, rigid_body.lin_vel().y));},
                (g::Axis::LeftStickX, Class::Wizard, _) => {rigid_body.set_lin_vel(Vector2::new(-value*Self::WIZSPD, rigid_body.lin_vel().y));},
                (g::Axis::LeftStickX, Class::Fighter, _) => {rigid_body.set_lin_vel(Vector2::new(-value*Self::FIGSPD, rigid_body.lin_vel().y));},
            
                (g::Axis::LeftStickY, Class::Barbarian, _) => {rigid_body.set_lin_vel(Vector2::new(rigid_body.lin_vel().x, value*Self::BARBSPD));},
                (g::Axis::LeftStickY, Class::Rogue, _) => {rigid_body.set_lin_vel(Vector2::new(rigid_body.lin_vel().x, value*Self::ROGSPD));},
                (g::Axis::LeftStickY, Class::Wizard, _) => {rigid_body.set_lin_vel(Vector2::new(rigid_body.lin_vel().x, value*Self::WIZSPD));},
                (g::Axis::LeftStickY, Class::Fighter, _) => {rigid_body.set_lin_vel(Vector2::new(rigid_body.lin_vel().x, value*Self::FIGSPD));},

                //can't turn while attacking or parrying
                (g::Axis::RightStickX, _, PlayerState::Attacking(_)) => {},
                (g::Axis::RightStickY, _, PlayerState::Attacking(_)) => {},
                (g::Axis::RightStickX, _, PlayerState::Parry(_)) => {},
                (g::Axis::RightStickY, _, PlayerState::Parry(_)) => {},

                (g::Axis::RightStickX, _, _) if (value.clone() != 0.0) => {script.facing.x = -*value;},
                (g::Axis::RightStickY, _, _) if (value.clone() != 0.0) => {script.facing.y = *value;},
                _ => (),
            }
        } else {println!("didn't get rigidbody");} 
    }

    pub fn update_look(facing: Vector3<f32>, node: &mut Node) {
        node.local_transform_mut().set_rotation(UnitQuaternion::face_towards(&Vector3::z_axis(), &facing));
    }







    pub fn start_melee_attack(&self, script: &mut Player, ctx: &mut ScriptMessageContext) {
        let atk = match script.state {
            PlayerState::Idle => true,
            PlayerState::Charging => true,
            _ => false
        };
        
        if atk {
            script.state = Attacking(1);
            if let Some(wephandle) = script.weapon {
                if let Some(weapon) = ctx.scene.graph[wephandle.clone()].cast_mut::<RigidBody>(){
                    weapon.set_visibility(true);
                }
            }
        }
    }

    pub fn cont_attack(&self, script: &mut Player, frame: i32, ctx: &mut ScriptContext) {
        let barbdam = match script.state {
            PlayerState::Charging => 2 * Self::BARBDAM,
            _ => Self::BARBDAM,
        };
        
        
        //match for attack constants
        let (interval, lag, spd, dam, knock) = match self {
            Class::Barbarian => (Self::BARBINT, Self::BARBLAG, Self::BARBWEPSPD, barbdam, Self::BARBKNOCK),
            Class::Rogue => (Self::ROGINT, Self::ROGLAG, Self::ROGWEPSPD, Self::ROGDAM, Self::ROGKNOCK),
            Class::Wizard => (Self::WIZINT, Self::WIZLAG, Self::WIZWEPSPD, Self::WIZDAM, Self::WIZKNOCK),
            Class::Fighter => (Self::FIGINT, Self::FIGLAG, Self::FIGWEPSPD, Self::FIGDAM, Self::FIGKNOCK),
        };

        //while in the attack
        if frame <= interval {
            if let Some(wephandle) = script.weapon {
                //check for a hit:
                //find the collider of the weapon
                if let Some((_,colnode)) = ctx.scene.graph.find(wephandle, &mut |c| c.is_collider2d()) {
                    let collider = colnode.as_collider2d();
                    // iterate over collisions
                    for i in collider.intersects(&ctx.scene.graph.physics2d) {
                        //for each active contact
                        if i.has_any_active_contact {
                            //find its parent
                            if let Some((phandle, p)) = ctx.scene.graph.find_up(i.collider1, &mut |c| c.is_rigid_body2d()) {
                                let mut knockvec = script.facing.clone();
                                knockvec.set_magnitude(knock);
                                ctx.message_sender.send_to_target(phandle, Message::Hit{
                                    damage: dam, 
                                    knockback: knockvec,
                                    body: phandle.clone(),
                                    sender: ctx.handle.clone(),
                                });
                                // if let Some(s) = p.as_rigid_body2d().script() {
                                //     if let Some(s) = s.cast::<Player>() {
                                //         println!("hit a player!");
                                //         ctx.message_sender.send_to_target(phandle, Message::Hit{damage: dam, knockback: knock});
                                //     }
                                // }
                            }
                        }
                    }
                }


                if let Some(weapon) = ctx.scene.graph[wephandle.clone()].cast_mut::<RigidBody>(){
                    //rotate the weapon equal to the weapon speed constant
                    let currotation = weapon.local_transform().rotation().clone();
                    weapon.local_transform_mut().set_rotation(currotation.append_axisangle_linearized(
                        &(&Vector3::z() * spd)));
                }
            }
            //advance the current frame
            script.state = Attacking(frame+1);
        } else if frame < interval + lag {
            //if we're in end lag, don't touch the weapon, just advance the frame
            script.state = Attacking(frame+1);
        } else {
            //attack is over
            script.state = Idle;
            //make weapon invisible
            if let Some(wephandle) = script.weapon {
                if let Some(weapon) = ctx.scene.graph[wephandle.clone()].cast_mut::<RigidBody>(){
                    weapon.set_visibility(false);
                    //return weapon to starting rotation 
                    weapon.local_transform_mut()
                        .set_rotation(UnitQuaternion::from_axis_angle(&Vector3::z_axis(), -(std::f32::consts::FRAC_PI_2)));
                }
            }
        }

    }

    pub fn projectiles(&self, script: &mut Player, ctx: &mut ScriptMessageContext) {
        let range_atk = match self{
            Class::Barbarian => {self.start_charge(script, ctx); return()}
            Class::Rogue => return(),
            Class::Fighter if script.charges > 0 => {script.charges -= 1; true}
            Class::Fighter => return(),
            Class::Wizard => true
        };
        
        
        if (script.cooldown > Self::RCOOL && script.state == Idle && range_atk) {
            let mut trans = ctx.scene.graph[ctx.handle.clone()].local_transform().clone();
            let mut dirvec = script.facing.clone();
            dirvec.set_magnitude(1.25);
            trans.offset(dirvec);

            let mut spd = Vector2::new(script.facing[0],script.facing[1]);
            spd.set_magnitude(Self::RATKSPD);

            let proj = RigidBodyBuilder::new(BaseBuilder::new().with_children(&[
                RectangleBuilder::new(
                    BaseBuilder::new().with_local_transform(
                        TransformBuilder::new()
                            // Size of the rectangle is defined only by scale.
                            .with_local_scale(Vector3::new(0.3, 0.5, 1.0))
                            .build()
                    )
                )
                    .with_texture(ctx.resource_manager.request::<Texture, _>("data/white_rectangle.png"))
                    .build(&mut ctx.scene.graph),
                // Rigid body must have at least one collider
                ColliderBuilder::new(BaseBuilder::new())
                    .with_shape(ColliderShape::cuboid(0.15, 0.25))
                    .with_sensor(true)
                    .build(&mut ctx.scene.graph),
                
                ])
                .with_local_transform(trans)
            )
            .with_gravity_scale(0.0)
            .with_lin_vel(spd)
            .with_can_sleep(false)
            .with_ccd_enabled(true)
            .build(&mut ctx.scene.graph);

            set_script(&mut ctx.scene.graph[proj.clone()], 
                        Projectile{facing: script.facing.clone(), hit: false, life: 120}
                        );

            script.cooldown = 0
        }
    }

    pub fn start_charge(&self, script: &mut Player, ctx: &mut ScriptMessageContext) {
        if script.cooldown > Self::CCOOL {
            script.state = PlayerState::Charging;
            
            let mut norm_facing = script.facing.clone();
            norm_facing.set_magnitude(1.0);

            if let Some(rigid_body) = ctx.scene.graph[ctx.handle.clone()].cast_mut::<RigidBody>() {
                rigid_body.set_lin_vel(Vector2::new(norm_facing[0]*6.0*Self::BARBSPD, norm_facing[1]*6.0*Self::BARBSPD));
            }
            script.cooldown = 0;
        }
    }

    pub fn charging(&self, script: &mut Player, ctx: &mut ScriptContext) {
        if script.cooldown >= Self::CHARLEN {
            if let Some(rigid_body) = ctx.scene.graph[ctx.handle.clone()].cast_mut::<RigidBody>() {
                rigid_body.set_lin_vel(Vector2::new(0.0, 0.0));
            }
            script.state = PlayerState::Idle;
        }
    }

    pub fn takehit(&self, script: &mut Player, dam: u32, knock: Vector3<f32>, bod: Handle<Node>, sender: Handle<Node>, ctx: &mut ScriptMessageContext) {
        //check if hit is valid
        if let Some((bhandle, b)) = ctx.scene.graph.find(ctx.handle.clone(), &mut |c| c.instance_id() == ctx.scene.graph[bod].instance_id()) {
            match script.state {
                PlayerState::Hit(_) => {},
                PlayerState::Parry(_) => {
                    if bod == script.weapon.clone().unwrap() {
                        ctx.message_sender.send_to_target(sender, Message::Parried{});
                        script.state = PlayerState::Idle;
                        //put weapon away
                        if let Some(weapon) = ctx.scene.graph[script.weapon.clone().unwrap()].cast_mut::<RigidBody>(){
                            weapon.set_visibility(false);
                            //return weapon to starting rotation 
                            weapon.local_transform_mut()
                                .set_rotation(UnitQuaternion::from_axis_angle(&Vector3::z_axis(), -(std::f32::consts::FRAC_PI_2)));
                        }
                    } else {
                        //take a hit
                        script.state = PlayerState::Hit(0);
                        if script.health <= dam {
                            self.die(script, ctx);
                            script.health = 0;
                        } else {
                            script.health -= dam;
                        }
                        if let Some(rigid_body) = ctx.scene.graph[ctx.handle.clone()].cast_mut::<RigidBody>() {
                            rigid_body.set_lin_vel(Vector2::new(knock.x, knock.y));

                            //fix weapon
                            if let Some(wephandle) = script.weapon {
                                if let Some(weapon) = ctx.scene.graph[wephandle.clone()].cast_mut::<RigidBody>(){
                                    weapon.set_visibility(false);
                                    //return weapon to starting rotation 
                                    weapon.local_transform_mut()
                                        .set_rotation(UnitQuaternion::from_axis_angle(&Vector3::z_axis(), -(std::f32::consts::FRAC_PI_2)));
                                }
                            }
                        }
                    }
                },
                _ => {
                    //take a hit
                    script.state = PlayerState::Hit(0);
                    if script.health <= dam {
                        self.die(script, ctx);
                        script.health = 0;
                    } else {
                        script.health -= dam;
                    }
                    if let Some(rigid_body) = ctx.scene.graph[ctx.handle.clone()].cast_mut::<RigidBody>() {
                        rigid_body.set_lin_vel(Vector2::new(knock.x, knock.y));

                        //fix weapon
                        if let Some(wephandle) = script.weapon {
                            if let Some(weapon) = ctx.scene.graph[wephandle.clone()].cast_mut::<RigidBody>(){
                                weapon.set_visibility(false);
                                //return weapon to starting rotation 
                                weapon.local_transform_mut()
                                    .set_rotation(UnitQuaternion::from_axis_angle(&Vector3::z_axis(), -(std::f32::consts::FRAC_PI_2)));
                            }
                        }
                    }



                }
            }
        }
    }


    pub fn cont_hit(&self, script: &mut Player, frame: i32, context: &mut ScriptContext) {
        //if we're still stunlocked
        if frame < Self::HITDUR {
            let v = context.scene.graph[context.handle.clone()].global_visibility();
            context.scene.graph[context.handle.clone()].set_visibility(!v);
            
            script.state = PlayerState::Hit(frame+1);
        } else {
            context.scene.graph[context.handle.clone()].set_visibility(true);
            script.state = PlayerState::Idle;

            
        }


    }

    pub fn die(&self, script: &mut Player, context: &mut ScriptMessageContext) {
        script.state = PlayerState::Dead;
        context.scene.graph[context.handle.clone()].set_enabled(false);
        context.scene.graph[context.handle.clone()].set_visibility(false);
    }


    pub fn parry(&self, script: &mut Player, context: &mut ScriptMessageContext) {
        match script.state {
            PlayerState::Idle => (),
            _ => {return;}
        }
        //change state to parrying
        script.state = PlayerState::Parry(0);

        //move blade in front and make visible
        let mut weapnode = &mut context.scene.graph[script.weapon.unwrap().clone()];
        weapnode.set_visibility(true);
        if let Some(weapon) = weapnode.cast_mut::<RigidBody>(){
            //rotate the weapon out in front
            weapon.local_transform_mut().set_rotation(UnitQuaternion::from_axis_angle(&Vector3::z_axis(), 0.0));
        }
    }

    pub fn cont_parry(&self, script: &mut Player, frame: i32, context: &mut ScriptContext) {
        if frame == 16 {
            //put blade away
            if let Some(weapon) = context.scene.graph[script.weapon.clone().unwrap()].cast_mut::<RigidBody>(){
                weapon.set_visibility(false);
                //return weapon to starting rotation 
                weapon.local_transform_mut()
                    .set_rotation(UnitQuaternion::from_axis_angle(&Vector3::z_axis(), -(std::f32::consts::FRAC_PI_2)));
            }
            script.state = PlayerState::Parry(17);
        } else if frame == 28 {
            script.state = PlayerState::Idle;
        }  else {
            script.state = PlayerState::Parry(frame+1);
        }

    }

    pub fn parried(&self, script: &mut Player, context: &mut ScriptMessageContext) {
        if let PlayerState::Attacking(_) = script.state {

            //fix weapon
            if let Some(wephandle) = script.weapon {
                if let Some(weapon) = context.scene.graph[wephandle.clone()].cast_mut::<RigidBody>(){
                    weapon.set_visibility(false);
                    //return weapon to starting rotation 
                    weapon.local_transform_mut()
                        .set_rotation(UnitQuaternion::from_axis_angle(&Vector3::z_axis(), -(std::f32::consts::FRAC_PI_2)));
                }
            }
        }
    }

}

fn main() {
    println!("hi");
}
