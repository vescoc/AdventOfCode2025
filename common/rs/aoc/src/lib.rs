use std::fs;
use std::path::PathBuf;

use toml::{Table, Value};

use chrono::{FixedOffset, NaiveDate, TimeZone, Utc};

use dirs::config_dir;

use reqwest::{
    blocking::{Client as HttpClient, Response},
    header::{self, HeaderMap, HeaderValue},
    redirect::Policy,
};

fn is_day_unlocked(year: i32, day: u32) -> bool {
    let timezone = FixedOffset::east_opt(-5 * 3600).unwrap();
    let now = timezone.from_utc_datetime(&Utc::now().naive_utc());

    let local_datetime = NaiveDate::from_ymd_opt(year, 12, day)
        .unwrap()
        .and_hms_opt(0, 0, 0)
        .unwrap();

    let unlock_datetime = timezone
        .from_local_datetime(&local_datetime)
        .single()
        .unwrap();

    now.signed_duration_since(unlock_datetime)
        .num_milliseconds()
        >= 0
}

/// # Panics
pub fn get_input(year: i32, day: u32, input_file: &str) {
    let input = PathBuf::from(input_file);

    let input_data = fs::read_to_string(&input);

    let input_is_missing = input_data.as_ref().map(String::is_empty).unwrap_or(true);
    if !input_is_missing {
        return;
    }

    if !is_day_unlocked(year, day) {
        let data = "";
        let must_write = input_data.map(|content| content != data).unwrap_or(true);
        if must_write {
            fs::write(&input, data).expect("cannot write (default) input file");
        }
        return;
    }

    let session_key = fs::read_to_string(
        config_dir()
            .expect("cannot find config dir")
            .join("adventofcode.session"),
    )
    .expect("cannot read session key");

    let cookie_header = HeaderValue::from_str(&format!("session={}", session_key.trim())).unwrap();
    let content_type_header = HeaderValue::from_str("text/plain").unwrap();
    let user_agent_header = HeaderValue::from_str(&format!(
        "{} {}",
        env!("CARGO_PKG_REPOSITORY"),
        env!("CARGO_PKG_VERSION")
    ))
    .unwrap();

    let url = format!("https://adventofcode.com/{year}/day/{day}/input");

    let mut headers = HeaderMap::new();
    headers.insert(header::COOKIE, cookie_header);
    headers.insert(header::CONTENT_TYPE, content_type_header);
    headers.insert(header::USER_AGENT, user_agent_header);

    let data = HttpClient::builder()
        .default_headers(headers)
        .redirect(Policy::none())
        .build()
        .unwrap()
        .get(url)
        .send()
        .and_then(Response::error_for_status)
        .and_then(Response::text)
        .unwrap();

    let must_write = input_data.map(|content| content != data).unwrap_or(true);
    if must_write {
        fs::write(&input, &data).expect("cannot write (default) input file");
    }
}

/// # Panics
pub fn get_input_info_from_cargo(input_file: Option<String>) {
    let config = fs::read_to_string("Cargo.toml")
        .expect("cannot find Cargo.toml")
        .parse::<Table>()
        .expect("invalid Cargo.toml");

    let data = config
        .get("package")
        .and_then(|value| value.get("metadata"))
        .and_then(|value| value.get("aoc"))
        .expect("cannot find package.metadata.aoc");

    let year = get_int(
        data.get("year")
            .expect("cannot find package.metadata.aoc.year"),
    )
    .expect("invalid year")
    .try_into()
    .expect("invalid year value");
    let day = get_int(
        data.get("day")
            .expect("cannot find package.metadata.aoc.day"),
    )
    .expect("invalid day")
    .try_into()
    .expect("invalid day value");
    let input_file = data
        .get("input_file")
        .and_then(get_string)
        .or(input_file)
        .expect("invalid input file");

    get_input(year, day, &input_file);
}

fn get_int(value: &Value) -> Option<i64> {
    match value {
        Value::Integer(v) => Some(*v),
        _ => None,
    }
}

fn get_string(value: &Value) -> Option<String> {
    match value {
        Value::String(v) => Some(v.clone()),
        _ => None,
    }
}
