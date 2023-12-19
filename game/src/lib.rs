//! Game project
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
        //color::Color,
    },
    gui::brush::Brush,
    gui::canvas::CanvasBuilder,
    gui::button::ButtonBuilder,
    gui::{message::{UiMessage}, core::color::Color},
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
    //keyboard::KeyCode,
    impl_component_provider,
    resource::texture::{Texture, TextureResource},
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
    Button::{RightTrigger,LeftTrigger},
};
use fyrox::script::Script;

pub mod class;
pub mod messages;

use messages::{
    Message,
    Message::{Controller, Hit},
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



// the functions fyrox gives us to create text were not great so i made my own
// each of the next 4 functions create text with a background (like highlighted)
// there's 4, one for each possible player, each with a different color
// also they take in floating point numbers as parameters for position
// default visibility is set to false!!

// player 1
fn create_text_with_background_1(ui: &mut UserInterface, text: &str, x: f32, y: f32) -> Handle<UiNode> {
    let text_widget =
        TextBuilder::new(WidgetBuilder::new().with_foreground(Brush::Solid(Color::BLACK)))
            .with_text(text)
            .build(&mut ui.build_ctx());
    BorderBuilder::new(
        WidgetBuilder::new().with_desired_position(Vector2::new(x,y))
            .with_child(text_widget) // <-- Text is now a child of the border
            .with_background(Brush::Solid(Color::opaque(66, 245, 158))) // green
            .with_visibility(false),
    )
    .build(&mut ui.build_ctx())
}

// player 2
fn create_text_with_background_2(ui: &mut UserInterface, text: &str, x: f32, y: f32) -> Handle<UiNode> {
    let text_widget =
        TextBuilder::new(WidgetBuilder::new().with_foreground(Brush::Solid(Color::BLACK)))
            .with_text(text)
            .build(&mut ui.build_ctx());
    BorderBuilder::new(
        WidgetBuilder::new().with_desired_position(Vector2::new(x, y))
            .with_child(text_widget) // <-- Text is now a child of the border
            .with_background(Brush::Solid(Color::opaque(66, 167, 245))) // blue
            .with_visibility(false),
    )
    .build(&mut ui.build_ctx())
}

// player 3
fn create_text_with_background_3(ui: &mut UserInterface, text: &str, x: f32, y: f32) -> Handle<UiNode> {
    let text_widget =
        TextBuilder::new(WidgetBuilder::new().with_foreground(Brush::Solid(Color::BLACK)))
            .with_text(text)
            .build(&mut ui.build_ctx());
    BorderBuilder::new(
        WidgetBuilder::new().with_desired_position(Vector2::new(x, y))
            .with_child(text_widget) // <-- Text is now a child of the border
            .with_background(Brush::Solid(Color::opaque(194, 136, 252))) // purple
            .with_visibility(false),
    )
    .build(&mut ui.build_ctx())
}

// player 4
fn create_text_with_background_4(ui: &mut UserInterface, text: &str, x: f32, y: f32) -> Handle<UiNode> {
    let text_widget =
        TextBuilder::new(WidgetBuilder::new().with_foreground(Brush::Solid(Color::BLACK)))
            .with_text(text)
            .build(&mut ui.build_ctx());
    BorderBuilder::new(
        WidgetBuilder::new().with_desired_position(Vector2::new(x, y))
            .with_child(text_widget) // <-- Text is now a child of the border
            .with_background(Brush::Solid(Color::opaque(250, 135, 215))) // pink
            .with_visibility(false),
    )
    .build(&mut ui.build_ctx())
}


fn create_cube_rigid_body(graph: &mut Graph) -> Handle<Node> {
    RigidBodyBuilder::new(BaseBuilder::new().with_children(&[
            // Rigid body must have at least one collider
            ColliderBuilder::new(BaseBuilder::new())
                .with_shape(ColliderShape::cuboid(0.25, 0.2))
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
                .with_local_scale(Vector3::new(0.4, 0.4, 0.4))
                .build(),
        ),
    )
    .with_texture(resource_manager.request::<Texture, _>("data/White_star.png"))
    .with_color(Color{r:0, g:255, b:127, a:255})
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
    id_list: Vec::<GamepadId>,
    player_text1: Handle<UiNode>,
    player_text2: Handle<UiNode>,
    player_text3: Handle<UiNode>,
    player_text4: Handle<UiNode>,
    text1: Handle<UiNode>,
    text2: Handle<UiNode>,
    text3: Handle<UiNode>,
    text4: Handle<UiNode>,
    //ctx: UserInterface,
    //HEALTH_TXT: String,
}
use gilrs::GamepadId;
impl Game {
    pub fn new(scene_path: Option<&str>, context: PluginContext) -> Self {
        context
            .async_scene_loader
            .request(scene_path.unwrap_or("data/scene.rgs"));

        Self {
            //ctx: context.user_interface,
            scene: Handle::NONE,
            gils: Gilrs::new().unwrap(),
            players: HashMap::new(),
            id_list: Vec::<GamepadId>::new(),
            player_text1: create_text_with_background_1(context.user_interface, "health:", 100.0, 1000.0),
            player_text2: create_text_with_background_2(context.user_interface, "health:", 300.0, 1000.0),
            player_text3: create_text_with_background_3(context.user_interface, "health:", 500.0, 1000.0),
            player_text4: create_text_with_background_4(context.user_interface, "health:", 700.0, 1000.0),
            text1: create_text_with_background_1(context.user_interface, "", 175.0, 1000.0),
            text2: create_text_with_background_2(context.user_interface, "", 375.0, 1000.0),
            text3: create_text_with_background_3(context.user_interface, "", 575.0, 1000.0),
            text4: create_text_with_background_4(context.user_interface, "", 775.0, 1000.0),
            //HEALTH_TXT: "health:".to_string(),
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
                    // add player ID to vector of IDs
                    self.id_list.push(id);

                    //adds script player to object
                    set_script(&mut context.scenes[self.scene].graph[player_handle.clone()], 
                        Player{
                            class: Class::Rogue,
                            state: PlayerState::Idle,
                            weapon: None,
                                cooldown: 0,
                                facing: Vector3::new(0.0,1.0,0.0),
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

        // changes the number of xs in the health status bar
        // this has not been tested yet so idk if it works
        let ctx = &mut context.user_interface;
        // let health_txt = "health:";
        let mut text = "".to_string();
        

        // create text/border widgets with visibility off
        // let player_text1: Handle<UiNode> = create_text_with_background_1(ctx, health_txt, 100.0, 1000.0);
        // let player_text2: Handle<UiNode> = create_text_with_background_2(ctx, health_txt, 300.0, 1000.0);
        // let player_text3: Handle<UiNode> = create_text_with_background_3(ctx, health_txt, 500.0, 1000.0);
        // let player_text4: Handle<UiNode> = create_text_with_background_4(ctx, health_txt, 700.0, 1000.0);

        // let mut text1 = create_text_with_background_1(ctx, text.as_str(), 175.0, 1000.0);
        // let mut text2 = create_text_with_background_2(ctx, text.as_str(), 375.0, 1000.0);
        // let mut text3 = create_text_with_background_3(ctx, text.as_str(), 575.0, 1000.0);
        // let mut text4 = create_text_with_background_4(ctx, text.as_str(), 775.0, 1000.0);

        // for player 1
        if self.players.len() > 0 {
            // creates health variable here
            let mut h: u32 = 10;

            ctx.build_ctx()[self.player_text1.clone()].set_visibility(true);

            // gets the player handle from hash map for player 1
            if let Some(player_script) = self.players.get(&self.id_list[0]) {
                // gets the node
                let mut node1 = &mut context.scenes[self.scene].graph[player_script.clone()];
                // gets the actual player object
                let node2 = node1.script_mut().expect("error").cast_mut::<Player>().expect("error");
                // sets health variable to player's health
                h = node2.health;
            }

            // creates text variable to be passed into text function
            let mut text: String = "".to_string();
            let mut i = 0;
            // makes the text variable have the number of xs corresponding to health value
            while i < h {
                text = text.to_owned() + "x";
                i = i+1;
            }

            self.text1 = create_text_with_background_1(ctx, text.as_str(), 175.0, 1000.0);
            ctx.build_ctx()[self.text1.clone()].set_visibility(true);
        }

        // player 2
        if self.players.len() > 1 {
            // creates health variable here
            let mut h: u32 = 10;
            ctx.build_ctx()[self.player_text2.clone()].set_visibility(true);
            // gets the player handle from hash map for player 1
            if let Some(player_script) = self.players.get(&self.id_list[1]) {
                // gets the node
                let mut node1 = &mut context.scenes[self.scene].graph[player_script.clone()];
                // gets the actual player object
                let node2 = node1.script().unwrap().cast::<Player>().unwrap();
                // sets health variable to player's health
                h = node2.health;
            }

            // creates text variable to be passed into text function
            let mut text: String = "".to_string();
            let mut i = 0;
            // makes the text variable have the number of xs corresponding to health value
            while i < h {
                text = text.to_owned() + "x";
                i = i+1;
            }
            self.text2 = create_text_with_background_2(ctx, text.as_str(), 375.0, 1000.0);
            ctx.build_ctx()[self.text2.clone()].set_visibility(true);
        }

        // player 3
        if self.players.len() > 2 {
            // creates health variable here
            let mut h: u32 = 10;
            ctx.build_ctx()[self.player_text3.clone()].set_visibility(true);
            // let playerText3: Handle<UiNode> = create_text_with_background_2(ctx, health_txt, 500.0, 1000.0);
            // gets the player handle from hash map for player 1
            if let Some(player_script) = self.players.get(&self.id_list[2]) {
                // gets the node
                let mut node1 = &mut context.scenes[self.scene].graph[player_script.clone()];
                // gets the actual player object
                let node2 = node1.script().unwrap().cast::<Player>().unwrap();
                // sets health variable to player's health
                h = node2.health;
            }

            // creates text variable to be passed into text function
            let mut text: String = "".to_string();
            let mut i = 0;
            // makes the text variable have the number of xs corresponding to health value
            while i < h {
                text = text.to_owned() + "x";
                i = i+1;
            }

            self.text3 = create_text_with_background_3(ctx, text.as_str(), 575.0, 1000.0);
            ctx.build_ctx()[self.text3.clone()].set_visibility(true);
        }

        // player 4
        if self.players.len() > 3 {
            // creates health variable here
            let mut h = 10;
            ctx.build_ctx()[self.player_text4.clone()].set_visibility(true);
            // gets the player handle from hash map for player 1
            if let Some(player_script) = self.players.get(&self.id_list[3]) {
                // gets the node
                let mut node1 = &mut context.scenes[self.scene].graph[player_script.clone()];
                // gets the actual player object
                let node2 = node1.script().unwrap().cast::<Player>().unwrap();
                // sets health variable to player's health
                h = node2.health;
            }

            // creates text variable to be passed into text function
            let mut text: String = "".to_string();
            let mut i = 0;
            // makes the text variable have the number of xs corresponding to health value
            while i < h {
                text = text.to_owned() + "x";
                i = i+1;
            }

            self.text4 = create_text_with_background_4(ctx, text.as_str(), 775.0, 1000.0);
            ctx.build_ctx()[self.text4.clone()].set_visibility(true);
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
     }
}

#[derive(Visit, Reflect, Debug, Clone, Default, PartialEq)]
pub enum PlayerState {
    #[default]
    Idle,
    //the field holds the number of frames the player is into the action
    Attacking(i32),
    Hit(i32),
}

#[derive(Visit, Reflect, Debug, Clone, Default)]
pub struct Player{
    class: Class,
    state: PlayerState,
    weapon: Option<Handle<Node>>,
    cooldown: i32,
    facing: Vector3<f32>, //z axis should always be 0.0 here!
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

        self.cooldown += 1;
        Class::update_look(self.facing.clone(), &mut context.scene.graph[context.handle.clone()]);

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
                        AxisChanged(axis, value, _code) => self.class.clone().moveplayer(self, axis, value, ctx),
                        //must clone class for any method that takes a 'self' as well.
                        ButtonPressed(RightTrigger, _) => self.class.clone().start_melee_attack(self, ctx),
                        //projectiles
                        ButtonPressed(LeftTrigger, _) => self.class.clone().projectiles(self, ctx),
                        _ => (),
                    }
                },

                Hit{damage: dam, knockback: knock, body: bod} => {
                    self.class.clone().takehit(self, dam.clone(), knock.clone(), bod.clone(), ctx);
                }
                _ => (),
            }
        }
    }

    // Returns unique script ID for serialization needs.
    fn id(&self) -> Uuid {
        Self::type_uuid()
    }
}

#[derive(Visit, Reflect, Debug, Clone, Default)]

pub struct Projectile {

}

impl_component_provider!(Projectile,);

impl TypeUuidProvider for Projectile {
    // Returns unique script id for serialization needs.
    fn type_uuid() -> Uuid {
        uuid!("c5671d19-9f1a-4286-8486-add4ebaadaed")
    }
}

impl ScriptTrait for Projectile {
    // Called once at initialization.
    fn on_init(&mut self, context: &mut ScriptContext) {}
    
    // Put start logic - it is called when every other script is already initialized.
    fn on_start(&mut self, context: &mut ScriptContext) { }

    // Called whenever there is an event from OS (mouse click, keypress, etc.)
    fn on_os_event(&mut self, event: &Event<()>, context: &mut ScriptContext) {}

    // Called every frame at fixed rate of 60 FPS.
    fn on_update(&mut self, context: &mut ScriptContext) {}

    // Returns unique script ID for serialization needs.
    fn id(&self) -> Uuid {
        Self::type_uuid()
    }
}