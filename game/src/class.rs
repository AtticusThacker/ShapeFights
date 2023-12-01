use crate::{Visit, Reflect, Visitor, VisitResult, FieldInfo};

#[derive(Visit, Reflect, Debug, Clone, Default)]
pub enum Class {
    Barbarian,
    Rogue,
    Wizard,
    #[default]
    Fighter,
}

fn main() {
    println!("hi");
}
