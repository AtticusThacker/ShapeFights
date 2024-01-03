//This module handles the player's weapon
use crate::*;
use fyrox::scene::transform::Transform;

#[derive(Visit, Reflect, Default, Debug, Clone)]
pub struct Weapon {
    pub player: Handle<Node>,
    pub class: Class,
}

impl_component_provider!(Weapon);

impl TypeUuidProvider for Weapon {
    fn type_uuid() -> Uuid {
        uuid!("bf0f9804-56cb-4a2e-beba-93d75371a568")
    }
}

impl ScriptTrait for Weapon {
    fn on_init(&mut self, context: &mut ScriptContext) {
        // Put initialization logic here.

    }
    
    fn on_start(&mut self, context: &mut ScriptContext) {
        // subscribe to messages
        context.message_dispatcher.subscribe_to::<Message>(context.handle);

        //setup the correct positioning and visibility of the weapon:
        
        let offset = match self.class {
            Class::Barbarian => 1.0,
            Class::Fighter => 1.0,
            _ => 0.75,
        };

        context.scene.graph[context.handle.clone()].set_visibility(false);
        //change the local position of the weapon
        if let Some(weapon) = context.scene.graph[context.handle.clone()].cast_mut::<RigidBody>() {
            let axis = Vector3::z_axis();
            //the transform encodes essentially all position information
            let mut starting_transform = Transform::identity();
            //first, change its rotation angle to pi/4 radians (45 degrees)
            starting_transform.set_rotation(UnitQuaternion::from_axis_angle(&axis, -(std::f32::consts::FRAC_PI_2)))
                //these should always be negatives of each other in x and y coords.
                //this sets the position relative to the player
                .set_position(Vector3::new(0.0, offset,0.0))
                //this sets the position of the rotation pivot (the thing it rotates around) to the center of the player
                .set_rotation_pivot(Vector3::new(0.0,-offset,0.0));
            
            weapon.set_local_transform(starting_transform);
        }
    }

    fn on_os_event(&mut self, event: &Event<()>, context: &mut ScriptContext) {
        // Respond to OS events here.
    }

    fn on_update(&mut self, context: &mut ScriptContext) {
        // Put object logic here.
    }

    fn id(&self) -> Uuid {
        Self::type_uuid()
    }
}

impl Weapon {


}

//todo: