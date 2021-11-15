use anyhow::Error;
use glium::backend::glutin::glutin::event_loop::EventLoopProxy;
use serde_json::Value;

use crate::cache::{cache_set, create_cacher};
use crate::Call;

lazy_static! {
    pub static ref HOME: &'static str = "https://cd-static.bamgrid.com/dp-117731241344/home.json";
}

pub async fn fetch(event_loop_proxy: EventLoopProxy<Call> ) -> Result<(),Error> {

    let cache_proxy = event_loop_proxy.clone();
    let cacher = create_cacher(cache_proxy).await;
    let response = reqwest::get(HOME.to_string() ).await?;
    let json: Value = serde_json::from_str( response.text().await?.as_str() )?;

    if let Value::Array(containers) = &json["data"]["StandardCollection"]["containers"] {
        for container in containers {
            if let Value::String(title) = &container["set"]["text"]["title"]["full"]["set"]["default"]["content"]
            {
                if let Value::Array(items) = container["set"]["items"].clone(){
                    let mut set = Set::new(title.clone());
                    set.items = parse_items(items).await;
                    event_loop_proxy.send_event(Call::AddSet(set.clone()));
                    cache_set(set,cacher.clone() );
                } else {
                    if let Value::String(ref_id) = container["set"]["refId"].clone() {
                        let set = get_set(ref_id,title.clone()).await?;
                        event_loop_proxy.send_event(Call::AddSet(set.clone()));
                        cache_set(set,cacher.clone());
                    }
                    else {
                        return Err(anyhow!("could not find refId for set"));
                    }
                }
            }
        }
    }
    Ok(())
}

async fn get_set( ref_id: String, title: String ) -> Result<Set,Error> {
    let url = format!("https://cd-static.bamgrid.com/dp-117731241344/sets/{}.json",ref_id);
    let response = reqwest::get(url.to_string() ).await?;
    let response = response.text().await?;


    let json: Value = serde_json::from_str( response.as_str() )?;
    let mut set = json["data"]["CuratedSet"].clone();

    if Value::Null == set {
        set = json["data"]["TrendingSet"].clone();
    }

    if Value::Null == set {
        set = json["data"]["PersonalizedCuratedSet"].clone();
    }


    if Value::Null == set {
        println!("{}",response);
    }

    if let Value::Array(items) = set["items"].clone(){
        let mut set = Set::new(title);
        set.items = parse_items(items).await;
        return Ok(set);
    }else {
        println!("coulud not find items...");
        Err(anyhow!("could not find 'items'"))
    }
}

async fn parse_items(items: Vec<Value>) -> Vec<Item> {
    let mut rtn = vec![];
    for item in &items {
        // not my most elegant code, but i'm a bit surprised by the unexpected variety of image types (series,program,default)
        if let Value::String(image_url) = item["image"]["tile"]["1.78"]["series"]["default"]["url"].clone() {
            rtn.push(Item {
                image_url
            });
        } else  if let Value::String(image_url) = item["image"]["tile"]["1.78"]["program"]["default"]["url"].clone() {
            rtn.push(Item {
                image_url
            });
        } else  if let Value::String(image_url) = item["image"]["tile"]["1.78"]["default"]["default"]["url"].clone() {
            rtn.push(Item {
                image_url
            });
        } else {
            println!("could not find image_url for Item...");
        }
    }
    rtn
}


pub struct Data {
    pub sets: Vec<Set>
}

impl Data{
    pub fn new()->Self {
        Self {
            sets: vec![]
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

