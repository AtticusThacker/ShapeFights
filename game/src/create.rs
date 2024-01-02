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