//! Game project.
use std::collections::HashMap;
use std::vec::Vec;
use fyrox::{

    core::{
        pool::Handle,
        algebra::{Vector2, Vector3},
        reflect::prelude::*,
        uuid::{uuid, Uuid},
        visitor::prelude::*, TypeUuidProvider,
        futures::executor::block_on,
        color::Color,
    },
    gui::brush::Brush,
    gui::canvas::CanvasBuilder,
    gui::button::ButtonBuilder,
    gui::message::{UiMessage, MessageDirection},
    gui::{BuildContext, Orientation, HorizontalAlignment, VerticalAlignment},
    gui::widget::WidgetBuilder,
    gui::UserInterface,
    gui::text::TextBuilder,
    gui::border::BorderBuilder,
    gui::wrap_panel::WrapPanelBuilder,
    gui::progress_bar::{ProgressBarBuilder, ProgressBarMessage},
    gui::UiNode,
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
            joint::{JointBuilder, JointParams, BallJoint},
        },
        node::{Node},
        Scene, SceneLoader, SceneContainer,
        graph::{Graph},
        base::BaseBuilder,
        transform::TransformBuilder,
        rigidbody::RigidBodyType,
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
    ev::*,
    Button::{RightTrigger,},
};
use fyrox::script::Script;

pub mod class;
pub mod messages;

use messages::{
    Message,
    Message::{Controller},
};
use class::Class;


fn create_text(ui: &mut UserInterface, text: &str) -> Handle<UiNode> {
    TextBuilder::new(WidgetBuilder::new())
        .with_text(text)
        .build(&mut ui.build_ctx())
}

fn create_centered_text(ui: &mut UserInterface, text: &str) -> Handle<UiNode> {
    TextBuilder::new(WidgetBuilder::new())
        .with_horizontal_text_alignment(HorizontalAlignment::Center)
        .with_vertical_text_alignment(VerticalAlignment::Center)
    .with_text(text)
    .build(&mut ui.build_ctx())
}

// fn text_color_background(ui: &mut UserInterface, text: &str, color: const) -> Handle<UiNode> {
//     let text_widget =
//         TextBuilder::new(WidgetBuilder::new().with_foreground(Brush::Solid(Color::color)))
//             .with_text(text)
//             .build(&mut ui.build_ctx());
//     BorderBuilder::new(
//         WidgetBuilder::new().with_desired_position(Vector2::new(100.0, 200.0))
//             .with_child(text_widget) // <-- Text is now a child of the border
//             .with_background(Brush::Solid(Color::opaque(50, 50, 50))),
//     )
//     .build(&mut ui.build_ctx())
// }


// the functions fyrox gives us to create text were not great so i made my own
// each of the next 4 functions create text with a background (like highlighted)
// there's 4, one for each possible player, each with a different color
// also they take in floating point numbers as parameters for position
fn create_text_with_background_1(ui: &mut UserInterface, text: &str, x: f32, y: f32) -> Handle<UiNode> {
    let text_widget =
        TextBuilder::new(WidgetBuilder::new().with_foreground(Brush::Solid(Color::BLACK)))
            .with_text(text)
            .build(&mut ui.build_ctx());
    BorderBuilder::new(
        WidgetBuilder::new().with_desired_position(Vector2::new(x,y))
            .with_child(text_widget) // <-- Text is now a child of the border
            .with_background(Brush::Solid(Color::opaque(66, 245, 158))), // green
    )
    .build(&mut ui.build_ctx())
}

fn create_text_with_background_2(ui: &mut UserInterface, text: &str, x: f32, y: f32) -> Handle<UiNode> {
    let text_widget =
        TextBuilder::new(WidgetBuilder::new().with_foreground(Brush::Solid(Color::BLACK)))
            .with_text(text)
            .build(&mut ui.build_ctx());
    BorderBuilder::new(
        WidgetBuilder::new().with_desired_position(Vector2::new(x, y))
            .with_child(text_widget) // <-- Text is now a child of the border
            .with_background(Brush::Solid(Color::opaque(66, 167, 245))), // blue
    )
    .build(&mut ui.build_ctx())
}

