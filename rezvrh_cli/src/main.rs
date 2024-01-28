use clap::Parser;
use inquire::Select;
use reqwest::Url;
use rezvrh_scraper::{Bakalari, Type, Which};
use serde::{Deserialize, Serialize};
use tokio::fs;

#[derive(Serialize, Deserialize)]
pub struct Config {
    url: String,
    username: Option<String>,
    password: Option<String>,
}

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Path to config file
    #[arg(short, long)]
    config: String,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args = Args::parse();

    let s = fs::read_to_string(args.config)
        .await?;
    let conf = serde_json::from_str::<Config>(&s)?;
    let url = Url::parse(&conf.url)?;
    let bakalari = if let (Some(username), Some(password)) = (conf.username, conf.password) {
        Bakalari::from_creds((username, password), url).await?
    } else {
        Bakalari::no_auth(url).await?
    };
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
    }.unwrap();

    let table = bakalari.get_timetable(which, &selection).await?;

    fs::write("timetable.json", serde_json::to_string_pretty(&table)?).await?;

    Ok(())
}
