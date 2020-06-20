use reqwest::blocking::{get, ClientBuilder};
use serde_derive::{Deserialize, Serialize};
use std::{
    convert::{TryFrom, TryInto},
    env, fmt,
    fs::File,
    io::Write,
    time::Duration,
};
use url::Url;

use crate::{
    cli::{Args, OutFormat},
    BASE_URL,
};

pub struct XkcdClient {
    args: Args,
}

impl XkcdClient {
    pub fn new(args: Args) -> Self {
        XkcdClient { args }
    }

    pub fn run(&self) -> anyhow::Result<()> {
        let url = if let Some(n) = self.args.num {
            format!("{}/{}/info.0.json", BASE_URL, n)
        } else {
            format!("{}/info.0.json", BASE_URL)
        };
        let http_client = ClientBuilder::new()
            .timeout(Duration::from_secs(self.args.timeout))
            .build()?;
        let response: ComicResponse = http_client.get(&url).send()?.text()?.try_into()?;
        let comic: Comic = response.into();

        if self.args.save {
            comic.save()?;
        }
        comic.print(self.args.output)?;
        Ok(())
    }
}

#[derive(Serialize)]
pub struct Comic {
    title: String,
    num: usize,
    date: String,
    desc: String,
    img_url: String,
}

impl Comic {
    pub fn save(&self) -> anyhow::Result<()> {
        let url = Url::parse(self.img_url.as_str())?;
        let dir = env::current_dir()?;
        let image_name = url.path_segments().unwrap().last().unwrap();
        let path = dir.join(image_name);
        let mut file = File::create(path)?;
        let body = get(&self.img_url)?;
        file.write_all(&body.bytes()?).map_err(|e| e.into())
    }

    pub fn print(&self, of: OutFormat) -> anyhow::Result<()> {
        match of {
            OutFormat::Json => println!("{}", serde_json::to_string(self)?),
            OutFormat::Text => println!("{}", self),
        }
        Ok(())
    }
}

impl fmt::Display for Comic {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Title: {}\n\
               Comic No: {}\n\
               Date: {}\n\
               Description: {}\n\
               Image: {}\n",
            self.title, self.num, self.date, self.desc, self.img_url
        )
    }
}

impl From<ComicResponse> for Comic {
    fn from(cr: ComicResponse) -> Self {
        Comic {
            title: cr.title,
            num: cr.num,
            date: format!("{}-{}-{}", cr.day, cr.month, cr.year),
            desc: cr.alt,
            img_url: cr.img,
        }
    }
}

#[allow(dead_code)]
#[derive(Deserialize)]
pub struct ComicResponse {
    month: String,
    num: usize,
    link: String,
    year: String,
    news: String,
    safe_title: String,
    transcript: String,
    alt: String,
    img: String,
    title: String,
    day: String,
}

impl TryFrom<String> for ComicResponse {
    type Error = anyhow::Error;

    fn try_from(json: String) -> Result<Self, Self::Error> {
        serde_json::from_str(&json).map_err(|e| e.into())
    }
}
