use crate::{Visit, Reflect, Visitor, VisitResult, FieldInfo, RigidBodyType};
use std::collections::HashMap;
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
};
use fyrox::script::Script;

use crate::messages::{
    Message,
    Message::{Controller},
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
    const BARBSPD:f32 = 1.0;
    const ROGSPD:f32 = 2.0;
    const WIZSPD:f32 = 1.0;
    const FIGSPD:f32 = 1.0;

    const BARBWEP:CuboidShape = CuboidShape{half_extents: Vector2::new(0.2,0.7)};
    const ROGWEP:CuboidShape = CuboidShape{half_extents: Vector2::new(0.2,0.7)};
    const WIZWEP:CuboidShape = CuboidShape{half_extents: Vector2::new(0.2,0.7)};
    const FIGWEP:CuboidShape = CuboidShape{half_extents: Vector2::new(0.2,0.7)};

    pub fn startup(&self, context: &mut ScriptContext) {
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

            //context.scene.graph[weapon].set_visibility(false);
            //set weapon to be a child of the player
            context.scene.graph.link_nodes(weapon, context.handle);
            //change the local position of the weapon
            if let Some(weapon) = context.scene.graph[weapon.clone()].cast_mut::<RigidBody>() {
                let axis = Vector3::z_axis();
                //the transform encodes essentially all position information
                let mut starting_transform = Transform::identity();
                //first, change its rotation angle to pi/4 radians (45 degrees)
                starting_transform.set_rotation(UnitQuaternion::from_axis_angle(&axis, -std::f32::consts::FRAC_PI_4))
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









    pub fn pressbutton(&self, button: &Button, ctx: &mut ScriptMessageContext) {
        if let Some(rigid_body) = ctx.scene.graph[ctx.handle.clone()].cast_mut::<RigidBody>() {
            match (button, self) {
                (g::Button::South, Class::Rogue) => {rigid_body.set_lin_vel(Vector2::new(rigid_body.lin_vel().x, Self::ROGSPD));},
                _ => (),
            }
        }
    }

}

fn main() {
    println!("hi");
}
