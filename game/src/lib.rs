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

use create::{
    create_text_with_background,
    create_cube_rigid_body,
    create_rect,
    set_script,
};













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

    player1: Handle<UiNode>,
    player2: Handle<UiNode>,
    player3: Handle<UiNode>,
    player4: Handle<UiNode>,

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

    id_list: Vec::<GamepadId>,

    // first 4 entries are the four "health:" widgets, then their respective strings of x for health bars
    hud: Vec<Handle<UiNode>>,
    // player_text1: Handle<UiNode>,
    // hud[1]: Handle<UiNode>,
    // hud[2]: Handle<UiNode>,
    // hud[3]: Handle<UiNode>,
    // hud[4]: Handle<UiNode>,
    // hud[5]: Handle<UiNode>,
    // hud[6]: Handle<UiNode>,
    // hud[7]: Handle<UiNode>,
    //ctx: UserInterface,
    //HEALTH_TXT: String,
}

fn start_button(ui: &mut UserInterface) -> Handle<UiNode> {
    ButtonBuilder::new(
        WidgetBuilder::new()
        .with_desired_position(Vector2::new(800.0, 0.0))
        .with_width(200.0)
        .with_height(60.0),
    )
    .with_content(
        TextBuilder::new(WidgetBuilder::new())
            .with_text("Start Game")
            .with_horizontal_text_alignment(HorizontalAlignment::Center)
            .build(&mut ui.build_ctx()),
    )
    .build(&mut ui.build_ctx())
}

fn p1fig(ui: &mut UserInterface) -> Handle<UiNode> {
    ButtonBuilder::new(
        WidgetBuilder::new()
        .with_width(100.0)
        .with_height(40.0),  
    )
    .with_content(
        TextBuilder::new(WidgetBuilder::new())
            .with_text("Fighter")
            .with_vertical_text_alignment(VerticalAlignment::Bottom)
            
            .build(&mut ui.build_ctx()),
    )
    .build(&mut ui.build_ctx())
}

fn p1barb(ui: &mut UserInterface) -> Handle<UiNode> {
    ButtonBuilder::new(
        WidgetBuilder::new()
        .with_width(200.0)
        .with_height(40.0),
    )
    .with_content(
        TextBuilder::new(WidgetBuilder::new())
            .with_text("Barbarian")
            .with_horizontal_text_alignment(HorizontalAlignment::Right)
            .with_vertical_text_alignment(VerticalAlignment::Bottom)
            
            .build(&mut ui.build_ctx()),
    )
    .build(&mut ui.build_ctx())
}

fn p1rog(ui: &mut UserInterface) -> Handle<UiNode> {
    ButtonBuilder::new(
        WidgetBuilder::new()
        .with_width(100.0)
        .with_height(60.0), 
    )
    .with_content(
        TextBuilder::new(WidgetBuilder::new())
            .with_text("Rogue")
            .with_vertical_text_alignment(VerticalAlignment::Bottom)
            
            .build(&mut ui.build_ctx()),
    )
    .build(&mut ui.build_ctx())
}

fn p1wiz(ui: &mut UserInterface) -> Handle<UiNode> {
    ButtonBuilder::new(
        WidgetBuilder::new()
        .with_width(200.0)
        .with_height(60.0),
    )
    .with_content(
        TextBuilder::new(WidgetBuilder::new())
            .with_text("Wizard")
            .with_horizontal_text_alignment(HorizontalAlignment::Right)
            .with_vertical_text_alignment(VerticalAlignment::Bottom)
            
            .build(&mut ui.build_ctx()),
    )
    .build(&mut ui.build_ctx())
}

fn p2fig(ui: &mut UserInterface) -> Handle<UiNode> {
    ButtonBuilder::new(
        WidgetBuilder::new()
        .with_desired_position(Vector2::new(200.0, 0.0))
        .with_width(100.0)
        .with_height(40.0),
   )
    .with_content(
        TextBuilder::new(WidgetBuilder::new())
            .with_text("Fighter")
            .with_vertical_text_alignment(VerticalAlignment::Bottom)
            
            .build(&mut ui.build_ctx()),
    )
    .build(&mut ui.build_ctx())
}

