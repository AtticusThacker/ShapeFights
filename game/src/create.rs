//this module contains various functions to streamline creation of fyrox objects.
use crate::*;

// the functions fyrox gives us to create text were not great so i made my own
// create text with a background (like highlighted)
// take in floating point numbers as parameters for position
// default visibility is set to false!!
pub fn create_text_with_background(ui: &mut UserInterface, text: &str, x: f32, y: f32, color: Color) -> Handle<UiNode> {
    let text_widget =
        TextBuilder::new(WidgetBuilder::new().with_foreground(Brush::Solid(Color::BLACK)))
            .with_text(text)
            .build(&mut ui.build_ctx());
    BorderBuilder::new(
        WidgetBuilder::new().with_desired_position(Vector2::new(x, y))
            .with_child(text_widget) // <-- Text is now a child of the border
            .with_background(Brush::Solid(color)) // pink
            .with_visibility(false),
    )
    .build(&mut ui.build_ctx());
    return text_widget;
}

pub fn create_cube_rigid_body(graph: &mut Graph) -> Handle<Node> {
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

pub fn create_kinematic_rigid_body(graph: &mut Graph) -> Handle<Node> {
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

pub fn create_rect(graph: &mut Graph, resource_manager: &ResourceManager, color: &Vec<u8>, shape: String) -> Handle<Node> {
    RectangleBuilder::new(
        BaseBuilder::new().with_local_transform(
            TransformBuilder::new()
                // Size of the rectangle is defined only by scale.
                .with_local_scale(Vector3::new(0.4, 0.4, 0.4))
                .build(),
        ),
    )
    .with_texture(resource_manager.request::<Texture, _>(shape))
    .with_color(Color{r: color[0], g: color[1], b: color[2], a: 255})
    .build(graph)
}

pub fn set_script<T: ScriptTrait>(node: &mut Node, script: T) {
    node.set_script(Some(Script::new(script)))
}

pub fn create_text(ui: &mut UserInterface, text: &str) -> Handle<UiNode> {
    TextBuilder::new(WidgetBuilder::new())
        .with_text(text)
        .build(&mut ui.build_ctx())
}

pub fn create_centered_text(ui: &mut UserInterface, text: &str) -> Handle<UiNode> {
    TextBuilder::new(WidgetBuilder::new())
        .with_horizontal_text_alignment(HorizontalAlignment::Center)
        .with_vertical_text_alignment(VerticalAlignment::Center)
    .with_text(text)
    .build(&mut ui.build_ctx())
}

pub fn create_weapon_body(class: &Class, context: &mut PluginContext, game: &Game) -> Handle<Node> {
    //setting up melee weapon
    let weapontype = match class {
        Class::Barbarian => Class::BARBWEP,
        Class::Rogue => Class::ROGWEP,
        Class::Wizard => Class::WIZWEP,
        Class::Fighter => Class::FIGWEP,

    };
    RigidBodyBuilder::new(BaseBuilder::new().with_children(&[
        RectangleBuilder::new(
            BaseBuilder::new().with_local_transform(
                TransformBuilder::new()
                    // Size of the rectangle is defined only by scale.
                    .with_local_scale(Vector3::new(weapontype.half_extents[0].clone()*2.0, weapontype.half_extents[1].clone()*2.0,1.0))
                    .build()
            )
        )
            .with_texture(context.resource_manager.request::<Texture, _>("data/white_rectangle.png"))
            .build(&mut context.scenes[game.scene].graph),
        // Rigid body must have at least one collider
        ColliderBuilder::new(BaseBuilder::new())
            .with_shape(ColliderShape::Cuboid(weapontype))
            .with_sensor(true)
            .build(&mut context.scenes[game.scene].graph),
        
        ]))
    .with_body_type(RigidBodyType::KinematicPositionBased)
    .with_ccd_enabled(true)
    .build(&mut context.scenes[game.scene].graph)

    

}

//create and position a new player object
pub fn create_player(player_num: i8, player_class: Class, id: GamepadId, context: &mut PluginContext, game: &mut Game) {
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
    //make a weapon rigid body / collider
    let weapon_handle = create_weapon_body(&player_class, context, &game);
    //make the weapon a child of the player
    context.scenes[game.scene].graph.link_nodes(weapon_handle, player_handle);
    //add a weapon script to the weapon
    set_script(&mut context.scenes[game.scene].graph[weapon_handle.clone()],
        Weapon{
            player: player_handle.clone(),
            class: player_class.clone(),
        }
    );
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
                weapon: weapon_handle,
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
                weapon: weapon_handle,
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
            weapon: weapon_handle,
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
            weapon: weapon_handle,
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

///create a new button with position x,y, dimensions w,h, text, and optional alignment.
/// used primarily to create the class selection menu
pub fn create_player_class_button(
    ui: &mut UserInterface, x: f32, y: f32, w:f32, h: f32, text: &str, 
    valign: Option<VerticalAlignment>, halign: Option<HorizontalAlignment>
) -> Handle<UiNode> {
    //create the text widget; we do this first bc of optional alignment stuff
    let mut b = TextBuilder::new(WidgetBuilder::new()).with_text(text);
    if valign.is_some() {
        b = b.with_vertical_text_alignment(valign.unwrap());
    }
    if halign.is_some() {
        b = b.with_horizontal_text_alignment(halign.unwrap());
    }
    let textwidget = b.build(&mut ui.build_ctx());

    ButtonBuilder::new(
        WidgetBuilder::new()
        .with_desired_position(Vector2::new(x, y))
        .with_width(w)
        .with_height(h),
    )
    .with_content(textwidget)
    .build(&mut ui.build_ctx())

}