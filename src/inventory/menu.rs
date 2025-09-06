use crate::player::player::PlayerExtension;

pub trait Menu<P : PlayerExtension> {
    
    fn container_name(&self) -> usize;
    
}


pub enum DungeonMenu {
    Mort,
}

impl<P : PlayerExtension> Menu<P> for DungeonMenu {
    
    fn container_name(&self) -> usize {
        todo!()
    }
    
    
    
}