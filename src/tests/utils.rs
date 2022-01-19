use serenity::model::prelude::User;
use std::time::Duration;

use crate::utils::{get_full_username, get_human_readable_timestamp};

#[test]
fn test_get_full_username() {
    let mut user = User::default();
    user.name = "hello world".to_string();
    user.discriminator = 1234;

    let result = get_full_username(&user);
    assert_eq!(result, "hello world#1234");
}

#[test]
fn test_get_human_readable_timestamp() {
    let duration = Duration::from_secs(53);
    let result = get_human_readable_timestamp(duration);
    assert_eq!(result, "00:53");

    let duration = Duration::from_secs(3599);
    let result = get_human_readable_timestamp(duration);
    assert_eq!(result, "59:59");

    let duration = Duration::from_secs(96548);
    let result = get_human_readable_timestamp(duration);
    assert_eq!(result, "26:49:08");
}
