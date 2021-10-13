use crate::utils;
use std::time::Duration;

#[test]
fn test_get_human_readable_timestamp() {
    let duration = Duration::from_secs(96548);
    let result = utils::get_human_readable_timestamp(duration);
    assert_eq!(result, "26:49:08")
}
