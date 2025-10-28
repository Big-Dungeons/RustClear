use uuid::Uuid;

#[derive(Debug)]
pub struct ProfileId {
    uuid: Uuid
}

impl ProfileId {
    pub fn new(uuid: Uuid) -> Self {
        Self { uuid }
    }
    
    pub fn get_id(&self) -> Uuid {
        self.uuid
    }
}