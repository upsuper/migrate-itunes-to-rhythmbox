use crate::itunes_library::Track;
use std::fmt;

#[derive(Clone, Copy, Eq, Hash, PartialEq)]
pub struct TrackKey<'a> {
    pub name: &'a str,
    pub artist: Option<&'a str>,
    pub album: Option<&'a str>,
    pub disc_number: Option<usize>,
    pub track_number: Option<usize>,
}

impl<'a> From<&'a Track> for TrackKey<'a> {
    fn from(track: &'a Track) -> Self {
        TrackKey {
            name: &track.name,
            artist: track.artist.as_ref().map(String::as_ref),
            album: track.album.as_ref().map(String::as_str),
            disc_number: track.disc_number,
            track_number: track.track_number,
        }
    }
}

impl<'a> fmt::Display for TrackKey<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
        write!(
            f,
            "{} / {} / {}",
            self.name,
            self.artist.unwrap_or(""),
            self.album.unwrap_or(""),
        )
    }
}
