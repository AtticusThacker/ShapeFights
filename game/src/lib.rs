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
        core::color::Color,
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

use messages::{
    Message,
    Message::{Controller, Hit, Parried},
};
use class::Class;

use create::*;
    // create_text_with_background,
    // set_script,
    // create_player,













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
    //start_button_handle: Handle<UiNode>,
 
    //holds the buttons for class choice selection and starting the game;
    //look at new() for more info
    menu: Vec<Handle<UiNode>>,

    id_list: Vec::<GamepadId>,

    // first 4 entries are the four "health:" widgets, then their respective strings of x for health bars
    hud: Vec<Handle<UiNode>>,
    // indicates if on_updtate should check the health bars for players
    phealthchanged: bool,
    //ctx: UserInterface,
    //HEALTH_TXT: String,
}


impl Game {
    pub fn new(scene_path: Option<&str>, context: PluginContext) -> Self {
        context
            .async_scene_loader
            .request(scene_path.unwrap_or("data/scene.rgs"));

        //create Heads Up Display
        let color1 = Color::opaque(66, 245, 158); 
        let color2 = Color::opaque(66, 167, 245);
        let color3 = Color::opaque(194, 136, 252);
        let color4 = Color::opaque(250, 135, 215);


        let mut hud = Vec::<Handle<UiNode>>::new();
        hud.push(create_text_with_background(context.user_interface, "health:", 100.0, 100.0, color1.clone()));
        hud.push(create_text_with_background(context.user_interface, "health:", 300.0, 100.0, color2.clone()));
        hud.push(create_text_with_background(context.user_interface, "health:", 500.0, 100.0, color3.clone()));
        hud.push(create_text_with_background(context.user_interface, "health:", 700.0, 100.0, color4.clone()));

        hud.push(create_text_with_background(context.user_interface, "", 175.0, 100.0, color1.clone()));
        hud.push(create_text_with_background(context.user_interface, "", 375.0, 100.0, color2.clone()));
        hud.push(create_text_with_background(context.user_interface, "", 575.0, 100.0, color3.clone()));
        hud.push(create_text_with_background(context.user_interface, "", 775.0, 100.0, color4.clone()));

        //create class choice menu
        let mut menu = Vec::<Handle<UiNode>>::new();
        //start button
        menu.push(create_player_class_button(context.user_interface, 800.0, 0.0, 200.0, 60.0, "Start Game!", Option::None, Some(HorizontalAlignment::Center)));
        //player 1 barb, rogue, wizard, fighter (in that order)
        menu.push(create_player_class_button(context.user_interface, 0.0, 0.0, 200.0, 60.0, "Barbarian", Some(VerticalAlignment::Bottom), Some(HorizontalAlignment::Right)));
        menu.push(create_player_class_button(context.user_interface, 0.0, 0.0, 100.0, 60.0, "Rogue", Some(VerticalAlignment::Bottom), Option::None));
        menu.push(create_player_class_button(context.user_interface, 0.0, 0.0, 200.0, 40.0, "Wizard", Some(VerticalAlignment::Bottom), Some(HorizontalAlignment::Right)));
        menu.push(create_player_class_button(context.user_interface, 0.0, 0.0, 100.0, 40.0, "Fighter", Some(VerticalAlignment::Bottom), Option::None));
        //player 2, same order
        menu.push(create_player_class_button(context.user_interface, 200.0, 0.0, 200.0, 60.0, "Barbarian", Some(VerticalAlignment::Bottom), Some(HorizontalAlignment::Right)));
        menu.push(create_player_class_button(context.user_interface, 200.0, 0.0, 100.0, 60.0, "Rogue", Some(VerticalAlignment::Bottom), Option::None));
        menu.push(create_player_class_button(context.user_interface, 200.0, 0.0, 200.0, 40.0, "Wizard", Some(VerticalAlignment::Bottom), Some(HorizontalAlignment::Right)));
        menu.push(create_player_class_button(context.user_interface, 200.0, 0.0, 100.0, 40.0, "Fighter", Some(VerticalAlignment::Bottom), Option::None));
        //player 3
        menu.push(create_player_class_button(context.user_interface, 400.0, 0.0, 200.0, 60.0, "Barbarian", Some(VerticalAlignment::Bottom), Some(HorizontalAlignment::Right)));
        menu.push(create_player_class_button(context.user_interface, 400.0, 0.0, 100.0, 60.0, "Rogue", Some(VerticalAlignment::Bottom), Option::None));
        menu.push(create_player_class_button(context.user_interface, 400.0, 0.0, 200.0, 40.0, "Wizard", Some(VerticalAlignment::Bottom), Some(HorizontalAlignment::Right)));
        menu.push(create_player_class_button(context.user_interface, 400.0, 0.0, 100.0, 40.0, "Fighter", Some(VerticalAlignment::Bottom), Option::None));
        //player 4
        menu.push(create_player_class_button(context.user_interface, 600.0, 0.0, 200.0, 60.0, "Barbarian", Some(VerticalAlignment::Bottom), Some(HorizontalAlignment::Right)));
        menu.push(create_player_class_button(context.user_interface, 600.0, 0.0, 100.0, 60.0, "Rogue", Some(VerticalAlignment::Bottom), Option::None));
        menu.push(create_player_class_button(context.user_interface, 600.0, 0.0, 200.0, 40.0, "Wizard", Some(VerticalAlignment::Bottom), Some(HorizontalAlignment::Right)));
        menu.push(create_player_class_button(context.user_interface, 600.0, 0.0, 100.0, 40.0, "Fighter", Some(VerticalAlignment::Bottom), Option::None));
        //player labels
        menu.push(create_player_class_button(context.user_interface, 0.0, 0.0, 200.0, 20.0, "Player 1", Some(VerticalAlignment::Bottom), Option::None));
        menu.push(create_player_class_button(context.user_interface, 200.0, 0.0, 200.0, 20.0, "Player 2", Some(VerticalAlignment::Bottom), Option::None));
        menu.push(create_player_class_button(context.user_interface, 400.0, 0.0, 200.0, 20.0, "Player 3", Some(VerticalAlignment::Bottom), Option::None));
        menu.push(create_player_class_button(context.user_interface, 600.0, 0.0, 200.0, 20.0, "Player 4", Some(VerticalAlignment::Bottom), Option::None));

        Self {
            //ctx: context.user_interface,
            scene: Handle::NONE,
            gils: Gilrs::new().unwrap(),
            players: HashMap::new(),
            playerclasses: HashMap::new(),
            idList: Vec::new(),

            menu,
            
            
            id_list: Vec::<GamepadId>::new(),
            hud,
            phealthchanged: false,
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
        while let Some(gEvent { id, event, .. }) = self.gils.next_event() {
            
            //matching on the event type 
            match event {
                Connected => {
                    // //NOTE on these; context has a field scenes which is a scenecontainer
                    // // and scenecontainer can be indexed by Handle<Scene>, which is what self.scene is.
                    // // then, graph can be indexed by a Handle<Node>>, to get a dynamic object that we
                    // // have to 'downcast' using the .cast_mut thingy to get the actual player object.
                    // // its complicated i know, but it works!

                    self.idList.push(id);

                },
                //send the controller event to the player
                _ => if let Some(player_handle) = self.players.get(&id) {
                    if let Some(message_sender) = &messager {
                        message_sender.send_to_target(player_handle.clone(), Message::Controller{event});
                    } else {println!("didn't get messager");}
                } else {println!("didn't get player handle");}

            }  
        }  

        // changes the number of xs in the health status bar
        let ctx = &mut context.user_interface;
        // let health_txt = "health:";
        //let text = "".to_string();

        // updates all player health ui
        if self.phealthchanged {
        
            // for player 1
            if self.players.len() > 0 {
                // creates health variable here
                let mut h: u32 = 10;

            

                // gets the player handle from hash map for player 1
                if let Some(player_script) = self.players.get(&self.id_list[0]) {
                    // gets the node
                    let node1 = &mut context.scenes[self.scene].graph[player_script.clone()];
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

                ctx.send_message(TextMessage::text(
                    self.hud[4],
                    MessageDirection::ToWidget,
                    text.to_owned(),
                ));
                let mut p: Handle<UiNode> = self.hud[4];
                if let Some(wid) = ctx.build_ctx()[self.hud[4].clone()].cast::<Text>() {
                    p = wid.parent.clone();
                }
                ctx.build_ctx()[p].set_visibility(true);
            }

            // player 2
            if self.players.len() > 1 {
                // creates health variable here
                let mut h: u32 = 10;

                // gets the player handle from hash map for player 2
                if let Some(player_script) = self.players.get(&self.id_list[1]) {
                    // gets the node
                    let node1 = &mut context.scenes[self.scene].graph[player_script.clone()];
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
                ctx.send_message(TextMessage::text(
                    self.hud[5],
                    MessageDirection::ToWidget,
                    text.to_owned(),
                ));
                let mut p: Handle<UiNode> = self.hud[5];
                if let Some(wid) = ctx.build_ctx()[self.hud[5].clone()].cast::<Text>() {
                    p = wid.parent.clone();
                }
                ctx.build_ctx()[p].set_visibility(true);
            }

            // player 3
            if self.players.len() > 2 {
                // creates health variable here
                let mut h: u32 = 10;
                // gets the player handle from hash map for player 3
                if let Some(player_script) = self.players.get(&self.id_list[2]) {
                    // gets the node
                    let node1 = &mut context.scenes[self.scene].graph[player_script.clone()];
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

                ctx.send_message(TextMessage::text(
                    self.hud[6],
                    MessageDirection::ToWidget,
                    text.to_owned(),
                ));
                let mut p: Handle<UiNode> = self.hud[6];
                if let Some(wid) = ctx.build_ctx()[self.hud[6].clone()].cast::<Text>() {
                    p = wid.parent.clone();
                }
                ctx.build_ctx()[p].set_visibility(true);
            }

            // player 4
            if self.players.len() > 3 {
                // creates health variable here
                let mut h = 10;
                // gets the player handle from hash map for player 4
                if let Some(player_script) = self.players.get(&self.id_list[3]) {
                    // gets the node
                    let node1 = &mut context.scenes[self.scene].graph[player_script.clone()];
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

                ctx.send_message(TextMessage::text(
                    self.hud[7],
                    MessageDirection::ToWidget,
                    text.to_owned(),
                ));
                let mut p: Handle<UiNode> = self.hud[7];
                if let Some(wid) = ctx.build_ctx()[self.hud[7].clone()].cast::<Text>() {
                    p = wid.parent.clone();
                }
                ctx.build_ctx()[p].set_visibility(true);
            }
            self.phealthchanged = false;
        }

        // loop{
        //     if let None = ctx.poll_message() {
        //         break;
        //     }
        // }
        ctx.poll_message();
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
            let mut i = 0;

            let c = |h: &&Handle<UiNode>| -> bool {

                if message.destination() == **h {
                    true
                } else {
                    i += 1;
                    false
                }
            };

            if let Some(_) = self.menu.iter().find(c) {
                match i {

                    //player 1 class buttons
                    1 if self.idList.len() > 0 => {self.playerclasses.insert(self.idList[0], Class::Barbarian);},
                    2 if self.idList.len() > 0 => {self.playerclasses.insert(self.idList[0], Class::Rogue);},
                    3 if self.idList.len() > 0 => {self.playerclasses.insert(self.idList[0], Class::Wizard);},
                    4 if self.idList.len() > 0 => {self.playerclasses.insert(self.idList[0], Class::Fighter);},

                    //player 2 class buttons
                    5 if self.idList.len() > 1 => {self.playerclasses.insert(self.idList[0], Class::Barbarian);},
                    6 if self.idList.len() > 1 => {self.playerclasses.insert(self.idList[0], Class::Rogue);},
                    7 if self.idList.len() > 1 => {self.playerclasses.insert(self.idList[0], Class::Wizard);},
                    8 if self.idList.len() > 1 => {self.playerclasses.insert(self.idList[0], Class::Fighter);},

                    //player 3 class buttons
                    9 if self.idList.len() > 2 => {self.playerclasses.insert(self.idList[0], Class::Barbarian);},
                    10 if self.idList.len() > 2 => {self.playerclasses.insert(self.idList[0], Class::Rogue);},
                    11 if self.idList.len() > 2 => {self.playerclasses.insert(self.idList[0], Class::Wizard);},
                    12 if self.idList.len() > 2 => {self.playerclasses.insert(self.idList[0], Class::Fighter);},

                    //player 4 class buttons
                    13 if self.idList.len() > 3 => {self.playerclasses.insert(self.idList[0], Class::Barbarian);},
                    14 if self.idList.len() > 3 => {self.playerclasses.insert(self.idList[0], Class::Rogue);},
                    15 if self.idList.len() > 3 => {self.playerclasses.insert(self.idList[0], Class::Wizard);},
                    16 if self.idList.len() > 3 => {self.playerclasses.insert(self.idList[0], Class::Fighter);},

                    //start button
                    0 => {
                        for (player, class) in &self.playerclasses {
                            println!("{player:?} is {class:?}");
                        }
                        let ctx = &mut context.user_interface;
        
                        //hide the class selection menu
                        for b in &self.menu {
                            ctx.build_ctx()[b.clone()].set_visibility(false);
                        }
        
                        let mut i = 1;
                        for (id, class) in self.playerclasses.clone() {
                            create_player(i, class, id, context, self);
                            i += 1;
                        }
        
                        let ctx = &mut context.user_interface;
        
                        for i in 0..self.players.len() {
                            // makes "health:" visible
                            ctx.build_ctx()[self.hud[i].clone()].set_visibility(true);
                            let mut q: Handle<UiNode> = self.hud[i];
                            if let Some(txt) = ctx.build_ctx()[self.hud[i].clone()].cast::<Text>() {
                                q = txt.parent.clone();
                            }
                            ctx.build_ctx()[q].set_visibility(true);
                        }
        
                    },

                    _ => (),


                }
            }
        }
    }
    
    fn on_scene_begin_loading(&mut self, _path: &Path, ctx: &mut PluginContext) {
        if self.scene.is_some() {
            ctx.scenes.remove(self.scene);
        }
    }

    fn on_scene_loaded(
        &mut self,
        _path: &Path,
        scene: Handle<Scene>,
        _data: &[u8],
        _context: &mut PluginContext,
    ) {
        self.scene = scene;
     }
}

#[derive(Visit, Reflect, Debug, Clone, Default, PartialEq)]
pub enum PlayerState {
    #[default]
    Idle,
    Charging,
    Dead,
    Riposting,
    //the field holds the number of frames the player is into the action
    Attacking(i32),
    Hit(i32),
    Parry(i32),
}

#[derive(Visit, Reflect, Debug, Clone, Default)]
pub struct Player{
    class: Class,
    state: PlayerState,
    weapon: Option<Handle<Node>>,
    cooldown: i32,
    facing: Vector3<f32>, //z axis should always be 0.0 here!
    health: u32,
    charges: i32,
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
    fn on_init(&mut self, _context: &mut ScriptContext) {}
    
    // Put start logic - it is called when every other script is already initialized.
    fn on_start(&mut self, context: &mut ScriptContext) { 
        context.message_dispatcher.subscribe_to::<Message>(context.handle);
        self.class.clone().startup(self, context);
    }

    // Called whenever there is an event from OS (mouse click, keypress, etc.)
    fn on_os_event(&mut self, _event: &Event<()>, _context: &mut ScriptContext) {}

    // Called every frame at fixed rate of 60 FPS.
    fn on_update(&mut self, context: &mut ScriptContext) {
        match self.state {
            PlayerState::Dead => return(),
            PlayerState::Attacking(frame) => {self.class.clone().cont_attack(self, frame, context)},
            PlayerState::Hit(frame) => {self.class.clone().cont_hit(self, frame, context)},

            PlayerState::Charging => {self.class.clone().charging(self, context)},
            PlayerState::Parry(frame) => {self.class.clone().cont_parry(self, frame, context)},
            PlayerState::Riposting => {self.class.clone().riposting(self, context)}
            _ => (),
        }

        match self.class {
            Class::Barbarian if self.cooldown == 8 => {
                if let Some(rigid_body) = context.scene.graph[context.handle.clone()].cast_mut::<RigidBody>(){
                    rigid_body.set_lin_vel(Vector2::new(0.0, 0.0));
                }
            }
            _ => {},
        };

        self.cooldown += 1;
        Class::update_look(self.facing.clone(), &mut context.scene.graph[context.handle.clone()]);

    }

    fn on_message(&mut self,
        message: &mut dyn ScriptMessagePayload,
        ctx: &mut ScriptMessageContext,
    ) {
        match self.state {
            PlayerState::Dead => return(),
            _ => {}
        }

        if let Some(message) = message.downcast_ref::<Message>(){
            match message {
                Controller{event} => {
                    match event {
                        // put the various controller events here, as well as calls to
                        //the correct class methods-- player has a class field now!
                        AxisChanged(axis, value, _code) => self.class.clone().moveplayer(self, axis, value, ctx),
                        //must clone class for any method that takes a 'self' as well.
                        ButtonPressed(button, _) => {
                            match button {
                                RightTrigger => self.class.clone().start_melee_attack(self, ctx),
                                LeftTrigger => self.class.clone().projectiles(self, ctx),
                                RightThumb => self.class.clone().parry(self, ctx),
                                _ => (),
                            }},
                        // ButtonPressed(RightTrigger, _) => self.class.clone().start_melee_attack(self, ctx),
                        // //projectiles
                        // ButtonPressed(LeftTrigger, _) => self.class.clone().projectiles(self, ctx),
                        // //parrying
                        // ButtonPressed(RightThumb, _) => self.class.clone().parry(self, ctx),
                        _ => (),
                    }
                },

                Hit{damage: dam, knockback: knock, body: bod, sender: send} => {
                    self.class.clone().takehit(self, dam.clone(), knock.clone(), bod.clone(), send.clone(), ctx);
                },
                Parried{} => {self.class.clone().parried(self, ctx)},
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
    facing: Vector3<f32>,
    hit: bool,
    life: u32,
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
    fn on_init(&mut self, _context: &mut ScriptContext) {}
    
    // Put start logic - it is called when every other script is already initialized.
    fn on_start(&mut self, _context: &mut ScriptContext) { }

    // Called whenever there is an event from OS (mouse click, keypress, etc.)
    fn on_os_event(&mut self, _event: &Event<()>, _context: &mut ScriptContext) {}

    // Called every frame at fixed rate of 60 FPS.
    fn on_update(&mut self, ctx: &mut ScriptContext) {
        if self.life == 0 {
            ctx.scene.graph.remove_node(ctx.handle);
            return;
        }

        self.life -= 1;
        //prevent crash in last frame after deletion.
        if self.hit {return;}
        //check for a hit:
        //find the collider of the weapon
        if let Some((_,colnode)) = ctx.scene.graph.find(ctx.handle.clone(), &mut |c| c.is_collider2d()) {
            let collider = colnode.as_collider2d();
            // iterate over collisions
            for i in collider.intersects(&ctx.scene.graph.physics2d) {
                //for each active contact
                if i.has_any_active_contact {
                    //find its parent
                    if let Some((target, _t)) = ctx.scene.graph.find_up(i.collider1, &mut |c| c.try_get_script::<Player>().is_some()) {
                        let mut knockvec = self.facing.clone();
                        knockvec.set_magnitude(3.0);
                        ctx.message_sender.send_to_target(target, Message::Hit{
                            damage: 3, 
                            knockback: knockvec,
                            body: target.clone(),
                            sender: ctx.handle.clone()
                        });
                    }
                    self.hit = true;   
                }
            }
        }
        if self.hit {
            //destroy the projectile 5 frames after hit
            self.life = 5;
            ctx.scene.graph[ctx.handle].set_visibility(false);
        }
    }

    // Returns unique script ID for serialization needs.
    fn id(&self) -> Uuid {
        Self::type_uuid()
    }
}

// TODO:
// clean up robert's code, move player and projectile into separate files 
// (re-import them here bc that's probably what fyrox needs)
// maybe even move game into its own spot? make lib all nice
// then we can work on class...