use crate::{Visit, Reflect, Visitor, VisitResult, FieldInfo};
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
    },
    script::{ScriptContext, ScriptTrait, ScriptMessageSender, 
        ScriptMessagePayload, ScriptMessageContext},
};
use std::path::Path;
use gilrs as g;
use gilrs::{
    Gilrs,
    Event as gEvent,
    EventType::*,
    EventType, 
    Axis,
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
}

fn main() {
    println!("hi");
}
