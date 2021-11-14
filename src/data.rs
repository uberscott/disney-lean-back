use dashmap::DashSet;

pub struct Data {
    pub sets: DashSet<Set>
}

impl Data{
    pub fn new()->Self {
        Self {
            sets: DashSet::new()
        }
    }
}

#[derive(Clone,Hash,Eq,PartialEq,Ord,PartialOrd)]
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

#[derive(Clone,Hash,Eq,PartialEq,Ord,PartialOrd)]
pub struct Item {
    pub image_url: String
}
