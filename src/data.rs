



pub struct Data {
    pub sets: Vec<Set>
}


pub struct Set {
    pub title: String,
    pub items: Vec<Item>
}

impl Set {
    pub fn new(title: String)->Self {
        Self {
            title,
            items: vec![]
        }
    }
}

pub struct Item {
    pub image_url: String
}