fn p2barb(ui: &mut UserInterface) -> Handle<UiNode> {
    ButtonBuilder::new(
        WidgetBuilder::new()
        .with_desired_position(Vector2::new(200.0, 0.0))
        .with_width(200.0)
        .with_height(40.0),    
    )
    .with_content(
        TextBuilder::new(WidgetBuilder::new())
            .with_text("Barbarian")
            .with_horizontal_text_alignment(HorizontalAlignment::Right)
            .with_vertical_text_alignment(VerticalAlignment::Bottom)
            
            .build(&mut ui.build_ctx()),
    )
    .build(&mut ui.build_ctx())
}

fn p2rog(ui: &mut UserInterface) -> Handle<UiNode> {
    ButtonBuilder::new(
        WidgetBuilder::new()
        .with_desired_position(Vector2::new(200.0, 0.0))
        .with_width(100.0)
        .with_height(60.0),   
    )
    .with_content(
        TextBuilder::new(WidgetBuilder::new())
            .with_text("Rogue")
            .with_vertical_text_alignment(VerticalAlignment::Bottom)
            
            .build(&mut ui.build_ctx()),
    )
    .build(&mut ui.build_ctx())
}

fn p2wiz(ui: &mut UserInterface) -> Handle<UiNode> {
    ButtonBuilder::new(
        WidgetBuilder::new()
        .with_desired_position(Vector2::new(200.0, 0.0))
        .with_width(200.0)
        .with_height(60.0),   
    )
    .with_content(
        TextBuilder::new(WidgetBuilder::new())
            .with_text("Wizard")
            .with_horizontal_text_alignment(HorizontalAlignment::Right)
            .with_vertical_text_alignment(VerticalAlignment::Bottom)
            
            .build(&mut ui.build_ctx()),
    )
    .build(&mut ui.build_ctx())
}

fn p3fig(ui: &mut UserInterface) -> Handle<UiNode> {
    ButtonBuilder::new(
        WidgetBuilder::new()
        .with_desired_position(Vector2::new(400.0, 0.0))
        .with_width(100.0)
        .with_height(40.0),   
    )
    .with_content(
        TextBuilder::new(WidgetBuilder::new())
            .with_text("Fighter")
            .with_vertical_text_alignment(VerticalAlignment::Bottom)
            
            .build(&mut ui.build_ctx()),
    )
    .build(&mut ui.build_ctx())
}

fn p3barb(ui: &mut UserInterface) -> Handle<UiNode> {
    ButtonBuilder::new(
        WidgetBuilder::new()
        .with_desired_position(Vector2::new(400.0, 0.0))
        .with_width(200.0)
        .with_height(40.0),    
    )
    .with_content(
        TextBuilder::new(WidgetBuilder::new())
            .with_text("Barbarian")
            .with_horizontal_text_alignment(HorizontalAlignment::Right)
            .with_vertical_text_alignment(VerticalAlignment::Bottom)
            
            .build(&mut ui.build_ctx()),
    )
    .build(&mut ui.build_ctx())
}

fn p3rog(ui: &mut UserInterface) -> Handle<UiNode> {
    ButtonBuilder::new(
        WidgetBuilder::new()
        .with_desired_position(Vector2::new(400.0, 0.0))
        .with_width(100.0)
        .with_height(60.0),  
    )
    .with_content(
        TextBuilder::new(WidgetBuilder::new())
            .with_text("Rogue")
            .with_vertical_text_alignment(VerticalAlignment::Bottom)
            
            .build(&mut ui.build_ctx()),
    )
    .build(&mut ui.build_ctx())
}

fn p3wiz(ui: &mut UserInterface) -> Handle<UiNode> {
    ButtonBuilder::new(
        WidgetBuilder::new()
        .with_desired_position(Vector2::new(400.0, 0.0))
        .with_width(200.0)
        .with_height(60.0),   
    )
    .with_content(
        TextBuilder::new(WidgetBuilder::new())
            .with_text("Wizard")
            .with_horizontal_text_alignment(HorizontalAlignment::Right)
            .with_vertical_text_alignment(VerticalAlignment::Bottom)
            
            .build(&mut ui.build_ctx()),
    )
    .build(&mut ui.build_ctx())
}

