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
    gui::message::UiMessage,
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
            collider::{ColliderShape, ColliderBuilder},
        },
        node::{Node},
        Scene, SceneLoader, SceneContainer,
        graph::{Graph},
        base::BaseBuilder,
        transform::TransformBuilder,
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

    // const BARBWEP:ColliderShape = ColliderShape::cuboid(0.2, 0.7);
    // const ROGWEP:ColliderShape = ColliderShape::cuboid(0.2, 0.7);
    // const WIZWEP:ColliderShape = ColliderShape::cuboid(0.2, 0.7);
    // const FIGWEP:ColliderShape = ColliderShape::cuboid(0.2, 0.7);

    pub fn startup(&self, context: &mut ScriptContext) {
        if let Some(rigid_body) = context.scene.graph[context.handle.clone()].cast_mut::<RigidBody>() {
            let weapontype = match self {
                Class::Barbarian => ColliderShape::cuboid(0.2, 0.7),//Self::BARBWEP,
                Class::Rogue => ColliderShape::cuboid(0.2, 0.7),//Self::ROGWEP,
                Class::Wizard => ColliderShape::cuboid(0.2, 0.7),//Self::WIZWEP,
                Class::Fighter => ColliderShape::cuboid(0.2, 0.7),//Self::FIGWEP,

            };
            let weapon = RigidBodyBuilder::new(BaseBuilder::new().with_children(&[
                // Rigid body must have at least one collider
                ColliderBuilder::new(BaseBuilder::new())
                    .with_shape(weapontype)
                    .with_sensor(true)
                    .build(&mut context.scene.graph),
                RectangleBuilder::new(
                    BaseBuilder::new().with_local_transform(
                        TransformBuilder::new()
                            // Size of the rectangle is defined only by scale.
                            .with_local_scale(Vector3::new(1.0, 1.0, 1.0))
                            .build(),
                    ),
                )
                    .with_texture(context.resource_manager.request::<Texture, _>("data/rcircle.png"))
                    .build(&mut context.scene.graph)
                ]))
            .with_body_type(RigidBodyType::KinematicPositionBased)
            .build(&mut context.scene.graph);

            context.scene.graph[weapon].set_visibility(false);


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