fn create_text_with_background_3(ui: &mut UserInterface, text: &str, x: f32, y: f32) -> Handle<UiNode> {
    let text_widget =
        TextBuilder::new(WidgetBuilder::new().with_foreground(Brush::Solid(Color::BLACK)))
            .with_text(text)
            .build(&mut ui.build_ctx());
    BorderBuilder::new(
        WidgetBuilder::new().with_desired_position(Vector2::new(x, y))
            .with_child(text_widget) // <-- Text is now a child of the border
            .with_background(Brush::Solid(Color::opaque(194, 136, 252))), // purple
    )
    .build(&mut ui.build_ctx())
}

fn create_text_with_background_4(ui: &mut UserInterface, text: &str, x: f32, y: f32) -> Handle<UiNode> {
    let text_widget =
        TextBuilder::new(WidgetBuilder::new().with_foreground(Brush::Solid(Color::BLACK)))
            .with_text(text)
            .build(&mut ui.build_ctx());
    BorderBuilder::new(
        WidgetBuilder::new().with_desired_position(Vector2::new(x, y))
            .with_child(text_widget) // <-- Text is now a child of the border
            .with_background(Brush::Solid(Color::opaque(250, 135, 215))), // pink
    )
    .build(&mut ui.build_ctx())
}


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

fn create_kinematic_rigid_body(graph: &mut Graph) -> Handle<Node> {
    RigidBodyBuilder::new(BaseBuilder::new().with_children(&[
            // Rigid body must have at least one collider
            ColliderBuilder::new(BaseBuilder::new())
                .with_shape(ColliderShape::cuboid(0.5, 0.5))
                .with_sensor(true)
                .build(graph),
        ]))
    .with_body_type(RigidBodyType::KinematicVelocityBased)
    .build(graph)
}

fn create_joint(graph: &mut Graph, body1: Handle<Node>, body2: Handle<Node>) -> Handle<Node> {
    JointBuilder::new(BaseBuilder::new())
        .with_body1(body1)
        .with_body2(body2)
        .with_params(JointParams::BallJoint(BallJoint {
            limits_enabled: true,
            limits_angles: (0.0..1.0),
        }))
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
    idList: Vec::<GamepadId>,
}
use gilrs::GamepadId;
impl Game {
    pub fn new(scene_path: Option<&str>, context: PluginContext) -> Self {
        context
            .async_scene_loader
            .request(scene_path.unwrap_or("data/scene.rgs"));

        Self {
            scene: Handle::NONE,
            gils: Gilrs::new().unwrap(),
            players: HashMap::new(),
            idList: Vec::<GamepadId>::new(),
        }

    }
}



impl Plugin for Game {

    fn on_deinit(&mut self, _context: PluginContext) {
        // Do a cleanup here.
    }

    fn update(&mut self, context: &mut PluginContext) {

        let mut messager: Option<&ScriptMessageSender> = None;

        //get the scene messager... because that can't be done in on_scene_loaded apparently.
        for scripted_scene in &context.script_processor.scripted_scenes {
            if scripted_scene.handle == self.scene {
                messager = Some(&scripted_scene.message_sender);
            }
        }

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
                    set_script(&mut context.scenes[self.scene].graph[player_handle.clone()], 
                        Player{
                            class: Class::Rogue,
                            state: PlayerState::Idle,
                            weapon: None,
                            health: 10,
                        })

                },
                //send the controller event to the player
                _ => if let Some(player_handle) = self.players.get(&id) {
                    if let Some(message_sender) = &messager {
                        message_sender.send_to_target(player_handle.clone(), Message::Controller{event});
                    } else {println!("didn't get messager");}
                } else {println!("didn't get player handle");}
                
