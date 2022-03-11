mod data;
use crate::data::*;
mod utils;
use crate::utils::*;

use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::{env, io};
use warp::http::Response;
use warp::Filter;

#[tokio::main]
async fn main() -> Result<(), io::Error> {
    if env::var_os("RUST_LOG").is_none() {
        // Set `RUST_LOG=img=debug` to see debug logs,
        // this only shows access logs.
        env::set_var("RUST_LOG", "img=info");
    }
    pretty_env_logger::init();

    let data = Data::load().await?;

    println!("Data loaded!\nNum accounts: {}", data.accounts.len(),);

    let async_data = Arc::new(Mutex::new(data));
    let async_data = warp::any().map(move || async_data.clone());

    let cors = warp::cors().allow_any_origin();
    let log = warp::log("api");

    let accounts = warp::path!("account" / AccountId)
        .and(async_data)
        .and_then(|account_id, async_data| async move {
            if let Some(account) = get_account(account_id, async_data).await {
                Ok(warp::reply::json(&account))
            } else {
                Err(warp::reject::not_found())
            }
        })
        .with(cors.clone())
        .with(log);

    warp::serve(accounts).run(([127, 0, 0, 1], 3032)).await;

    Ok(())
}

async fn get_account(account_id: AccountId, async_data: AsyncData) -> Option<Account> {
    async_data.lock().unwrap().get_account(&account_id)
}
