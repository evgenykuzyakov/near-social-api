use crate::*;
use serde::{Deserialize, Serialize};

pub type AccountId = String;
pub type NearSocialAccountId = u64;

/*
"id":107916959145894674,
"username":"nate",
"domain":null,
"created_at":"2022-03-07T19:41:55.929584",
"note":"",
"display_name":"nategeier",
"avatar_file_name":"d1cd99c5c40bc501.jpeg",
"avatar_content_type":"image/jpeg",
"avatar_file_size":30755,
"header_file_name":"518edec4c916d88d.jpg","header_content_type":"image/jpeg","header_file_size":278290
"fields":[{"name": "hey", "value": "ho"}],
"actor_type":"Person",
"discoverable":true
 */

#[derive(Serialize, Deserialize, Clone)]
pub struct AccountField {
    pub name: String,
    pub value: String,
}

#[derive(Serialize, Deserialize)]
pub struct RawAccount {
    pub id: NearSocialAccountId,
    pub username: String,
    pub domain: Option<String>,
    pub created_at: String,
    pub note: String,
    pub display_name: String,
    pub avatar_file_name: Option<String>,
    pub avatar_content_type: Option<String>,
    pub avatar_file_size: Option<u64>,
    pub header_file_name: Option<String>,
    pub header_content_type: Option<String>,
    pub header_file_size: Option<u64>,
    #[serde(deserialize_with = "deserialize_null_default")]
    pub fields: Vec<AccountField>,
    #[serde(deserialize_with = "deserialize_null_default")]
    pub actor_type: String,
    #[serde(deserialize_with = "deserialize_null_default")]
    pub discoverable: bool,
    #[serde(default)]
    pub statuses_count: u64,
    #[serde(default)]
    pub last_status_at: Option<String>,
}

// "id":1,
// "account_id":107770678854969080,
// "statuses_count":2,
// "following_count":0,
// "created_at":"2022-02-10T00:41:25.3457","last_status_at":"2022-02-13T00:53:11.542797

#[derive(Serialize, Deserialize)]
pub struct AccountStats {
    pub id: u64,
    pub account_id: NearSocialAccountId,
    pub statuses_count: u64,
    pub following_count: u64,
    pub created_at: String,
    pub last_status_at: Option<String>,
}

// {"id":1,"created_at":"2022-02-17T20:51:20.940777","account_id":107810547446224761,"target_account_id":107815243269595728}, {

#[derive(Serialize, Deserialize)]
pub struct FollowEdge {
    pub id: u64,
    pub created_at: String,
    pub account_id: NearSocialAccountId,
    pub target_account_id: NearSocialAccountId,
}

pub struct Data {
    pub accounts: HashMap<AccountId, Account>,
}

pub type AsyncData = Arc<Mutex<Data>>;

#[derive(Deserialize, Serialize, Clone)]
pub struct Image {
    pub url: String,
    pub content_type: String,
    pub file_size: u64,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Account {
    pub account_id: AccountId,
    pub note: String,
    pub display_name: String,
    pub avatar: Option<Image>,
    pub header: Option<Image>,
    pub fields: Vec<AccountField>,
    pub statuses_count: u64,
    pub following: Vec<AccountId>,
    pub followers: Vec<AccountId>,
}

impl Data {
    pub async fn load() -> Result<Data, io::Error> {
        let accounts: Vec<RawAccount> =
            fetch_json("https://near.social/data/accounts.json").await?;
        let account_stats: Vec<AccountStats> =
            fetch_json("https://near.social/data/account_stats.json").await?;
        let follow_edges: Vec<FollowEdge> =
            fetch_json("https://near.social/data/follows.json").await?;

        let account_ids: HashMap<NearSocialAccountId, AccountId> = HashMap::from_iter(
            accounts
                .iter()
                .map(|account| (account.id, add_near_suffix(&account.username))),
        );

        let mut raw_accounts: HashMap<NearSocialAccountId, RawAccount> =
            HashMap::from_iter(accounts.into_iter().map(|account| (account.id, account)));

        for account_stats in account_stats {
            if let Some(account) = raw_accounts.get_mut(&account_stats.account_id) {
                account.statuses_count = account_stats.statuses_count;
                account.last_status_at = account_stats.last_status_at;
            }
        }

        let mut following = HashMap::new();
        let mut followers = HashMap::new();
        for edge in follow_edges {
            following
                .entry(edge.account_id)
                .or_insert_with(Vec::new)
                .push(account_ids.get(&edge.target_account_id).unwrap().clone());
            followers
                .entry(edge.target_account_id)
                .or_insert_with(Vec::new)
                .push(account_ids.get(&edge.account_id).unwrap().clone());
        }

        let accounts = raw_accounts
            .into_iter()
            .map(|(id, account)| {
                let RawAccount {
                    note,
                    display_name,
                    avatar_file_name,
                    avatar_content_type,
                    avatar_file_size,
                    header_file_name,
                    header_content_type,
                    header_file_size,
                    fields,
                    statuses_count,
                    ..
                } = account;
                let account_id = account_ids.get(&id).unwrap().clone();
                (
                    account_id.clone(),
                    Account {
                        account_id,
                        note,
                        display_name,
                        avatar: make_image(
                            id,
                            "avatars",
                            avatar_file_name,
                            avatar_content_type,
                            avatar_file_size,
                        ),
                        header: make_image(
                            id,
                            "headers",
                            header_file_name,
                            header_content_type,
                            header_file_size,
                        ),
                        fields,
                        statuses_count,
                        following: following.remove(&id).unwrap_or_default(),
                        followers: followers.remove(&id).unwrap_or_default(),
                    },
                )
            })
            .collect();

        Ok(Data { accounts })
    }

    pub fn get_account(&self, account_id: &AccountId) -> Option<Account> {
        self.accounts.get(account_id).cloned()
    }
}
