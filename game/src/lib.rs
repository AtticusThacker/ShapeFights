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
    gui::{
        UiNode,
        UserInterface,
        message::UiMessage,
        button::{
            ButtonBuilder,
            ButtonMessage,
        },
        window::{WindowBuilder, WindowTitle}, 
        widget::WidgetBuilder,
        text::TextBuilder,
        stack_panel::StackPanelBuilder,
        HorizontalAlignment,
    },
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
        base::{BaseBuilder},
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
    Message::{Controller, Hit},
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
    playerclasses: HashMap<g::GamepadId, class::Class>,
    idList: Vec::<g::GamepadId>,
    start_button_handle: Handle<UiNode>,
    class_button_handle: Handle<UiNode>,

    p1fig: Handle<UiNode>,
    p1barb: Handle<UiNode>,
    p1rog: Handle<UiNode>,
    p1wiz: Handle<UiNode>,

    p2fig: Handle<UiNode>,
    p2barb: Handle<UiNode>,
    p2rog: Handle<UiNode>,
    p2wiz: Handle<UiNode>,

    p3fig: Handle<UiNode>,
    p3barb: Handle<UiNode>,
    p3rog: Handle<UiNode>,
    p3wiz: Handle<UiNode>,

    p4fig: Handle<UiNode>,
    p4barb: Handle<UiNode>,
    p4rog: Handle<UiNode>,
    p4wiz: Handle<UiNode>,
    
    // messager: Option<ScriptMessageSender>,
}

fn start_button(ui: &mut UserInterface) -> Handle<UiNode> {
    ButtonBuilder::new(
        WidgetBuilder::new()
        .with_width(200.0)
        .with_height(60.0),
        //.with_horizontal_alignment(HorizontalAlignment::Left),
    )
    .with_content(
        TextBuilder::new(WidgetBuilder::new())
            .with_text("Start Game  ")
            .with_horizontal_text_alignment(HorizontalAlignment::Right)
            .build(&mut ui.build_ctx()),
    )
    .build(&mut ui.build_ctx())
}

fn class_button(ui: &mut UserInterface) -> Handle<UiNode> {
    ButtonBuilder::new(
        WidgetBuilder::new()
        .with_width(100.0)
        .with_height(60.0),
        //.with_horizontal_alignment(HorizontalAlignment::Right),
    
    )
    .with_content(
        TextBuilder::new(WidgetBuilder::new())
            .with_text("Class Select")
            
            .build(&mut ui.build_ctx()),
    )
    .build(&mut ui.build_ctx())
}

