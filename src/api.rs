use std::time::Duration;

use feed_rs::{model, parser};

const DEFAULT_USER_AGENT: &str =  "Mozilla/5.0 (compatible; FrostFeed/1.0; +https://github.com/marciosobel/frostfeed)";

#[derive(Debug, Clone)]
pub struct Feed {
    pub url: String,
    pub title: String,
    pub description: Option<String>,
    pub items: Vec<FeedItem>
}

#[derive(Debug, Clone)]
pub struct FeedItem {
    pub title: String,
    pub url: Option<String>,
    pub description: Option<String>
}

pub async fn fetch_rss(url: String) -> Feed {
    Feed::from_url(url).await
}

impl Feed {
    pub async fn from_url(url: String) -> Self {
        let ua = DEFAULT_USER_AGENT;
        let client = reqwest::Client::builder()
            .redirect(reqwest::redirect::Policy::limited(10))
            .timeout(Duration::from_secs(10))
            .build().unwrap();

        let response = client
            .get(url.clone())
            .header("User-Agent", ua)
            .header(
                "Accept",
                "application/rss+xml, application/atom+xml, application/xml, text/xml, */*",
            )
            .header("Accept-Encoding", "gzip, deflate")
            .header("Accept-Language", "en-US,en;q=0.9")
            .header("Cache-Control", "no-cache")
            .header("Connection", "keep-alive")
            .send().await.unwrap();

        let final_url = response.url().clone();
        let status = response.status();
        // let content_type = response
        //     .headers()
        //     .get("content-type")
        //     .and_then(|ct| ct.to_str().ok())
        //     .unwrap_or("unknown")
        //     .to_lowercase();

        if !status.is_success() {
            panic!( "HTTP error {}: Failed to fetch feed from {}", status, url);
        }

        let content = response.bytes().await.unwrap();

         if content.len() < 100 {
            panic!(
                "Response too short ({} bytes), might be empty or an error page",
                content.len()
            );
        }

        let content_start = String::from_utf8_lossy(&content[..std::cmp::min(200, content.len())]);
        if content_start.trim_start().starts_with("<!DOCTYPE html")
            || content_start.trim_start().starts_with("<html")
        {
            panic!(
                "Received HTML page instead of RSS/Atom feed. URL might be incorrect or require authentication. Final URL: {}",
                final_url
            );
        }

        let feed = parser::parse(&content[..]).unwrap();
        
        let title = feed.title.map(|t| t.content).unwrap_or("Untitled feed".to_string());
        let items = feed.entries.into_iter().map(FeedItem::from_entry).collect();
        let description = feed.description.map(|d| d.content);

        Self {
            url: final_url.to_string(),
            title,
            items,
            description
        }
    }
}

impl FeedItem {
    fn from_entry(entry: model::Entry) -> Self {
        dbg!("{:?}", &entry);
        let title=entry
                .title
                .as_ref()
                .map(|t| t.content.clone())
                .unwrap_or_else(|| "Untitled".to_string());

            let url =entry.links.first().map(|link| link.href.clone());

            let description = if let Some(content) = entry.content.as_ref() {
            Some(content.body.clone().unwrap_or_default())
        } else {
            entry
                .summary
                .as_ref()
                .map(|summary| summary.content.clone())
        };

            Self {
                title,
                url,
                description,
            }
    }
}

pub async fn download_image(uri: String) -> iced::widget::image::Handle {
    let response = reqwest::get(uri).await.unwrap();
    let bytes = response.bytes().await.unwrap();
    iced::widget::image::Handle::from_bytes(bytes)
}
