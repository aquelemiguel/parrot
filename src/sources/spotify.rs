use std::str::FromStr;

use regex::Regex;
use rspotify::{
    clients::BaseClient,
    model::{AlbumId, Country, Id, Market, PlayableItem, PlaylistId, SimplifiedArtist, TrackId},
    ClientCredsSpotify, Credentials,
};

use crate::commands::play::QueryType;

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

    pub async fn extract(spotify: &ClientCredsSpotify, query: &str) -> Option<QueryType> {
        let re = Regex::new(r"spotify.com/(?P<media_type>.+)/(?P<media_id>.*?)(?:\?|$)").unwrap();
        let captures = re.captures(query).unwrap();

        let media_type = captures.name("media_type").unwrap();
        let media_type = MediaType::from_str(media_type.as_str()).unwrap();

        let media_id = captures.name("media_id").unwrap().as_str();

        match media_type {
            MediaType::Track => match Self::get_track_info(&spotify, media_id).await {
                Ok(query) => Some(QueryType::Keywords(query)),
                Err(_) => None,
            },
            MediaType::Album => match Self::get_album_info(&spotify, media_id).await {
                Ok(query_list) => Some(QueryType::KeywordList(query_list)),
                Err(_) => None,
            },
            MediaType::Playlist => match Self::get_playlist_info(&spotify, media_id).await {
                Ok(query_list) => Some(QueryType::KeywordList(query_list)),
                Err(_) => None,
            },
        }
    }

    async fn get_track_info(spotify: &ClientCredsSpotify, id: &str) -> Result<String, ()> {
        let track_id = TrackId::from_id(id).unwrap();
        let track = spotify.track(&track_id).await.unwrap();
        let artist_names = Self::join_artist_names(&track.artists);

        Ok(Self::build_query(&artist_names, &track.name))
    }

    async fn get_album_info(spotify: &ClientCredsSpotify, id: &str) -> Result<Vec<String>, ()> {
        let album_id = AlbumId::from_id(id).unwrap();
        let album = spotify.album(&album_id).await.unwrap();
        let artist_names = Self::join_artist_names(&album.artists);

        let queries: Vec<String> = album
            .tracks
            .items
            .iter()
            .map(|track| Self::build_query(&artist_names, &track.name))
            .collect();

        Ok(queries)
    }

    async fn get_playlist_info(spotify: &ClientCredsSpotify, id: &str) -> Result<Vec<String>, ()> {
        let playlist_id = PlaylistId::from_id(id).unwrap();
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
                    let artist_names = Self::join_artist_names(&track.album.artists);
                    Some(Self::build_query(&artist_names, &track.name))
                }
                PlayableItem::Episode(_) => None,
            })
            .collect();

        Ok(queries)
    }

    fn build_query(artists: &str, track_name: &str) -> String {
        format!("{} - {}", artists, track_name)
    }

    fn join_artist_names(artists: &[SimplifiedArtist]) -> String {
        let artist_names: Vec<String> = artists.iter().map(|artist| artist.name.clone()).collect();
        artist_names.join(" ")
    }
}
