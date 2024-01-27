use std::fs;

use inquire::Select;
use reqwest::Url;
use rezvrh::{Bakalari, TimetableWhich};
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
    let mut bakalari = Bakalari::from_creds((conf.username, conf.password), url).await?;
    bakalari.test().await?;
    let options = bakalari.get_classes().await?;
    let mut choices = options.left_values().collect::<Vec<_>>();
    choices.sort();
    let select = Select::new("Choose class", choices).prompt()?;
    let class = options.get_by_left(select).unwrap();
    let table = bakalari.get_class(class, TimetableWhich::Actual).await?;
    println!("{:#?}", table);
    Ok(())
}