fn create_window(ui: &mut UserInterface) {
    WindowBuilder::new(
        WidgetBuilder::new()
            .with_desired_position(Vector2::new(300.0, 0.0))
            .with_width(300.0),
    )
    .with_content(
        StackPanelBuilder::new(
            WidgetBuilder::new()
                .with_child(
                    WindowBuilder::new(
                        WidgetBuilder::new()
                            .with_desired_position(Vector2::new(300.0, 0.0))
                            .with_width(300.0),
                    )
                    .with_content(
                        StackPanelBuilder::new(
                            WidgetBuilder::new()
                                .with_child(
                                    p1fig(ui)
                                )
                                .with_child(
                                    p1rog(ui)
                                )
                                .with_child(
                                    p1barb(ui)
                                )
                                .with_child(
                                    p1wiz(ui)
                                )
                        )
                            .build(&mut ui.build_ctx())
                    )
                    .with_title(WindowTitle::text("Player 1"))
                    .can_close(false)
                    .can_minimize(false)
                    .open(true)
                    .can_resize(false)
                    .build(&mut ui.build_ctx())
                )
                .with_child(
                    WindowBuilder::new(
                        WidgetBuilder::new()
                            .with_desired_position(Vector2::new(300.0, 0.0))
                            .with_width(300.0),
                    )
                    .with_content(
                        StackPanelBuilder::new(
                            WidgetBuilder::new()
                                .with_child(
                                    p2fig(ui)
                                )
                                .with_child(
                                    p2rog(ui)
                                )
                                .with_child(
                                    p2barb(ui)
                                )
                                .with_child(
                                    p2wiz(ui)
                                )
                        )
                            .build(&mut ui.build_ctx())
                    )
                    .with_title(WindowTitle::text("Player 2"))
                    .can_close(false)
                    .can_minimize(false)
                    .open(true)
                    .can_resize(false)
                    .build(&mut ui.build_ctx())
                )
                .with_child(
                    WindowBuilder::new(
                        WidgetBuilder::new()
                            .with_desired_position(Vector2::new(300.0, 0.0))
                            .with_width(300.0),
                    )
                    .with_content(
                        StackPanelBuilder::new(
                            WidgetBuilder::new()
                                .with_child(
                                    p3fig(ui)
                                )
                                .with_child(
                                    p3rog(ui)
                                )
                                .with_child(
                                    p3barb(ui)
                                )
                                .with_child(
                                    p3wiz(ui)
                                )
                        )
                            .build(&mut ui.build_ctx())
                    )
                    .with_title(WindowTitle::text("Player 3"))
                    .can_close(false)
                    .can_minimize(false)
                    .open(true)
                    .can_resize(false)
                    .build(&mut ui.build_ctx())
                )
                .with_child(
                    WindowBuilder::new(
                        WidgetBuilder::new()
                            .with_desired_position(Vector2::new(300.0, 0.0))
                            .with_width(300.0),
                    )
                    .with_content(
                        StackPanelBuilder::new(
                            WidgetBuilder::new()
                                .with_child(
                                    p4fig(ui)
                                )
                                .with_child(
                                    p4rog(ui)
                                )
                                .with_child(
                                    p4barb(ui)
                                )
                                .with_child(
                                    p4wiz(ui)
                                )
                        )
                            .build(&mut ui.build_ctx())
                    )
                    .with_title(WindowTitle::text("Player 4"))
                    .can_close(false)
                    .can_minimize(false)
                    .open(true)
                    .can_resize(false)
                    .build(&mut ui.build_ctx())
                )
        )
            .build(&mut ui.build_ctx())
    )
    .with_title(WindowTitle::text("Select Your Classes"))
    .can_close(true)
    .can_minimize(true)
    .open(true)
    .can_resize(false)
    .build(&mut ui.build_ctx());
}

fn p1fig(ui: &mut UserInterface) -> Handle<UiNode> {
    ButtonBuilder::new(
        WidgetBuilder::new()
        .with_width(100.0)
        .with_height(20.0),
        //.with_horizontal_alignment(HorizontalAlignment::Right),
    
    )
    .with_content(
        TextBuilder::new(WidgetBuilder::new())
            .with_text("Fighter")
            
            .build(&mut ui.build_ctx()),
    )
    .build(&mut ui.build_ctx())
}

fn p2fig(ui: &mut UserInterface) -> Handle<UiNode> {
    ButtonBuilder::new(
        WidgetBuilder::new()
        .with_width(100.0)
        .with_height(20.0),
        //.with_horizontal_alignment(HorizontalAlignment::Right),
    
    )
    .with_content(
        TextBuilder::new(WidgetBuilder::new())
            .with_text("Fighter")
            
            .build(&mut ui.build_ctx()),
    )
    .build(&mut ui.build_ctx())
}

fn p3fig(ui: &mut UserInterface) -> Handle<UiNode> {
    ButtonBuilder::new(
        WidgetBuilder::new()
        .with_width(100.0)
        .with_height(20.0),
        //.with_horizontal_alignment(HorizontalAlignment::Right),
    
    )
    .with_content(
        TextBuilder::new(WidgetBuilder::new())
            .with_text("Fighter")
            
            .build(&mut ui.build_ctx()),
    )
    .build(&mut ui.build_ctx())
}