                _ => (), //for now

            }  
        }

        let ctx = &mut context.user_interface;

        // changes the number of xs in the health status bar
        // this has not been tested yet so idk if it works

        if self.players.len() > 0 {
            let h = self.players.get(&self.idList[0]).health;
            let mut text: &str = "";
            let mut i = 0;
            while i < h {
                text = &(text.to_owned() + "x");
                i = i+1;
            }
            create_text_with_background_1(ctx, text, 175.0, 1000.0);

        }

        if self.players.len() > 1 {
            let h = self.players.get(&self.idList[1]).health;
            let mut text: &str = "";
            let mut i = 0;
            while i < h {
                text = &(text.to_owned() + "x");
                i = i+1;
            }
            create_text_with_background_2(ctx, text, 375.0, 1000.0);

        }

        if self.players.len() > 2 {
            let h = self.players.get(&self.idList[2]).health;
            let mut text: &str = "";
            let mut i = 0;
            while i < h {
                text = &(text.to_owned() + "x");
                i = i+1;
            }
            create_text_with_background_3(ctx, text, 575.0, 1000.0);

        }

        if self.players.len() > 3 {
            let h = self.players.get(&self.idList[3]).health;
            let mut text: &str = "";
            let mut i = 0;
            while i < h {
                text = &(text.to_owned() + "x");
                i = i+1;
            }
            create_text_with_background_4(ctx, text, 775.0, 1000.0);

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

        let ctx = &mut context.user_interface;
        //let ui = &mut context.user_interface;
        let text: &str = "health: ";

        // this hasnt been tested yet since i don't have a controller

        // creates text health indicators for number of players
        // each has different color and position
        let mut playerText1: Handle<UiNode>;
        let mut playerText2: Handle<UiNode>;
        let mut playerText3: Handle<UiNode>;
        let mut playerText4: Handle<UiNode>;
        if self.players.len() > 0 {
            playerText1 = create_text_with_background_1(ctx, text, 100.0, 1000.0);
            create_text_with_background_1(ctx, "xxxxxxxxxx", 175.0, 1000.0);
        }

        if self.players.len() > 1 {
            playerText2 = create_text_with_background_2(ctx, text, 300.0, 1000.0);
            create_text_with_background_2(ctx, "xxxxxxxxxx", 375.0, 1000.0);
        }
       
       if self.players.len() > 2 {
            playerText3 = create_text_with_background_3(ctx, text, 500.0, 1000.0);
            create_text_with_background_3(ctx, "xxxxxxxxxx", 575.0, 1000.0);
        }
        
        if self.players.len() > 3 {
            playerText4 = create_text_with_background_4(ctx, text, 700.0, 1000.0);
            create_text_with_background_4(ctx, "xxxxxxxxxx", 775.0, 1000.0);
        }

        for key in self.players.keys() {
            self.idList.push(*key);
        }
    }

}

#[derive(Visit, Reflect, Debug, Clone, Default)]
pub enum PlayerState {
    #[default]
    Idle,
    //the field holds the number of frames the player is into the action
    Attacking(i32),
}

#[derive(Visit, Reflect, Debug, Clone, Default)]
pub struct Player{
    class: Class,
    state: PlayerState,
    weapon: Option<Handle<Node>>,
    health: u32,
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
        self.class.clone().startup(self, context);
    }

    // Called whenever there is an event from OS (mouse click, keypress, etc.)
    fn on_os_event(&mut self, event: &Event<()>, context: &mut ScriptContext) {}

    // Called every frame at fixed rate of 60 FPS.
    fn on_update(&mut self, context: &mut ScriptContext) {
        match self.state {
            PlayerState::Attacking(frame) => {self.class.clone().cont_attack(self, frame, context)},

            _ => (),
        }


    }

    fn on_message(&mut self,
        message: &mut dyn ScriptMessagePayload,
        ctx: &mut ScriptMessageContext,
    ) {
        if let Some(message) = message.downcast_ref::<Message>(){
            match message {
                Controller{event} => {
                    match event {
                        // put the various controller events here, as well as calls to
                        //the correct class methods-- player has a class field now!
                        AxisChanged(axis, value, _code) => self.class.moveplayer(axis, value, ctx),
                        //must clone class for any method that takes a 'self' as well.
                        ButtonPressed(RightTrigger, _) => self.class.clone().start_melee_attack(self, ctx),
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