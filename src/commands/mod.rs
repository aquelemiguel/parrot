pub mod clear;
pub mod genius;
pub mod leave;
pub mod now_playing;
pub mod pause;
pub mod play;
pub mod playtop;
pub mod prefix;
pub mod queue;
pub mod remove;
pub mod repeat;
pub mod resume;
pub mod seek;
pub mod shuffle;
pub mod skip;
pub mod stop;
pub mod summon;

pub enum PlayFlag {
    DEFAULT,
    PLAYTOP,
}

pub enum EnqueueType {
    URI,
    SEARCH,
    PLAYLIST,
}
