use crate::*;

use crate::data::NearSocialAccountId;
use serde::de::DeserializeOwned;
use serde::{Deserialize, Deserializer};

fn make_image_url(prefix: &str, account_id: NearSocialAccountId, file_name: &str) -> String {
    let id = format!("{:018}", account_id);
    format!(
        "https://mastodon.near.social/system/accounts/{}/{}/{}/{}/{}/{}/{}/original/{}",
        prefix,
        &id[0..3],
        &id[3..6],
        &id[6..9],
        &id[9..12],
        &id[12..15],
        &id[15..18],
        file_name
    )
}

pub fn add_near_suffix(username: &AccountId) -> AccountId {
    format!("{}.near", username)
}

pub fn make_image(
    account_id: NearSocialAccountId,
    prefix: &str,
    file_name: Option<String>,
    content_type: Option<String>,
    file_size: Option<u64>,
) -> Option<Image> {
    file_name.map(|file_name| Image {
        url: make_image_url(prefix, account_id, &file_name),
        content_type: content_type.unwrap(),
        file_size: file_size.unwrap(),
    })
}

pub fn deserialize_null_default<'de, D, T>(deserializer: D) -> Result<T, D::Error>
where
    T: Default + Deserialize<'de>,
    D: Deserializer<'de>,
{
    let opt = Option::deserialize(deserializer)?;
    Ok(opt.unwrap_or_default())
}

pub async fn fetch(url: &str) -> Result<Vec<u8>, io::Error> {
    let body = reqwest::get(url)
        .await
        .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?
        .bytes()
        .await
        .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
    Ok(body.to_vec())
}

pub async fn fetch_json<T>(url: &str) -> Result<T, io::Error>
where
    T: DeserializeOwned,
{
    let body = fetch(url).await?;
    serde_json::from_slice(&body).map_err(|e| io::Error::new(io::ErrorKind::Other, e))
}
