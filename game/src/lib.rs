//! Game project
#![allow(nonstandard_style)]
use std::{
    collections::HashMap,
    vec::Vec,
    path::Path,
};

use fyrox::{
    script::{Script, ScriptContext, ScriptTrait, ScriptMessageSender, ScriptMessagePayload, ScriptMessageContext},
    plugin::{Plugin, PluginConstructor, PluginContext, PluginRegistrationContext},
    asset::manager::ResourceManager,
    event::Event,
    //keyboard::KeyCode,
    impl_component_provider,
    resource::texture::Texture,

    core::{
        reflect::prelude::*,
        pool::Handle,
        visitor::prelude::*, TypeUuidProvider,
        algebra::{Vector2, Vector3},
        uuid::{uuid, Uuid},
        //color::Color,
    },

    gui::{
        UiNode, UserInterface, HorizontalAlignment, VerticalAlignment,
        brush::Brush,
        button::{ButtonBuilder, ButtonMessage},
        core::{color::Color, algebra::UnitQuaternion},
        widget::WidgetBuilder,
        border::BorderBuilder, 
        message::{UiMessage, MessageDirection}, 
        text::{TextBuilder, TextMessage, Text},
    },

    scene::{
        Scene,
        node::Node,
        graph::Graph,
        base::BaseBuilder,
        transform::TransformBuilder,
        rigidbody::RigidBodyType,
        dim2::{
            rectangle::RectangleBuilder, 
            rigidbody::{RigidBody, RigidBodyBuilder}, 
            collider::{ColliderShape, ColliderBuilder},
        },
    },
};

use gilrs as g;
use gilrs::{
    EventType::*, 
    Gilrs, Event as gEvent, GamepadId,
    Button::{RightTrigger, LeftTrigger, RightThumb},
};

pub mod class;
pub mod messages;
pub mod create;
pub mod player;
pub mod projectile;
pub mod game;
pub mod weapon;

use messages::{
    Message,
    Message::{Controller, Hit, Parried},
};
use class::Class;

use create::*;
    // create_text_with_background,
    // set_script,
    // create_player,

use player::*;

use projectile::*;

use game::*;

use weapon::*;









pub struct GameConstructor;

impl PluginConstructor for GameConstructor {
    fn register(&self, context: PluginRegistrationContext) {
        // Register your scripts here.
        context.serialization_context.script_constructors.add::<Player>("Player");
        context.serialization_context.script_constructors.add::<Projectile>("Projectile");
        context.serialization_context.script_constructors.add::<Weapon>("Weapon");
    }

    fn create_instance(&self, scene_path: Option<&str>, context: PluginContext) -> Box<dyn Plugin> {
        Box::new(Game::new(scene_path, context))
    }
}
