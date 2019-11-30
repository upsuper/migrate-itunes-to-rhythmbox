use chrono::{DateTime, Utc};
use serde::Deserialize;
use std::collections::HashMap;

mod track_id;

use serde::de::IgnoredAny;
pub use track_id::TrackId;

#[derive(Debug, Deserialize)]
pub struct ItunesLibrary {
    #[serde(rename = "Tracks")]
    pub tracks: HashMap<TrackId, Track>,
    #[serde(rename = "Playlists")]
    pub playlists: Vec<Playlist>,
}

#[derive(Debug, Deserialize)]
pub struct Track {
    #[serde(rename = "Track ID")]
    pub id: TrackId,
    #[serde(rename = "Name")]
    pub name: String,
    #[serde(rename = "Artist")]
    pub artist: Option<String>,
    #[serde(rename = "Album")]
    pub album: Option<String>,
    #[serde(rename = "Genre")]
    pub genre: Option<String>,
    #[serde(rename = "Disc Number")]
    pub disc_number: Option<usize>,
    #[serde(rename = "Track Number")]
    pub track_number: Option<usize>,
    #[serde(rename = "Year")]
    pub year: Option<u16>,
    #[serde(rename = "Date Modified")]
    pub date_modified: DateTime<Utc>,
    #[serde(rename = "Date Added")]
    pub date_added: DateTime<Utc>,
    #[serde(rename = "Play Count")]
    pub play_count: Option<usize>,
    #[serde(rename = "Play Date UTC")]
    pub play_date: Option<DateTime<Utc>>,
    #[serde(rename = "Skip Count")]
    pub skip_count: Option<usize>,
    #[serde(rename = "Skip Date")]
    pub skip_date: Option<DateTime<Utc>>,
    #[serde(rename = "Rating")]
    pub rating: Option<u8>,
    #[serde(rename = "Movie", default)]
    pub movie: bool,
    #[serde(rename = "Location")]
    pub location: String,
}

#[derive(Debug, Deserialize)]
pub struct Playlist {
    #[serde(rename = "Name")]
    pub name: String,
    #[serde(rename = "Playlist ID")]
    pub id: u32,
    #[serde(rename = "Smart Info")]
    pub smart_info: Option<IgnoredAny>,
    #[serde(rename = "Playlist Items", default)]
    pub items: Vec<PlaylistItem>,
}

#[derive(Debug, Deserialize)]
pub struct PlaylistItem {
    #[serde(rename = "Track ID")]
    pub id: TrackId,
}
