use crate::itunes_library::{ItunesLibrary, Track, TrackId};
use crate::track_key::TrackKey;
use anyhow::{anyhow, ensure, Context, Result};
use by_address::ByAddress;
use clap::Parser;
use elementtree::{Element, QName, WriteOptions, XmlProlog};
use log::{info, warn};
use std::collections::{HashMap, HashSet};
use std::fs;
use std::fs::File;
use std::io::{BufReader, BufWriter};
use std::path::{Path, PathBuf};
use stderrlog::Timestamp;

mod itunes_library;
mod track_key;

#[derive(Debug, Parser)]
struct Opt {
    /// Path to the iTunes Library XML file
    #[arg(name = "iTunes Library file")]
    itunes_library: PathBuf,
    /// Path to the Rhythmbox path
    ///
    /// When not specified,
    /// it is `$XDG_DATA_HOME/rhythmbox` or `$HOME/.local/share/rhythmbox` by default.
    #[arg(name = "Rhythmbox path", short, long)]
    rhythmbox_path: Option<PathBuf>,
    /// Silence all output
    #[arg(short, long)]
    quiet: bool,
}

fn main() -> Result<()> {
    let opt = Opt::parse();
    stderrlog::new()
        .module(module_path!())
        // 2 for info level
        .verbosity(2)
        .quiet(opt.quiet)
        .timestamp(Timestamp::Off)
        .init()?;

    // Determine path of Rhythmbox data directory.
    let rhythmbox_path = match opt.rhythmbox_path {
        Some(path) => path,
        None => {
            let mut path = dirs::data_dir().ok_or_else(|| {
                anyhow!("No data dir available, please specify path to Rhythmbox data dir")
            })?;
            path.push("rhythmbox");
            path
        }
    };
    info!("Rhythmbox path: {}", rhythmbox_path.display());

    info!("Reading iTunes library...");
    let mut itunes_library: ItunesLibrary =
        plist::from_file(&opt.itunes_library).context("failed to read iTunes library")?;
    // Strip movies from the library.
    itunes_library.tracks.retain(|_, track| !track.movie);
    let itunes_track_map = itunes_library
        .tracks
        .values()
        .map(|track| {
            let key = TrackKey::from(track);
            (key, track)
        })
        .collect::<HashMap<_, _>>();
    ensure!(
        itunes_track_map.len() == itunes_library.tracks.len(),
        "duplicate song in iTunes library"
    );

    let (rhythmdb_path, playlists_path) =
        backup_rhythmbox_files(&rhythmbox_path).context("failed to backup Rhythmbox files")?;

    let track_locations = sync_to_database(&rhythmdb_path, &itunes_track_map)
        .context("failed to synchronize to Rhythmbox database")?;

    migrate_playlists(&playlists_path, &itunes_library, &track_locations)
        .context("failed to migrate playlists")?;

    Ok(())
}

fn backup_rhythmbox_files(rhythmbox_path: &Path) -> Result<(PathBuf, PathBuf)> {
    info!("Backing up existing Rhythmbox files...");
    const RHYTHMDB_FILENAME: &str = "rhythmdb.xml";
    const RHYTHMDB_BACKUP_FILENAME: &str = "rhythmdb.xml.bak";
    let rhythmdb_path = rhythmbox_path.join(RHYTHMDB_FILENAME);
    let rhythmdb_bak = rhythmbox_path.join(RHYTHMDB_BACKUP_FILENAME);
    ensure!(
        !rhythmdb_bak.exists(),
        "backup of database already exists: {}",
        rhythmdb_bak.display(),
    );
    fs::copy(&rhythmdb_path, &rhythmdb_bak)?;
    const PLAYLISTS_FILENAME: &str = "playlists.xml";
    const PLAYLISTS_BACKUP_FILENAME: &str = "playlists.xml.bak";
    let playlists_path = rhythmbox_path.join(PLAYLISTS_FILENAME);
    let playlists_bak = rhythmbox_path.join(PLAYLISTS_BACKUP_FILENAME);
    ensure!(
        !playlists_bak.exists(),
        "backup of playlists already exists: {}",
        playlists_bak.display(),
    );
    fs::copy(&playlists_path, &playlists_bak)?;
    Ok((rhythmdb_path, playlists_path))
}