fn p4fig(ui: &mut UserInterface) -> Handle<UiNode> {
    ButtonBuilder::new(
        WidgetBuilder::new()
        .with_desired_position(Vector2::new(600.0, 0.0))
        .with_width(100.0)
        .with_height(40.0),   
    )
    .with_content(
        TextBuilder::new(WidgetBuilder::new())
            .with_text("Fighter")
            .with_vertical_text_alignment(VerticalAlignment::Bottom)
            
            .build(&mut ui.build_ctx()),
    )
    .build(&mut ui.build_ctx())
}

fn p4barb(ui: &mut UserInterface) -> Handle<UiNode> {
    ButtonBuilder::new(
        WidgetBuilder::new()
        .with_desired_position(Vector2::new(600.0, 0.0))
        .with_width(200.0)
        .with_height(40.0),
    )
    .with_content(
        TextBuilder::new(WidgetBuilder::new())
            .with_text("Barbarian")
            .with_horizontal_text_alignment(HorizontalAlignment::Right)
            .with_vertical_text_alignment(VerticalAlignment::Bottom)
            
            .build(&mut ui.build_ctx()),
    )
    .build(&mut ui.build_ctx())
}

fn p4rog(ui: &mut UserInterface) -> Handle<UiNode> {
    ButtonBuilder::new(
        WidgetBuilder::new()
        .with_desired_position(Vector2::new(600.0, 0.0))
        .with_width(100.0)
        .with_height(60.0),
    )
    .with_content(
        TextBuilder::new(WidgetBuilder::new())
            .with_text("Rogue")
            .with_vertical_text_alignment(VerticalAlignment::Bottom)
            
            .build(&mut ui.build_ctx()),
    )
    .build(&mut ui.build_ctx())
}

fn p4wiz(ui: &mut UserInterface) -> Handle<UiNode> {
    ButtonBuilder::new(
        WidgetBuilder::new()
        .with_desired_position(Vector2::new(600.0, 0.0))
        .with_width(200.0)
        .with_height(60.0),
    )
    .with_content(
        TextBuilder::new(WidgetBuilder::new())
            .with_text("Wizard")
            .with_horizontal_text_alignment(HorizontalAlignment::Right)
            .with_vertical_text_alignment(VerticalAlignment::Bottom)
            
            .build(&mut ui.build_ctx()),
    )
    .build(&mut ui.build_ctx())
}

fn player1(ui: &mut UserInterface) -> Handle<UiNode> {
    ButtonBuilder::new(
        WidgetBuilder::new()
        .with_width(200.0)
        .with_height(20.0),
    )
    .with_content(
        TextBuilder::new(WidgetBuilder::new())
            .with_text("Player 1")
            .with_vertical_text_alignment(VerticalAlignment::Bottom)
            .build(&mut ui.build_ctx()),
    )
    .build(&mut ui.build_ctx())
}

fn player2(ui: &mut UserInterface) -> Handle<UiNode> {
    ButtonBuilder::new(
        WidgetBuilder::new()
        .with_desired_position(Vector2::new(200.0, 0.0))
        .with_width(200.0)
        .with_height(20.0),
    )
    .with_content(
        TextBuilder::new(WidgetBuilder::new())
            .with_text("Player 2")
            .with_vertical_text_alignment(VerticalAlignment::Bottom)
            .build(&mut ui.build_ctx()),
    )
    .build(&mut ui.build_ctx())
}

fn player3(ui: &mut UserInterface) -> Handle<UiNode> {
    ButtonBuilder::new(
        WidgetBuilder::new()
        .with_desired_position(Vector2::new(400.0, 0.0))
        .with_width(200.0)
        .with_height(20.0),
    )
    .with_content(
        TextBuilder::new(WidgetBuilder::new())
            .with_text("Player 3")
            .with_vertical_text_alignment(VerticalAlignment::Bottom)
            .build(&mut ui.build_ctx()),
    )
    .build(&mut ui.build_ctx())
}

