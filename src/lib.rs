extern crate failure;
extern crate rustic_core as rustic;
extern crate serde;
#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate log;
extern crate id3;
extern crate walkdir;

pub mod scanner;

use failure::Error;
use rustic::library::{self, SharedLibrary};
use rustic::provider::*;

#[derive(Clone, Deserialize, Debug)]
pub struct LocalProvider {
    path: String,
}

impl ProviderInstance for LocalProvider {
    fn title(&self) -> &'static str {
        "Local"
    }

    fn uri_scheme(&self) -> &'static str {
        "file"
    }

    fn setup(&mut self) -> Result<(), Error> {
        Ok(())
    }

    fn sync(&mut self, library: SharedLibrary) -> Result<SyncResult, Error> {
        let scanner = scanner::Scanner::new(self.path.clone());
        let tracks = scanner.scan()?;
        let albums: Vec<library::Album> = tracks
            .iter()
            .cloned()
            .map(|track| track.into())
            .filter(|album: &Option<library::Album>| album.is_some())
            .map(|album| album.unwrap())
            .fold(Vec::new(), |mut albums, album| {
                if albums.iter().find(|a| a.title == album.title).is_none() {
                    albums.push(album);
                }
                albums
            });
        let albums: Vec<library::Album> = albums
            .into_iter()
            .map(|mut album| -> Result<library::Album, Error> {
                library.add_album(&mut album)?;
                Ok(album)
            }).filter(|a| a.is_ok())
            .map(|a| a.unwrap())
            .collect();
        let mut tracks = tracks
            .into_iter()
            .map(library::Track::from)
            .map(|mut t| {
                if let Some(track_album) = &t.album {
                    let album = albums.iter().find(|a| a.title == track_album.title);
                    if let Some(album) = album {
                        t.album_id = album.id;
                    }
                }
                t
            }).collect();
        library.add_tracks(&mut tracks)?;
        Ok(SyncResult {
            tracks: tracks.len(),
            albums: albums.len(),
            artists: 0,
            playlists: 0,
        })
    }

    fn root(&self) -> ProviderFolder {
        ProviderFolder {
            folders: vec![],
            items: vec![],
        }
    }

    fn navigate(&self, _path: Vec<String>) -> Result<ProviderFolder, Error> {
        Ok(self.root())
    }

    fn search(&self, _query: String) -> Result<Vec<ProviderItem>, Error> {
        Ok(vec![])
    }

    fn resolve_track(&self, _uri: &str) -> Result<Option<library::Track>, Error> {
        Ok(None)
    }
}

impl From<scanner::Track> for library::Track {
    fn from(track: scanner::Track) -> Self {
        library::Track {
            id: None,
            title: track.title,
            album_id: None,
            album: track.album.map(|name| library::Album {
                id: None,
                title: name,
                artist_id: None,
                artist: None,
                provider: Provider::LocalMedia,
                image_url: None,
                uri: String::new(),
            }),
            artist_id: None,
            artist: track.artist.map(|name| library::Artist {
                id: None,
                name,
                uri: String::new(),
                image_url: None,
            }),
            image_url: None,
            stream_url: format!("file://{}", track.path),
            provider: Provider::LocalMedia,
            uri: format!("file://{}", track.path),
            duration: None,
        }
    }
}

impl From<scanner::Track> for Option<library::Album> {
    fn from(track: scanner::Track) -> Self {
        track.album.map(|name| library::Album {
            id: None,
            title: name,
            artist_id: None,
            artist: None,
            provider: Provider::LocalMedia,
            image_url: None,
            uri: String::new(),
        })
    }
}

impl From<scanner::Track> for Option<library::Artist> {
    fn from(track: scanner::Track) -> Self {
        track.artist.map(|name| library::Artist {
            id: None,
            name,
            uri: String::new(),
            image_url: None,
        })
    }
}
