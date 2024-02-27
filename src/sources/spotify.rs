use crate::{
    commands::play::QueryType,
    errors::ParrotError,
    messaging::messages::{SPOTIFY_INVALID_QUERY, SPOTIFY_PLAYLIST_FAILED},
};
use lazy_static::lazy_static;
use regex::Regex;
use rspotify::{
    clients::BaseClient,
    model::{AlbumId, PlayableItem, PlaylistId, SimplifiedArtist, TrackId},
    ClientCredsSpotify, Credentials,
};
use std::{env, str::FromStr};
use tokio::sync::Mutex;

lazy_static! {
    pub static ref SPOTIFY: Mutex<Result<ClientCredsSpotify, ParrotError>> =
        Mutex::new(Err(ParrotError::Other("no auth attempts")));
    pub static ref SPOTIFY_QUERY_REGEX: Regex =
        Regex::new(r"spotify.com/(?P<media_type>.+)/(?P<media_id>.*?)(?:\?|$)").unwrap();
}

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
    pub async fn auth() -> Result<ClientCredsSpotify, ParrotError> {
        let spotify_client_id = env::var("SPOTIFY_CLIENT_ID")
            .map_err(|_| ParrotError::Other("missing spotify client ID"))?;

        let spotify_client_secret = env::var("SPOTIFY_CLIENT_SECRET")
            .map_err(|_| ParrotError::Other("missing spotify client secret"))?;

        let creds = Credentials::new(&spotify_client_id, &spotify_client_secret);

        let spotify = ClientCredsSpotify::new(creds);
        spotify.request_token().await?;

        Ok(spotify)
    }

    pub async fn extract(
        spotify: &ClientCredsSpotify,
        query: &str,
    ) -> Result<QueryType, ParrotError> {
        let captures = SPOTIFY_QUERY_REGEX
            .captures(query)
            .ok_or(ParrotError::Other(SPOTIFY_INVALID_QUERY))?;

        let media_type = captures
            .name("media_type")
            .ok_or(ParrotError::Other(SPOTIFY_INVALID_QUERY))?
            .as_str();

        let media_type = MediaType::from_str(media_type)
            .map_err(|_| ParrotError::Other(SPOTIFY_INVALID_QUERY))?;

        let media_id = captures
            .name("media_id")
            .ok_or(ParrotError::Other(SPOTIFY_INVALID_QUERY))?
            .as_str();

        match media_type {
            MediaType::Track => Self::get_track_info(spotify, media_id).await,
            MediaType::Album => Self::get_album_info(spotify, media_id).await,
            MediaType::Playlist => Self::get_playlist_info(spotify, media_id).await,
        }
    }

    async fn get_track_info(
        spotify: &ClientCredsSpotify,
        id: &str,
    ) -> Result<QueryType, ParrotError> {
        let track_id = TrackId::from_id(id)
            .map_err(|_| ParrotError::Other("track ID contains invalid characters"))?;

        let track = spotify
            .track(track_id, None)
            .await
            .map_err(|_| ParrotError::Other("failed to fetch track"))?;

        let artist_names = Self::join_artist_names(&track.artists);

        let query = Self::build_query(&artist_names, &track.name);
        Ok(QueryType::Keywords(query))
    }

    async fn get_album_info(
        spotify: &ClientCredsSpotify,
        id: &str,
    ) -> Result<QueryType, ParrotError> {
        let album_id = AlbumId::from_id(id)
            .map_err(|_| ParrotError::Other("album ID contains invalid characters"))?;

        let album = spotify
            .album(album_id, None)
            .await
            .map_err(|_| ParrotError::Other("failed to fetch album"))?;

        let artist_names = Self::join_artist_names(&album.artists);

        let query_list: Vec<String> = album
            .tracks
            .items
            .iter()
            .map(|track| Self::build_query(&artist_names, &track.name))
            .collect();

        Ok(QueryType::KeywordList(query_list))
    }

    async fn get_playlist_info(
        spotify: &ClientCredsSpotify,
        id: &str,
    ) -> Result<QueryType, ParrotError> {
        let playlist_id = PlaylistId::from_id(id)
            .map_err(|_| ParrotError::Other("playlist ID contains invalid characters"))?;

        let playlist = spotify
            .playlist(playlist_id, None, None)
            .await
            .map_err(|_| ParrotError::Other(SPOTIFY_PLAYLIST_FAILED))?;

        let query_list: Vec<String> = playlist
            .tracks
            .items
            .iter()
            .filter_map(|item| match item.track.as_ref().unwrap() {
                PlayableItem::Track(track) => {
                    let artist_names = Self::join_artist_names(&track.album.artists);
                    Some(Self::build_query(&artist_names, &track.name))
                }
                PlayableItem::Episode(_) => None,
            })
            .collect();

        Ok(QueryType::KeywordList(query_list))
    }

    fn build_query(artists: &str, track_name: &str) -> String {
        format!("{} - {}", artists, track_name)
    }

    fn join_artist_names(artists: &[SimplifiedArtist]) -> String {
        let artist_names: Vec<String> = artists.iter().map(|artist| artist.name.clone()).collect();
        artist_names.join(" ")
    }
}