fn p4fig(ui: &mut UserInterface) -> Handle<UiNode> {
    ButtonBuilder::new(
        WidgetBuilder::new()
        .with_width(100.0)
        .with_height(20.0),
        //.with_horizontal_alignment(HorizontalAlignment::Right),
    
    )
    .with_content(
        TextBuilder::new(WidgetBuilder::new())
            .with_text("Fighter")
            
            .build(&mut ui.build_ctx()),
    )
    .build(&mut ui.build_ctx())
}

fn p1barb(ui: &mut UserInterface) -> Handle<UiNode> {
    ButtonBuilder::new(
        WidgetBuilder::new()
        .with_width(100.0)
        .with_height(20.0),
        //.with_horizontal_alignment(HorizontalAlignment::Right),
    
    )
    .with_content(
        TextBuilder::new(WidgetBuilder::new())
            .with_text("Barbarian")
            
            .build(&mut ui.build_ctx()),
    )
    .build(&mut ui.build_ctx())
}

fn p2barb(ui: &mut UserInterface) -> Handle<UiNode> {
    ButtonBuilder::new(
        WidgetBuilder::new()
        .with_width(100.0)
        .with_height(20.0),
        //.with_horizontal_alignment(HorizontalAlignment::Right),
    
    )
    .with_content(
        TextBuilder::new(WidgetBuilder::new())
            .with_text("Barbarian")
            
            .build(&mut ui.build_ctx()),
    )
    .build(&mut ui.build_ctx())
}

fn p3barb(ui: &mut UserInterface) -> Handle<UiNode> {
    ButtonBuilder::new(
        WidgetBuilder::new()
        .with_width(100.0)
        .with_height(20.0),
        //.with_horizontal_alignment(HorizontalAlignment::Right),
    
    )
    .with_content(
        TextBuilder::new(WidgetBuilder::new())
            .with_text("Barbarian")
            
            .build(&mut ui.build_ctx()),
    )
    .build(&mut ui.build_ctx())
}

fn p4barb(ui: &mut UserInterface) -> Handle<UiNode> {
    ButtonBuilder::new(
        WidgetBuilder::new()
        .with_width(100.0)
        .with_height(20.0),
        //.with_horizontal_alignment(HorizontalAlignment::Right),
    
    )
    .with_content(
        TextBuilder::new(WidgetBuilder::new())
            .with_text("Barbarian")
            
            .build(&mut ui.build_ctx()),
    )
    .build(&mut ui.build_ctx())
}

fn p1rog(ui: &mut UserInterface) -> Handle<UiNode> {
    ButtonBuilder::new(
        WidgetBuilder::new()
        .with_width(100.0)
        .with_height(20.0),
        //.with_horizontal_alignment(HorizontalAlignment::Right),
    
    )
    .with_content(
        TextBuilder::new(WidgetBuilder::new())
            .with_text("Rogue")
            
            .build(&mut ui.build_ctx()),
    )
    .build(&mut ui.build_ctx())
}

fn p2rog(ui: &mut UserInterface) -> Handle<UiNode> {
    ButtonBuilder::new(
        WidgetBuilder::new()
        .with_width(100.0)
        .with_height(20.0),
        //.with_horizontal_alignment(HorizontalAlignment::Right),
    
    )
    .with_content(
        TextBuilder::new(WidgetBuilder::new())
            .with_text("Rogue")
            
            .build(&mut ui.build_ctx()),
    )
    .build(&mut ui.build_ctx())
}

fn p3rog(ui: &mut UserInterface) -> Handle<UiNode> {
    ButtonBuilder::new(
        WidgetBuilder::new()
        .with_width(100.0)
        .with_height(20.0),
        //.with_horizontal_alignment(HorizontalAlignment::Right),
    
    )
    .with_content(
        TextBuilder::new(WidgetBuilder::new())
            .with_text("Rogue")
            
            .build(&mut ui.build_ctx()),
    )
    .build(&mut ui.build_ctx())
}