fn sync_to_database(
    rhythmdb_path: &Path,
    itunes_track_map: &HashMap<TrackKey<'_>, &Track>,
) -> Result<HashMap<TrackId, String>> {
    info!("Reading Rhythmbox database...");
    let rhythmdb = File::open(rhythmdb_path).context("failed to open database file")?;
    let mut rhythmdb =
        Element::from_reader(BufReader::new(rhythmdb)).context("failed to read database")?;
    ensure!(
        rhythmdb.tag() == &QName::from("rhythmdb"),
        "unknown database format",
    );
    ensure!(
        rhythmdb.get_attr("version") == Some("2.0"),
        "unknown database version",
    );

    info!("Synchronizing to Rhythmbox database...");
    let mut unused_itunes_tracks = itunes_track_map
        .values()
        .copied()
        .map(ByAddress)
        .collect::<HashSet<_>>();
    let mut track_locations = HashMap::with_capacity(itunes_track_map.len());
    for entry in rhythmdb.children_mut() {
        ensure!(
            entry.tag() == &QName::from("entry"),
            "unknown entry element in database"
        );
        if entry.get_attr("type") != Some("song") {
            continue;
        }
        // Read the metadata of the entry.
        let child_text = |tag: &'static str| entry.find(tag).map(Element::text);
        let name = child_text("title").expect("song without name");
        let artist = child_text("artist");
        let album = child_text("album");
        let disc_number = child_text("disc-number").map(str::parse).transpose()?;
        let track_number = child_text("track-number").map(str::parse).transpose()?;
        let location = child_text("location")
            .expect("song without location")
            .to_owned();
        // Fixup known "unknown" artist.
        let artist = match artist {
            Some("未知") => None,
            artist => artist,
        };
        let key = TrackKey {
            name,
            artist,
            album,
            disc_number,
            track_number,
        };

        let track = match itunes_track_map.get(&key) {
            Some(track) => {
                unused_itunes_tracks.remove(&ByAddress(*track));
                track_locations.insert(track.id, location);
                *track
            }
            None => {
                warn!("song {} not found", key);
                continue;
            }
        };
        // Create a new key from the iTunes track,
        // so that we stop holding sharable borrows to the entry.
        let key = TrackKey::from(track);

        let mut update_or_append_child = |tag: &'static str, text: String| match entry.find_mut(tag)
        {
            Some(element) => {
                if tag != "first-seen" {
                    warn!("overriding {} of {}: {}", tag, key, element.text());
                }
                element.set_text(text);
            }
            None => {
                let indentation = entry.text().to_string();
                let last_element = entry.get_child_mut(entry.child_count() - 1).unwrap();
                let mut element = Element::new(tag);
                element.set_text(text);
                element.set_tail(last_element.tail());
                last_element.set_tail(indentation);
                entry.append_child(element);
            }
        };
        update_or_append_child("first-seen", track.date_added.timestamp().to_string());
        if let Some(play_date) = track.play_date {
            update_or_append_child("last-played", play_date.timestamp().to_string());
        }
        if let Some(play_count) = track.play_count {
            if play_count > 0 {
                update_or_append_child("play-count", play_count.to_string());
            }
        }
    }
    for track in unused_itunes_tracks {
        warn!("song {} unused", TrackKey::from(*track));
    }

    info!("Saving the change to Rhythmbox database...");
    let rhythmdb_file = File::create(rhythmdb_path).context("failed to open database to update")?;
    let options = WriteOptions::new().set_xml_prolog(Some(XmlProlog::Version10));
    rhythmdb
        .to_writer_with_options(BufWriter::new(rhythmdb_file), options)
        .context("failed to update database")?;

    Ok(track_locations)
}

fn migrate_playlists(
    playlists_path: &Path,
    itunes_library: &ItunesLibrary,
    track_locations: &HashMap<TrackId, String>,
) -> Result<()> {
    info!("Reading Rhythmbox playlists...");
    let playlists = File::open(&playlists_path).context("failed to open playlists file")?;
    let mut playlists =
        Element::from_reader(BufReader::new(playlists)).context("failed to read playlists")?;
    ensure!(
        playlists.tag() == &QName::from("rhythmdb-playlists"),
        "unknown playlists format"
    );

    info!("Migrating playlists...");
    playlists
        .get_child_mut(playlists.child_count() - 1)
        .unwrap()
        .set_tail("\n  ");
    for playlist in itunes_library.playlists.iter() {
        if playlist.smart_info.is_some() {
            // Skip smart playlists, until we are able to parse and convert them.
            warn!("playlist {} is skipped because it's smart", playlist.name);
            continue;
        }
        let mut playlist_element = Element::new("playlist");
        playlist_element.set_attr("name", &playlist.name);
        playlist_element.set_attr("type", "static");
        playlist_element.set_text("\n    ");
        let mut unfound_count = 0;
        for item in playlist.items.iter() {
            let location = match track_locations.get(&item.id) {
                Some(location) => location,
                None => {
                    unfound_count += 1;
                    continue;
                }
            };
            let mut location_element = Element::new("location");
            location_element.set_text(location);
            location_element.set_tail("\n    ");
            playlist_element.append_child(location_element);
        }
        let item_count = playlist_element.child_count();
        if item_count > 0 {
            playlist_element
                .get_child_mut(item_count - 1)
                .unwrap()
                .set_tail("\n  ");
        } else {
            playlist_element.set_text("");
        }
        playlist_element.set_tail("\n  ");
        playlists.append_child(playlist_element);
        if unfound_count > 0 {
            warn!(
                "{} items in playlist {} are not found",
                unfound_count, playlist.name
            );
        }
    }
    playlists
        .get_child_mut(playlists.child_count() - 1)
        .unwrap()
        .set_tail("\n");

    info!("Saving the playlists...");
    let playlists_file =
        File::create(&playlists_path).context("failed to open playlists to update")?;
    let options = WriteOptions::new().set_xml_prolog(Some(XmlProlog::Version10));
    playlists
        .to_writer_with_options(BufWriter::new(playlists_file), options)
        .context("failed to update playlists")?;

    Ok(())
}
