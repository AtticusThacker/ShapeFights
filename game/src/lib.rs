//! Game project.
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
};
use fyrox::script::Script;

pub mod class;
pub mod messages;
use messages::{
    Message,
    Message::{Controller},
};
use class::Class;

fn create_cube_rigid_body(graph: &mut Graph) -> Handle<Node> {
    RigidBodyBuilder::new(BaseBuilder::new().with_children(&[
            // Rigid body must have at least one collider
            ColliderBuilder::new(BaseBuilder::new())
                .with_shape(ColliderShape::cuboid(0.5, 0.5))
                .build(graph),
        ]))
    .with_mass(2.0)
    .with_gravity_scale(0.0)
    .with_can_sleep(false)
    .with_rotation_locked(true)
    .with_lin_vel(Vector2::new(0.0, 0.0))
    .build(graph)
}

fn create_rect(graph: &mut Graph, resource_manager: &ResourceManager) -> Handle<Node> {
    RectangleBuilder::new(
        BaseBuilder::new().with_local_transform(
            TransformBuilder::new()
                // Size of the rectangle is defined only by scale.
                .with_local_scale(Vector3::new(0.4, 0.2, 1.0))
                .build(),
        ),
    )
    .with_texture(resource_manager.request::<Texture, _>("data/rcircle.png"))
    .build(graph)
}

fn set_script<T: ScriptTrait>(node: &mut Node, script: T) {
    node.set_script(Some(Script::new(script)))
}

pub struct GameConstructor;

impl PluginConstructor for GameConstructor {
    fn register(&self, context: PluginRegistrationContext) {
        // Register your scripts here.
        context.serialization_context.script_constructors.add::<Player>("Player");
    }

    fn create_instance(&self, scene_path: Option<&str>, context: PluginContext) -> Box<dyn Plugin> {
        Box::new(Game::new(scene_path, context))
    }
}

pub struct Game {
    scene: Handle<Scene>,
    gils: Gilrs,
    players: HashMap<g::GamepadId, Handle<Node>>,
    messager: Option<ScriptMessageSender>,
}

impl Game {
    pub fn new(scene_path: Option<&str>, context: PluginContext) -> Self {
        context
            .async_scene_loader
            .request(scene_path.unwrap_or("data/scene.rgs"));

        Self {
            scene: Handle::NONE,
            gils: Gilrs::new().unwrap(),
            players: HashMap::new(),
            messager: None,
        }
    }
}

impl Plugin for Game {
    fn on_deinit(&mut self, _context: PluginContext) {
        // Do a cleanup here.
    }

    fn update(&mut self, context: &mut PluginContext) {

        //read in all new gilrs events
        while let Some(gEvent { id, event, time }) = self.gils.next_event() {
            
            //matching on the event type 
            match event {
                Connected => {
                    //NOTE on these; context has a field scenes which is a scenecontainer
                    // and scenecontainer can be indexed by Handle<Scene>, which is what self.scene is.
                    // then, graph can be indexed by a Handle<Node>>, to get a dynamic object that we
                    // have to 'downcast' using the .cast_mut thingy to get the actual player object.
                    // its complicated i know, but it works!

                    //create a new player
                    let player_handle = create_cube_rigid_body(&mut context.scenes[self.scene].graph);
                    //create a sprite for the player
                    let sprite_handle = create_rect(&mut context.scenes[self.scene].graph, context.resource_manager);
                    //make the sprite a child of the player
                    context.scenes[self.scene].graph.link_nodes(sprite_handle, player_handle);
                    //add the player to the game's struct
                    self.players.insert(id, player_handle);

                    //adds script player to object
                    set_script(&mut context.scenes[self.scene].graph[player_handle.clone()], Player{class: Class::Fighter})

                },
                //send the controller event to the player
                _ => if let Some(player_handle) = self.players.get(&id) {
                    if let Some(message_sender) = &self.messager {
                        message_sender.send_to_target(player_handle.clone(), Message::Controller{event});
                    }
                }
                // AxisChanged(axis, value, code) => {
                //     if let Some(handle) = self.players.get(&id){
                //         match axis {
                //             g::Axis::LeftStickX => {// change the x velocity of the right player
                //                 if let Some(player) = context.scenes[self.scene].graph[handle.clone()].cast_mut::<RigidBody>() {
                //                     player.set_lin_vel(Vector2::new(-value, player.lin_vel().y));
                //                 }
                //             },
                //             g::Axis::LeftStickY => {// change the x velocity of the right player
                //                 if let Some(player) = context.scenes[self.scene].graph[handle.clone()].cast_mut::<RigidBody>() {
                //                     player.set_lin_vel(Vector2::new(player.lin_vel().x, value));
                //                 }
                //             },
                //             _ => (), //for now
                //         }
                //     }
                // },
                _ => (), //for now

            }


            
        }
    }

    fn on_os_event(
        &mut self,
        _event: &Event<()>,
        _context: PluginContext,
    ) {
        // Do something on OS event here.
    }

    fn on_ui_message(
        &mut self,
        _context: &mut PluginContext,
        _message: &UiMessage,
    ) {
        // Handle UI events here.
    }
    
    fn on_scene_begin_loading(&mut self, path: &Path, ctx: &mut PluginContext) {
        if self.scene.is_some() {
            ctx.scenes.remove(self.scene);
        }
    }

    fn on_scene_loaded(
        &mut self,
        path: &Path,
        scene: Handle<Scene>,
        data: &[u8],
        context: &mut PluginContext,
    ) {    
        self.scene = scene;

        //gets the message sender for the current scene. why is this such a pain??
        for scripted_scene in &context.script_processor.scripted_scenes {
            if scripted_scene.handle == self.scene {
                self.messager = Option::Some(scripted_scene.message_sender.clone());
            }
        }

    }
}

#[derive(Visit, Reflect, Debug, Clone, Default)]
struct Player{
   class: Class 

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
    fn on_init(&mut self, context: &mut ScriptContext) {}
    
    // Put start logic - it is called when every other script is already initialized.
    fn on_start(&mut self, context: &mut ScriptContext) { 
        context.message_dispatcher.subscribe_to::<Message>(context.handle);
    }

    // Called whenever there is an event from OS (mouse click, keypress, etc.)
    fn on_os_event(&mut self, event: &Event<()>, context: &mut ScriptContext) {}

    // Called every frame at fixed rate of 60 FPS.
    fn on_update(&mut self, context: &mut ScriptContext) {}

    fn on_message(&mut self,
        message: &mut dyn ScriptMessagePayload,
        _ctx: &mut ScriptMessageContext,
    ) {
        if let Some(message) = message.downcast_ref::<Message>(){
            match message {
                Controller{event} => {
                    match event {
                        // put the various controller events here, as well as calls to
                        //the correct class methods-- player has a class field now!
                        AxisChanged(axis, value, _code) => (),
                        _ => (),
                    }

                },
                _ => (),
            }

        }
    }

    // Returns unique script ID for serialization needs.
    fn id(&self) -> Uuid {
        Self::type_uuid()
    }
}