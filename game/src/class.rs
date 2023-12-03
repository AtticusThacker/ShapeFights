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
    static BARBSPD:f32 = 1;
    static ROGSPD:f32 = 2;
    static WIZSPD:f32 = 1;
    static FIGSPD:f32 = 1;
    
    fn moveplayer(&self, event: &EventType, ctx: &mut ScriptMessageContext) {
        if let Some(rigid_body) = ctx.scene.graph[ctx.handle.clone()].cast_mut::<RigidBody>() {
            match self {
                Class::Barbarian => {}
            };
        }
    }
}

fn main() {
    println!("hi");
}
