use std::fs;

use inquire::Select;
use reqwest::Url;
use rezvrh::{Bakalari, Which};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct Config {
    username: String,
    password: String,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let s = fs::read_to_string("config.json").expect("failed to load config.json");
    let conf = serde_json::from_str::<Config>(&s).expect("failed to parse config.json");
    let url = Url::parse("https://ssps.bakalari.cz").unwrap();
    let bakalari = Bakalari::from_creds((conf.username, conf.password), url).await?;
    bakalari.test().await?;
    /*
    let mut options = bakalari.get_classes();
    options.sort();
    let select = Select::new("Choose class", options).prompt()?;
    let class = bakalari.get_class(&select).unwrap();

    let table = bakalari.get_timetable(Which::Actual, &class).await?;
    println!("{table:#?}");
    Ok(())
     */

    
    let options = bakalari.get_classes();
    for option in options {
        let class = bakalari.get_class(&option).unwrap();
        for w in [Which::Actual, Which::Next, Which::Permanent] {
            bakalari.get_timetable(w, &class).await?;
        }
    } 

    let options = bakalari.get_teachers();
    for option in options {
        let teacher = bakalari.get_teacher(&option).unwrap();
        for w in [Which::Actual, Which::Next, Which::Permanent] {
            bakalari.get_timetable(w, &teacher).await?;
        }
    }

    let options = bakalari.get_rooms();
    for option in options {
        let room = bakalari.get_room(&option).unwrap();
        for w in [Which::Actual, Which::Next, Which::Permanent] {
            bakalari.get_timetable(w, &room).await?;
        }
    }
    Ok(())
}
