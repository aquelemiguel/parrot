use std::str::FromStr;

use regex::Regex;
use rspotify::{
    clients::BaseClient,
    model::{AlbumId, Country, Id, Market, PlayableItem, PlaylistId, TrackId},
    ClientCredsSpotify, Credentials,
};

#[derive(Clone, Copy)]
pub enum MediaType {
    Track,
    Album,
    Playlist,
}

impl FromStr for MediaType {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "track" => Ok(Self::Track),
            "album" => Ok(Self::Album),
            "playlist" => Ok(Self::Playlist),
            _ => Err(()),
        }
    }
}

pub struct Spotify {}

impl Spotify {
    pub async fn auth() -> Result<ClientCredsSpotify, ()> {
        let creds = Credentials::from_env().ok_or(())?;
        let mut spotify = ClientCredsSpotify::new(creds);
        spotify.request_token().await.unwrap();

        Ok(spotify)
    }

    pub fn parse(query: &str) -> (MediaType, String) {
        let re = Regex::new(r"spotify.com/(?P<media_type>.+)/(?P<media_id>.*?)(?:\?|$)").unwrap();
        let captures = re.captures(query).unwrap();

        let media_type = captures.name("media_type").unwrap();
        let media_type = MediaType::from_str(media_type.as_str()).unwrap();

        let media_id = captures.name("media_id").unwrap().as_str().to_string();
        (media_type, media_id)
    }

    pub async fn get_track_info(spotify: &ClientCredsSpotify, id: String) -> Result<String, ()> {
        let track_id = TrackId::from_id(id.as_str()).unwrap();
        let track = spotify.track(&track_id).await.unwrap();

        let artist_names: Vec<String> = track
            .artists
            .iter()
            .map(|artist| artist.name.clone())
            .collect();

        let artist_names = artist_names.join(" ");

        let query = format!("{} - {}", artist_names, track.name);
        Ok(query)
    }

    pub async fn get_album_info(
        spotify: &ClientCredsSpotify,
        id: String,
    ) -> Result<Vec<String>, ()> {
        let album_id = AlbumId::from_id(id.as_str()).unwrap();
        let album = spotify.album(&album_id).await.unwrap();

        let artist_names: Vec<String> = album
            .artists
            .iter()
            .map(|artist| artist.name.clone())
            .collect();

        let artist_names = artist_names.join(" ");

        let queries: Vec<String> = album
            .tracks
            .items
            .iter()
            .map(|track| format!("{} - {}", artist_names, track.name))
            .collect();

        Ok(queries)
    }

    pub async fn get_playlist_info(
        spotify: &ClientCredsSpotify,
        id: String,
    ) -> Result<Vec<String>, ()> {
        let playlist_id = PlaylistId::from_id(id.as_str()).unwrap();
        let playlist = spotify
            .playlist(
                &playlist_id,
                Some(""),
                Some(&Market::Country(Country::UnitedStates)),
            )
            .await
            .unwrap();

        let queries: Vec<String> = playlist
            .tracks
            .items
            .iter()
            .filter_map(|item| match item.track.as_ref().unwrap() {
                PlayableItem::Track(track) => {
                    let artist_names: Vec<String> = track
                        .album
                        .artists
                        .iter()
                        .map(|artist| artist.name.clone())
                        .collect();

                    let artist_names = artist_names.join(" ");

                    let query = format!("{} - {}", artist_names, track.name);
                    Some(query)
                }
                PlayableItem::Episode(_) => None,
            })
            .collect();

        Ok(queries)
    }
}
