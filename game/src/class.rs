use crate::{Visit, Reflect, Visitor, VisitResult, FieldInfo, 
    RigidBodyType, PlayerState, 
    PlayerState::{Attacking, Idle}, Player};
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
    Message::{Controller, Hit},
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
    const BARBSPD:f32 = 1.0;
    const ROGSPD:f32 = 2.0;
    const WIZSPD:f32 = 1.0;
    const FIGSPD:f32 = 1.0;

    //shape of weapon (each number is half of the length of one of the sides)
    const BARBWEP:CuboidShape = CuboidShape{half_extents: Vector2::new(0.2,0.7)};
    const ROGWEP:CuboidShape = CuboidShape{half_extents: Vector2::new(0.2,0.7)};
    const WIZWEP:CuboidShape = CuboidShape{half_extents: Vector2::new(0.2,0.7)};
    const FIGWEP:CuboidShape = CuboidShape{half_extents: Vector2::new(0.2,0.7)};

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

    //damage done by each class in melee; not implemented yet
    const BARBDAM:i32 = 12;
    const ROGDAM:i32 = 12;
    const WIZDAM:i32 = 12;
    const FIGDAM:i32 = 12;

    //knockback done by each class in melee; not implemented yet
    const BARBKNOCK:f32 = 12.0;
    const ROGKNOCK:f32 = 12.0;
    const WIZKNOCK:f32 = 12.0;
    const FIGKNOCK:f32 = 12.0;

    pub fn startup(&self, script: &mut Player, context: &mut ScriptContext) {
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
                            .with_local_scale(Vector3::new(weapontype.half_extents[0].clone(), weapontype.half_extents[1].clone(),1.0))
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
                    .set_position(Vector3::new(0.0,0.5,0.0))
                    //this sets the position of the rotation pivot (the thing it rotates around) to the center of the player
                    .set_rotation_pivot(Vector3::new(0.0,-0.5,0.0));
                
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
    








    pub fn moveplayer(&self, axis: &Axis, value: &f32, ctx: &mut ScriptMessageContext) {
        if let Some(rigid_body) = ctx.scene.graph[ctx.handle.clone()].cast_mut::<RigidBody>() {
            match (axis, self) {
                (g::Axis::LeftStickX, Class::Barbarian) => {rigid_body.set_lin_vel(Vector2::new(-value*Self::BARBSPD, rigid_body.lin_vel().y));},
                (g::Axis::LeftStickX, Class::Rogue) => {rigid_body.set_lin_vel(Vector2::new(-value*Self::ROGSPD, rigid_body.lin_vel().y));},
                (g::Axis::LeftStickX, Class::Wizard) => {rigid_body.set_lin_vel(Vector2::new(-value*Self::WIZSPD, rigid_body.lin_vel().y));},
                (g::Axis::LeftStickX, Class::Fighter) => {rigid_body.set_lin_vel(Vector2::new(-value*Self::FIGSPD, rigid_body.lin_vel().y));},
            
                (g::Axis::LeftStickY, Class::Barbarian) => {rigid_body.set_lin_vel(Vector2::new(rigid_body.lin_vel().x, value*Self::BARBSPD));},
                (g::Axis::LeftStickY, Class::Rogue) => {rigid_body.set_lin_vel(Vector2::new(rigid_body.lin_vel().x, value*Self::ROGSPD));},
                (g::Axis::LeftStickY, Class::Wizard) => {rigid_body.set_lin_vel(Vector2::new(rigid_body.lin_vel().x, value*Self::WIZSPD));},
                (g::Axis::LeftStickY, Class::Fighter) => {rigid_body.set_lin_vel(Vector2::new(rigid_body.lin_vel().x, value*Self::FIGSPD));},

                _ => (),
            }
        } else {println!("didn't get rigidbody");} 
    }









    pub fn start_melee_attack(&self, script: &mut Player, ctx: &mut ScriptMessageContext) {
        if let Idle = script.state{
            script.state = Attacking(1);
            if let Some(wephandle) = script.weapon {
                if let Some(weapon) = ctx.scene.graph[wephandle.clone()].cast_mut::<RigidBody>(){
                    weapon.set_visibility(true);
                }
            }
        }
    }

    pub fn cont_attack(&self, script: &mut Player, frame: i32, ctx: &mut ScriptContext) {
        //match for attack constants
        let (interval, lag, spd, dam, knock) = match self {
            Class::Barbarian => (Self::BARBINT, Self::BARBLAG, Self::BARBWEPSPD, Self::BARBDAM, Self::BARBKNOCK),
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
                                if let Some(s) = p.as_rigid_body2d().script() {
                                    if let Some(s) = s.cast::<Player>() {
                                        println!("hit a player!");
                                        ctx.message_sender.send_to_target(phandle, Message::Hit{damage: dam, knockback: knock});
                                    }
                                }
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

    pub fn projectile(&self, ctx: &mut ScriptMessageContext) {
        //let proj = create_kinematic_rigid_body(&mut ctx.scene.graph, ctx);

        let proj = RigidBodyBuilder::new(BaseBuilder::new().with_children(&[
                    RectangleBuilder::new(
                        BaseBuilder::new().with_local_transform(
                            TransformBuilder::new()
                                // Size of the rectangle is defined only by scale.
                                .with_local_scale(Vector3::new(0.4, 0.2, 1.0))
                                .build(),
                        ),
                    )
                    .with_texture(ctx.resource_manager.request::<Texture, _>("data/rcircle.png"))
                    .build(&mut ctx.scene.graph),
                        // Rigid body must have at least one collider
                        ColliderBuilder::new(BaseBuilder::new())
                            .with_shape(ColliderShape::cuboid(0.5, 0.5))
                            .with_sensor(true)
                            .build(&mut ctx.scene.graph),
                    ]))
                .with_body_type(RigidBodyType::KinematicVelocityBased)
                .build(&mut ctx.scene.graph);




        ctx.scene.graph.link_nodes(proj, ctx.handle);

        if let Some(rigid_body) = ctx.scene.graph[proj.clone()].cast_mut::<RigidBody>() {
            rigid_body.set_lin_vel(Vector2::new(1.0, 0.0));
        }
    }

}

fn create_kinematic_rigid_body(graph: &mut Graph, ctx: &mut ScriptMessageContext) -> Handle<Node> {
    RigidBodyBuilder::new(BaseBuilder::new().with_children(&[
        RectangleBuilder::new(
            BaseBuilder::new().with_local_transform(
                TransformBuilder::new()
                    // Size of the rectangle is defined only by scale.
                    .with_local_scale(Vector3::new(0.4, 0.2, 1.0))
                    .build(),
            ),
        )
        .with_texture(ctx.resource_manager.request::<Texture, _>("data/rcircle.png"))
        .build(graph),
            // Rigid body must have at least one collider
            ColliderBuilder::new(BaseBuilder::new())
                .with_shape(ColliderShape::cuboid(0.5, 0.5))
                .with_sensor(true)
                .build(graph),
        ]))
    .with_body_type(RigidBodyType::KinematicVelocityBased)
    .build(graph)
}

fn main() {
    println!("hi");
}
