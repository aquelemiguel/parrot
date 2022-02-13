pub mod autopause;
pub mod clear;
pub mod forceskip;
pub mod leave;
pub mod now_playing;
pub mod pause;
pub mod play;
pub mod queue;
pub mod remove;
pub mod repeat;
pub mod resume;
pub mod seek;
pub mod shuffle;
pub mod skip;
pub mod stop;
pub mod summon;
pub mod version;

pub enum PlayFlag {
    DEFAULT,
    PLAYTOP,
    PLAYALL,
}

pub enum EnqueueType {
    URI,
    SEARCH,
    PLAYLIST,
}
