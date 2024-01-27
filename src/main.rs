use tokio::fs;
use inquire::Select;
use reqwest::Url;
use rezvrh::{Bakalari, Type, Which};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct Config {
    username: String,
    password: String,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let s = fs::read_to_string("config.json").await.expect("failed to load config.json");
    let conf = serde_json::from_str::<Config>(&s).expect("failed to parse config.json");
    let url = Url::parse("https://ssps.bakalari.cz").unwrap();
    let bakalari = Bakalari::from_creds((conf.username, conf.password), url).await?;
    bakalari.test().await?;

    let typ = Select::new("Choose type", vec![Type::Teacher, Type::Class, Type::Room]).prompt()?;
    let which = Select::new(
        "Choose which",
        vec![Which::Permanent, Which::Actual, Which::Next],
    )
    .prompt()?;

    let mut options = match typ {
        Type::Teacher => bakalari.get_teachers(),
        Type::Class => bakalari.get_classes(),
        Type::Room => bakalari.get_rooms(),
    };
    options.sort();
    let select = Select::new("Choose object", options).prompt()?;
    let selection = match typ {
        Type::Teacher => bakalari.get_teacher(&select),
        Type::Class => bakalari.get_class(&select),
        Type::Room => bakalari.get_room(&select),
    }
    .unwrap();

    let table = bakalari.get_timetable(which, &selection).await?;

    fs::write("timetable.json", serde_json::to_string_pretty(&table)?).await?;

    Ok(())
}