fn p4rog(ui: &mut UserInterface) -> Handle<UiNode> {
    ButtonBuilder::new(
        WidgetBuilder::new()
        .with_width(100.0)
        .with_height(20.0),
        //.with_horizontal_alignment(HorizontalAlignment::Right),
    
    )
    .with_content(
        TextBuilder::new(WidgetBuilder::new())
            .with_text("Rogue")
            
            .build(&mut ui.build_ctx()),
    )
    .build(&mut ui.build_ctx())
}

fn p1wiz(ui: &mut UserInterface) -> Handle<UiNode> {
    ButtonBuilder::new(
        WidgetBuilder::new()
        .with_width(100.0)
        .with_height(20.0),
        //.with_horizontal_alignment(HorizontalAlignment::Right),
    
    )
    .with_content(
        TextBuilder::new(WidgetBuilder::new())
            .with_text("Wizard")
            
            .build(&mut ui.build_ctx()),
    )
    .build(&mut ui.build_ctx())
}

fn p2wiz(ui: &mut UserInterface) -> Handle<UiNode> {
    ButtonBuilder::new(
        WidgetBuilder::new()
        .with_width(100.0)
        .with_height(20.0),
        //.with_horizontal_alignment(HorizontalAlignment::Right),
    
    )
    .with_content(
        TextBuilder::new(WidgetBuilder::new())
            .with_text("Wizard")
            
            .build(&mut ui.build_ctx()),
    )
    .build(&mut ui.build_ctx())
}

fn p3wiz(ui: &mut UserInterface) -> Handle<UiNode> {
    ButtonBuilder::new(
        WidgetBuilder::new()
        .with_width(100.0)
        .with_height(20.0),
        //.with_horizontal_alignment(HorizontalAlignment::Right),
    
    )
    .with_content(
        TextBuilder::new(WidgetBuilder::new())
            .with_text("Wizard")
            
            .build(&mut ui.build_ctx()),
    )
    .build(&mut ui.build_ctx())
}

