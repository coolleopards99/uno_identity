//
// Copyright (C) 2021 WithUno, Inc.
// All rights reserved.
//
// SPDX-License-Identifier: AGPL-3.0-only
//

use anyhow::Result;

#[cfg(feature = "s3")]
use anyhow::Context;

#[cfg(not(feature = "s3"))]
use api::store::FileStore;
#[cfg(not(feature = "s3"))]
async fn make_db(name: &'static str, version: &str) -> Result<FileStore>
{
    // use the current directory
    // TODO: figure out a better dir like /var/db but one that doesn't require
    //       root
    FileStore::new(".", name, version).await
}

#[cfg(feature = "s3")]
use api::store::S3Store;
#[cfg(feature = "s3")]
async fn make_db(name: &str, version: &str) -> Result<S3Store>
{
    let key_id = std::env::var("SPACES_ACCESS_KEY_ID")
        .context("Failed to lookup SPACES_ACCESS_KEY_ID")?;

    let secret = std::env::var("SPACES_SECRET_ACCESS_KEY")
        .context("Failed to lookup SPACES_SECRET_ACCESS_KEY")?;

    let host = std::env::var("SPACES_HOSTNAME")
        .context("Failed to lookup SPACES_HOSTNAME")?;

    let region = std::env::var("SPACES_REGION")
        .context("Failed to lookup SPACES_REGION")?;

    let bucket = std::env::var("SPACES_BUCKET_PREFIX")
        .context("Failed to lookup SPACES_BUCKET_PREFIX")?;

    let name = String::from(name) + "." + &String::from(bucket);

    S3Store::new(&host, &region, &key_id, &secret, &name, version).await
}

#[async_std::main]
async fn main() -> Result<()>
{
    let tok2 = make_db("tokens", "v2").await?;
    let vau2 = make_db("vaults", "v2").await?;
    let srv2 = make_db("services", "").await?; // not (yet) versioned
    let ses2 = make_db("sessions", "v2").await?;
    let mbx2 = make_db("mailboxes", "v2").await?;
    let shr2 = make_db("shares", "v2").await?;

    let api_v2 = api::build_routes(tok2, vau2, srv2, ses2, mbx2, shr2)?;

    let mut srv = tide::new();

    srv.at("/v2").nest(api_v2);

    tide::log::start();

    let port = std::env::var("PORT").unwrap_or("8080".to_string());
    srv.listen(format!("[::]:{}", port)).await?;
    Ok(())
}