fn player4(ui: &mut UserInterface) -> Handle<UiNode> {
    ButtonBuilder::new(
        WidgetBuilder::new()
        .with_desired_position(Vector2::new(600.0, 0.0))
        .with_width(200.0)
        .with_height(20.0),
    )
    .with_content(
        TextBuilder::new(WidgetBuilder::new())
            .with_text("Player 4")
            .with_vertical_text_alignment(VerticalAlignment::Bottom)
            .build(&mut ui.build_ctx()),
    )
    .build(&mut ui.build_ctx())
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

        Self {
            //ctx: context.user_interface,
            scene: Handle::NONE,
            gils: Gilrs::new().unwrap(),
            players: HashMap::new(),
            playerclasses: HashMap::new(),
            idList: Vec::new(),

            p1wiz: p1wiz(context.user_interface),
            p1rog: p1rog(context.user_interface),
            p1barb: p1barb(context.user_interface),
            p1fig: p1fig(context.user_interface),
            
            p2wiz: p2wiz(context.user_interface),
            p2rog: p2rog(context.user_interface),
            p2barb: p2barb(context.user_interface),
            p2fig: p2fig(context.user_interface),

            p3wiz: p3wiz(context.user_interface),
            p3rog: p3rog(context.user_interface),
            p3barb: p3barb(context.user_interface),
            p3fig: p3fig(context.user_interface),
            
            p4wiz: p4wiz(context.user_interface),
            p4rog: p4rog(context.user_interface),
            p4barb: p4barb(context.user_interface),
            p4fig: p4fig(context.user_interface),

            player1: player1(context.user_interface),
            player2: player2(context.user_interface),
            player3: player3(context.user_interface),
            player4: player4(context.user_interface),

            start_button_handle: start_button(context.user_interface),
            
            
            id_list: Vec::<GamepadId>::new(),
            hud,
            // player_text1: create_text_with_background_1(context.user_interface, "health:", 100.0, 100.0),
            // hud[1]: create_text_with_background_2(context.user_interface, "health:", 300.0, 100.0),
            // hud[2]: create_text_with_background_3(context.user_interface, "health:", 500.0, 100.0),
            // hud[3]: create_text_with_background_4(context.user_interface, "health:", 700.0, 100.0),
            // hud[4]: create_text_with_background_1(context.user_interface, "", 175.0, 100.0),
            // hud[5]: create_text_with_background_2(context.user_interface, "", 375.0, 100.0),
            // hud[6]: create_text_with_background_3(context.user_interface, "", 575.0, 100.0),
            // hud[7]: create_text_with_background_4(context.user_interface, "", 775.0, 100.0),
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
        
        // for player 1
        if self.players.len() > 0 {
            // creates health variable here
            let mut h: u32 = 10;

            // makes "health:" visible
            ctx.build_ctx()[self.hud[0].clone()].set_visibility(true);
            let mut q: Handle<UiNode> = self.hud[0];
            if let Some(txt) = ctx.build_ctx()[self.hud[0].clone()].cast::<Text>() {
                q = txt.parent.clone();
            }
            ctx.build_ctx()[q].set_visibility(true);
        

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
            // makes "health:" visible
            ctx.build_ctx()[self.hud[1].clone()].set_visibility(true);
            let mut q: Handle<UiNode> = self.hud[1];
            if let Some(txt) = ctx.build_ctx()[self.hud[1].clone()].cast::<Text>() {
                q = txt.parent.clone();
            }
            ctx.build_ctx()[q].set_visibility(true);

            // gets the player handle from hash map for player 1
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
            // makes "health:" visible
            ctx.build_ctx()[self.hud[2].clone()].set_visibility(true);
            let mut q: Handle<UiNode> = self.hud[2];
            if let Some(txt) = ctx.build_ctx()[self.hud[2].clone()].cast::<Text>() {
                q = txt.parent.clone();
            }
            ctx.build_ctx()[q].set_visibility(true);
            // let playerText3: Handle<UiNode> = create_text_with_background_2(ctx, health_txt, 500.0, 1000.0);
            // gets the player handle from hash map for player 1
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
            // makes "health:" visible
            ctx.build_ctx()[self.hud[3].clone()].set_visibility(true);
            let mut q: Handle<UiNode> = self.hud[3];
            if let Some(txt) = ctx.build_ctx()[self.hud[3].clone()].cast::<Text>() {
                q = txt.parent.clone();
            }
            ctx.build_ctx()[q].set_visibility(true);
            // gets the player handle from hash map for player 1
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
            //if only a match block could work :(
            if message.destination() == self.p1fig {
                if self.idList.len() > 0 {
                    self.playerclasses.insert(self.idList[0], Class::Fighter);
                }
            }
            if message.destination() == self.p1rog {
                if self.idList.len() > 0 {
                    self.playerclasses.insert(self.idList[0], Class::Rogue);
                }
            }
            if message.destination() == self.p1barb {
                if self.idList.len() > 0 {
                    self.playerclasses.insert(self.idList[0], Class::Barbarian);
                }               
            }
            if message.destination() == self.p1wiz {
                if self.idList.len() > 0 {
                    self.playerclasses.insert(self.idList[0], Class::Wizard);
                }               
            }
            if message.destination() == self.p2fig {
                if self.idList.len() > 1 {
                    self.playerclasses.insert(self.idList[1], Class::Fighter);
                }               
            }
            if message.destination() == self.p2rog {
                if self.idList.len() > 1 {
                    self.playerclasses.insert(self.idList[1], Class::Rogue);
                }                
            }
            if message.destination() == self.p2barb {
                if self.idList.len() > 1 {
                    self.playerclasses.insert(self.idList[1], Class::Barbarian);
                }              
            }
            if message.destination() == self.p2wiz {
                if self.idList.len() > 1 {
                    self.playerclasses.insert(self.idList[1], Class::Wizard);
                }            
            }
            if message.destination() == self.p3fig {
                if self.idList.len() > 2 {
                    self.playerclasses.insert(self.idList[2], Class::Fighter);
                }            
            }
            if message.destination() == self.p3rog {
                if self.idList.len() > 2 {
                    self.playerclasses.insert(self.idList[2], Class::Rogue);
                }            
            }
            if message.destination() == self.p3barb {
                if self.idList.len() > 2 {
                    self.playerclasses.insert(self.idList[2], Class::Barbarian);
                }            
            }
            if message.destination() == self.p3wiz {
                if self.idList.len() > 2 {
                    self.playerclasses.insert(self.idList[2], Class::Wizard);
                }            
            }
            if message.destination() == self.p4fig {
                if self.idList.len() > 3 {
                    self.playerclasses.insert(self.idList[3], Class::Fighter);
                }             
            }
            if message.destination() == self.p4rog {
                if self.idList.len() > 3 {
                    self.playerclasses.insert(self.idList[3], Class::Rogue);
                }             
            }
            if message.destination() == self.p4barb {
                if self.idList.len() > 3 {
                    self.playerclasses.insert(self.idList[3], Class::Barbarian);
                }             
            }
            if message.destination() == self.p4wiz {
                if self.idList.len() > 3 {
                    self.playerclasses.insert(self.idList[3], Class::Wizard);
                }            
            }

            if message.destination() == self.start_button_handle {
                for (player, class) in &self.playerclasses {
                    println!("{player:?} is {class:?}");
                }
                let ctx = &mut context.user_interface;

                ctx.build_ctx()[self.player1.clone()].set_visibility(false);
                ctx.build_ctx()[self.player2.clone()].set_visibility(false);
                ctx.build_ctx()[self.player3.clone()].set_visibility(false);
                ctx.build_ctx()[self.player4.clone()].set_visibility(false);

                ctx.build_ctx()[self.p1fig.clone()].set_visibility(false);
                ctx.build_ctx()[self.p1rog.clone()].set_visibility(false);
                ctx.build_ctx()[self.p1barb.clone()].set_visibility(false);
                ctx.build_ctx()[self.p1wiz.clone()].set_visibility(false);

                ctx.build_ctx()[self.p2fig.clone()].set_visibility(false);
                ctx.build_ctx()[self.p2rog.clone()].set_visibility(false);
                ctx.build_ctx()[self.p2barb.clone()].set_visibility(false);
                ctx.build_ctx()[self.p2wiz.clone()].set_visibility(false);

                ctx.build_ctx()[self.p3fig.clone()].set_visibility(false);
                ctx.build_ctx()[self.p3rog.clone()].set_visibility(false);
                ctx.build_ctx()[self.p3barb.clone()].set_visibility(false);
                ctx.build_ctx()[self.p3wiz.clone()].set_visibility(false);

                ctx.build_ctx()[self.p4fig.clone()].set_visibility(false);
                ctx.build_ctx()[self.p4rog.clone()].set_visibility(false);
                ctx.build_ctx()[self.p4barb.clone()].set_visibility(false);
                ctx.build_ctx()[self.p4wiz.clone()].set_visibility(false);

                ctx.build_ctx()[self.start_button_handle.clone()].set_visibility(false);

                let mut i = 1;
                for (id, class) in self.playerclasses.clone() {
                    create_player(i, class, id, context, self);
                    i += 1;
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






fn create_player(player_num: i8, player_class: Class, id: GamepadId, context: &mut PluginContext, game: &mut Game) {
    let mut player_data = (Vec::<u8>::new(), Vec::<f32>::new());

    if player_num == 1 {
        player_data.0 = Vec::from([66, 245, 158]);
        player_data.1 = Vec::from([6.0, 3.0, 0.0]);
    }
    else if player_num == 2 {
        player_data.0 = Vec::from([66, 167, 245]);
        player_data.1 = Vec::from([-6.0, 3.0, 0.0]);
    }
    else if player_num == 3 {
        player_data.0 = Vec::from([194, 136, 252]);
        player_data.1 = Vec::from([-6.0, -3.0, 0.0]);
    }
    else if player_num == 4 {
        player_data.0 = Vec::from([250, 135, 215]);
        player_data.1 = Vec::from([6.0, -3.0, 0.0]);
    }
    else {
        println!("Player cap reached");
        return;
    }

    //path to correct sprite, pre-coloring based on team
    let path = match player_class.clone() {
        Class::Barbarian => {"data/White_square.png".to_string()},
        Class::Fighter => {"data/White_circle.png".to_string()},
        Class::Rogue => {"data/White_triangle.png".to_string()},
        Class::Wizard => {"data/White_star.png".to_string()},

    };

    //create a new player
    let player_handle = create_cube_rigid_body(&mut context.scenes[game.scene].graph);
    //create a sprite for the player
    let sprite_handle = create_rect(&mut context.scenes[game.scene].graph, context.resource_manager, &player_data.0, path);
    //make the sprite a child of the player
    context.scenes[game.scene].graph.link_nodes(sprite_handle, player_handle);
    //add the player to the game's struct
    game.players.insert(id, player_handle);
    // add player ID to vector of IDs
    game.id_list.push(id);

    match player_class {
        Class::Barbarian => {
            set_script(&mut context.scenes[game.scene].graph[player_handle.clone()], 
            Player{
                class: Class::Barbarian,
                state: PlayerState::Idle,
                weapon: None,
                    cooldown: 0,
                    facing: Vector3::new(0.0,1.0,0.0),
                health: 14,
                charges: 0,
            })
        },
        Class::Fighter => {
            set_script(&mut context.scenes[game.scene].graph[player_handle.clone()], 
            Player{
                class: Class::Fighter,
                state: PlayerState::Idle,
                weapon: None,
                    cooldown: 0,
                    facing: Vector3::new(0.0,1.0,0.0),
                health: 12,
                charges: 0,
            })
        },
        Class::Rogue => {
            set_script(&mut context.scenes[game.scene].graph[player_handle.clone()], 
            Player{
            class: Class::Rogue,
            state: PlayerState::Idle,
            weapon: None,
                cooldown: 0,
                facing: Vector3::new(0.0,1.0,0.0),
            health: 7,
            charges: 0,
            })
        },
        Class::Wizard => {
            set_script(&mut context.scenes[game.scene].graph[player_handle.clone()], 
            Player{
            class: Class::Wizard,
            state: PlayerState::Idle,
            weapon: None,
                cooldown: 0,
                facing: Vector3::new(0.0,1.0,0.0),
            health: 7,
            charges: 0,
        })
        }
    }

    context.scenes[game.scene].graph[player_handle.clone()]
        .local_transform_mut()
        .set_position(Vector3::new(player_data.1[0], player_data.1[1], player_data.1[2]));
}