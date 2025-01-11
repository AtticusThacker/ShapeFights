//This module contains the data structure + implementation for the Projectile script
use crate::*;

#[derive(Visit, Reflect, Debug, Clone, Default)]

pub struct Projectile {
    pub facing: Vector3<f32>,
    pub hit: bool,
    pub life: u32,
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

    fn on_message(&mut self, message: &mut dyn ScriptMessagePayload, ctx: &mut ScriptMessageContext) {
        if let Some(message) = message.downcast_ref::<Message>(){
            match message {
                Message::Attack{s} if !*s => {self.hit = true;},
                _ => (),
            }
        }
    }

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
        if let Some((collider_handle, colnode)) = ctx.scene.graph.find(ctx.handle.clone(), &mut |c| c.is_collider2d()) {
            let collider = colnode.as_collider2d();
            // iterate over collisions
            for i in collider.intersects(&ctx.scene.graph.physics2d) {
                if i.has_any_active_contact{
                    //I think a very persistent bug in a previous version of this code arose from 
                    //sending the hit message to the wrong side of the interaction; I'm still
                    //trying to figure out how these intersection pairs work.
                    let other_collider_parent = if i.collider1 == collider_handle {
                        ctx.scene.graph[i.collider2].parent()
                    } else {
                        ctx.scene.graph[i.collider1].parent()
                    };
                    //probably should figure out how to get projectiles not to hit you 
                    // if other_collider_parent == self.player {
                    //     //stop hitting yourself
                    //     return;
                    // }
                    let parent_node = &ctx.scene.graph[other_collider_parent.clone()];

                    let mut knockvec = Vector3::new(1.0,1.0, 1.0);
                    //get the knockback vector
                    knockvec.set_magnitude(3.0);


                    ctx.message_sender.send_to_target(other_collider_parent,
                        Message::Hit{
                            damage: 3,
                            knockback: knockvec,
                            sender: ctx.handle,
                        }
                    );
                    self.hit = true;
                }
            }
            //     //for each active contact
            //     if i.has_any_active_contact {
            //         //find its parent
            //         if let Some((target, _t)) = ctx.scene.graph.find_up(i.collider1, &mut |c| c.try_get_script::<Player>().is_some()) {
            //             let mut knockvec = self.facing.clone();
            //             knockvec.set_magnitude(3.0);
            //             ctx.message_sender.send_to_target(target, Message::Hit{
            //                 damage: 3, 
            //                 knockback: knockvec,
            //                 sender: ctx.handle.clone()
            //             });
            //         }
            //         self.hit = true;   
            //     }
            // }
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
