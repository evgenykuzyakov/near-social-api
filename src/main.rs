mod data;
use crate::data::*;
mod utils;
use crate::utils::*;

use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::{env, io};
use tokio_cron_scheduler::{Job, JobScheduler};
use warp::Filter;

#[tokio::main]
async fn main() -> Result<(), io::Error> {
    if env::var_os("RUST_LOG").is_none() {
        env::set_var("RUST_LOG", "api=info");
    }
    pretty_env_logger::init();

    let mut sched = JobScheduler::new();

    let data = Data::load().await?;

    println!("Data loaded!\nNum accounts: {}", data.accounts.len(),);

    let async_data = Arc::new(Mutex::new(data));
    let wrap_async_data = async_data.clone();

    let wrap_data = warp::any().map(move || wrap_async_data.clone());

    let refresh_async_data = async_data.clone();

    sched
        .add(
            Job::new_async("10 * * * * *", move |_uuid, _l| {
                let async_data = refresh_async_data.clone();
                Box::pin(async move {
                    match Data::load().await {
                        Ok(new_data) => {
                            let mut data = async_data.lock().unwrap();
                            *data = new_data;
                        }
                        Err(e) => {
                            println!("Error while refetching data: {}", e);
                        }
                    }
                })
            })
            .unwrap(),
        )
        .expect("Failed to schedule refresh job");

    sched.start();

    let cors = warp::cors().allow_any_origin();
    let log = warp::log("api");

    let accounts = warp::path!("account" / AccountId)
        .and(wrap_data)
        .and_then(|account_id, async_data| async move {
            if let Some(account) = get_account(account_id, async_data).await {
                Ok(warp::reply::json(&account))
            } else {
                Err(warp::reject::not_found())
            }
        })
        .with(cors.clone())
        .with(log);

    println!("Serving");
    warp::serve(accounts).run(([127, 0, 0, 1], 3032)).await;

    Ok(())
}

async fn get_account(account_id: AccountId, async_data: AsyncData) -> Option<Account> {
    async_data.lock().unwrap().get_account(&account_id)
}
