use serde_json::json;
use serenity::model::prelude::User;
use std::time::Duration;

use crate::utils;

#[test]
fn test_get_full_username() {
    let mut user = User::default();
    user.name = "hello world".to_string();
    user.discriminator = 1234;

    let result = utils::get_full_username(&user);
    assert_eq!(result, "hello world#1234");
}

#[test]
fn test_get_human_readable_timestamp() {
    let duration = Duration::from_secs(53);
    let result = utils::get_human_readable_timestamp(duration);
    assert_eq!(result, "00:53");

    let duration = Duration::from_secs(3599);
    let result = utils::get_human_readable_timestamp(duration);
    assert_eq!(result, "59:59");

    let duration = Duration::from_secs(96548);
    let result = utils::get_human_readable_timestamp(duration);
    assert_eq!(result, "26:49:08");
}

#[test]
fn test_merge_json() {
    let mut result = json!({});

    let json = json!({
        "a": 1,
        "b": true,
        "c": "string",
        "d": {
            "f": "level1",
            "g": {
                "h": "level2"
            }
        }
    });
    utils::merge_json(&mut result, &json);
    assert_eq!(result, json);

    let json = json!({
        "a": 5,
        "d": {
            "f": "level1",
            "g": {
                "h": "changed prop"
            },
            "i": "new prop"
        }
    });
    utils::merge_json(&mut result, &json);
    assert_eq!(
        result,
        json!({
            "a": 5,
            "b": true,
            "c": "string",
            "d": {
                "f": "level1",
                "g": {
                    "h": "changed prop"
                },
                "i": "new prop"
            }
        })
    );
}