fn p4wiz(ui: &mut UserInterface) -> Handle<UiNode> {
    ButtonBuilder::new(
        WidgetBuilder::new()
        .with_width(100.0)
        .with_height(20.0),
        //.with_horizontal_alignment(HorizontalAlignment::Right),
    
    )
    .with_content(
        TextBuilder::new(WidgetBuilder::new())
            .with_text("Wizard")
            
            .build(&mut ui.build_ctx()),
    )
    .build(&mut ui.build_ctx())
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
            playerclasses: HashMap::new(),
            idList: Vec::new(),

            p1fig: p1fig(context.user_interface),
            p1barb: p1barb(context.user_interface),
            p1rog: p1rog(context.user_interface),
            p1wiz: p1wiz(context.user_interface),

            p2fig: p2fig(context.user_interface),
            p2barb: p2barb(context.user_interface),
            p2rog: p2rog(context.user_interface),
            p2wiz: p2wiz(context.user_interface),

            p3fig: p3fig(context.user_interface),
            p3barb: p3barb(context.user_interface),
            p3rog: p3rog(context.user_interface),
            p3wiz: p3wiz(context.user_interface),

            p4fig: p4fig(context.user_interface),
            p4barb: p4barb(context.user_interface),
            p4rog: p4rog(context.user_interface),
            p4wiz: p4wiz(context.user_interface),

            start_button_handle: start_button(context.user_interface),
            class_button_handle: class_button(context.user_interface),
            
            
            //messager: None,
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

                    //add the player and default class to struct
                    self.playerclasses.insert(id, Class::Rogue);

                    self.idList.push(id);



                    //adds script player to object
                    set_script(&mut context.scenes[self.scene].graph[player_handle.clone()], 
                        Player{
                                class: self.playerclasses[&id].clone(),
                                state: PlayerState::Idle,
                                weapon: None,
                                facing: Vector3::new(0.0,1.0,0.0),
                                })

                },
                //send the controller event to the player
                _ => if let Some(player_handle) = self.players.get(&id) {
                    if let Some(message_sender) = &messager {
                        message_sender.send_to_target(player_handle.clone(), Message::Controller{event});
                    } else {println!("didn't get messager");}
                } else {println!("didn't get player handle");}
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
        context: &mut PluginContext,
        message: &UiMessage,
    ) {
        // Handle UI events here.
        if let Some(ButtonMessage::Click) = message.data() {

            // match message.destination() {
            //     self.class_button_handle => create_window(context.user_interface),

            //     self.start_button_handle => (), //START THE GAME
            // }

            if message.destination() == self.class_button_handle {
                create_window(context.user_interface);
                //println!("{:?}", message.clone());
            }
            if message.destination() == self.start_button_handle {
                
                //START THE GAME
                //self.start_button_handle.set_visibility(false);
                //println!("{:?}", message.clone());
            }
            if message.destination() == self.p1fig {
                self.playerclasses.insert(self.idList[0], Class::Fighter);
                //playerclasses.//update player1's class to Class::Fighter
            }
            if message.destination() == self.p1rog {
                self.playerclasses.insert(self.idList[0], Class::Rogue);
                //playerclasses.//update player1's class to Class::Fighter
            }
            if message.destination() == self.p1barb {
                self.playerclasses.insert(self.idList[0], Class::Barbarian);
                //playerclasses.//update player1's class to Class::Fighter
            }
            if message.destination() == self.p1wiz {
                self.playerclasses.insert(self.idList[0], Class::Wizard);
                //playerclasses.//update player1's class to Class::Fighter
            }
            if message.destination() == self.p2fig {
                self.playerclasses.insert(self.idList[1], Class::Fighter);
                //playerclasses.//update player1's class to Class::Fighter
            }
            if message.destination() == self.p2rog {
                self.playerclasses.insert(self.idList[1], Class::Rogue);
                //playerclasses.//update player1's class to Class::Fighter
            }
            if message.destination() == self.p2barb {
                self.playerclasses.insert(self.idList[1], Class::Barbarian);
                //playerclasses.//update player1's class to Class::Fighter
            }
            if message.destination() == self.p2wiz {
                self.playerclasses.insert(self.idList[1], Class::Wizard);
                //playerclasses.//update player1's class to Class::Fighter
            }
            if message.destination() == self.p3fig {
                self.playerclasses.insert(self.idList[2], Class::Fighter);
                //playerclasses.//update player1's class to Class::Fighter
            }
            if message.destination() == self.p3rog {
                self.playerclasses.insert(self.idList[2], Class::Rogue);
                //playerclasses.//update player1's class to Class::Fighter
            }
            if message.destination() == self.p3barb {
                self.playerclasses.insert(self.idList[2], Class::Barbarian);
                //playerclasses.//update player1's class to Class::Fighter
            }
            if message.destination() == self.p3wiz {
                self.playerclasses.insert(self.idList[2], Class::Wizard);
                //playerclasses.//update player1's class to Class::Fighter
            }
            if message.destination() == self.p4fig {
                self.playerclasses.insert(self.idList[3], Class::Fighter);
                //playerclasses.//update player1's class to Class::Fighter
            }
            if message.destination() == self.p4rog {
                self.playerclasses.insert(self.idList[3], Class::Rogue);
                //playerclasses.//update player1's class to Class::Fighter
            }
            if message.destination() == self.p4barb {
                self.playerclasses.insert(self.idList[3], Class::Barbarian);
                //playerclasses.//update player1's class to Class::Fighter
            }
            if message.destination() == self.p4wiz {
                self.playerclasses.insert(self.idList[3], Class::Wizard);
                //playerclasses.//update player1's class to Class::Fighter
            }

        }
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

        //reset messager to be set in the new scene
        //self.messager = None;

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
    facing: Vector3<f32>, //z axis should always be 0.0 here!
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
                        _ => (),
                    }

                },

                Hit{damage: dam, knockback: knock} => {
                    println!("took {} damage and {} knockback!", dam, knock);
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