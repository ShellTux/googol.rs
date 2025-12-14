use crate::debugv;
use log::{debug, error, info};
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap, error::Error, fmt, fs::File, io, path::Path, sync::Arc, time::SystemTime,
};
use tokio::sync::RwLock;
use url::Url;

static TOP_STORIES_N: usize = 10;
static HACKER_NEWS_DB_FILE: &'static str = "state/hn_db.json";

#[derive(Debug, Deserialize)]
pub struct HackerNewsBody {
    pub words: Vec<String>,
}

#[derive(Debug, Deserialize)]
pub struct HackerNewsQuery {
    pub query: String,
}

#[derive(Debug, Serialize, Deserialize, Eq, Hash, PartialEq, Clone, Copy)]
pub struct HackerNewsId(usize);

impl fmt::Display for HackerNewsId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl HackerNewsId {
    pub async fn request_story(&self) -> Result<HackerNewsStory, Box<dyn Error>> {
        let url: Url = format!("https://hacker-news.firebaseio.com/v0/item/{}.json", self)
            .parse()
            .unwrap();
        debugv!(&url, display);

        Ok(reqwest::get(url).await?.json().await?)
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct HackerNewsStory {
    pub by: String,
    pub descendants: Option<usize>,
    pub id: HackerNewsId,
    pub kids: Option<Vec<HackerNewsId>>,
    pub score: usize,
    pub time: usize,
    pub title: Option<String>,
    pub r#type: String,
    pub url: Option<Url>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct HackerNewsDBGuard {
    pub db: HashMap<HackerNewsId, HackerNewsStory>,
    last_fetched_top_stories: Option<SystemTime>,
}

impl HackerNewsDBGuard {
    pub fn new() -> Self {
        Self {
            db: HashMap::new(),
            last_fetched_top_stories: None,
        }
    }

    pub fn load() -> Result<Self, Box<dyn Error>> {
        Self::load_path(HACKER_NEWS_DB_FILE)
    }

    pub fn load_path<P: AsRef<Path>>(path: P) -> Result<Self, Box<dyn Error>> {
        Ok(match File::open(&path) {
            Err(e) => {
                error!("Error `load` {}: {}", path.as_ref().display(), e);
                Self::new()
            }
            Ok(file) => match serde_json::from_reader(file) {
                Err(e) => {
                    error!("Error `load` {}: {}", path.as_ref().display(), e);
                    Self::new()
                }
                Ok(db) => {
                    info!("Loaded: {}", path.as_ref().display());
                    db
                }
            },
        })
    }

    pub fn save(&self) -> io::Result<()> {
        self.save_path(HACKER_NEWS_DB_FILE)
    }

    pub fn save_path<P: AsRef<Path>>(&self, path: P) -> io::Result<()> {
        let file = File::create(&path)?;
        debug!("writting to {}: {:?}", path.as_ref().display(), self);
        serde_json::to_writer(file, self)?;
        Ok(())
    }

    pub async fn fetch_top_stories_if_expired(&mut self) -> Result<(), Box<dyn Error>> {
        let now = SystemTime::now();

        static DAY_SECS: u64 = 1 * 24 * 60 * 60;

        if self.last_fetched_top_stories.map_or(true, |last| {
            now.duration_since(last)
                .map_or(true, |d| d.as_secs() >= DAY_SECS)
        }) {
            self.db.clear();

            let url = "https://hacker-news.firebaseio.com/v0/topstories.json";
            let top_stories_ids: Vec<HackerNewsId> = reqwest::get(url)
                .await?
                .json::<Vec<HackerNewsId>>()
                .await?
                .into_iter()
                .take(TOP_STORIES_N)
                .collect();

            for id in &top_stories_ids {
                self.db.insert(*id, id.request_story().await?);
            }

            self.last_fetched_top_stories = Some(now);
            info!("Fetched top stories.");
        }

        Ok(())
    }

    pub fn search(&self, keywords: &[String]) -> Vec<HackerNewsStory> {
        let keywords: Vec<String> = keywords.into_iter().map(|w| w.to_lowercase()).collect();

        let mut results = Vec::new();

        for (_, story) in &self.db {
            if let Some(ref title) = story.title {
                if keywords
                    .iter()
                    .any(|keyword| title.to_lowercase().contains(keyword))
                {
                    results.push(story.clone());
                }
            }
        }

        results
    }

    pub fn new_safe(db: Self) -> HackerNewsDB {
        Arc::new(RwLock::new(db))
    }
}

pub type HackerNewsDB = Arc<RwLock<HackerNewsDBGuard>>;
