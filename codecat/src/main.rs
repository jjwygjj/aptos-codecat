use anyhow::{Context, Result};
use clap::Parser;

use aptos_sdk::{rest_client::Client, types::LocalAccount};

use aptos_sdk::{
    bcs,
    move_types::{identifier::Identifier, language_storage::ModuleId},
    rest_client::PendingTransaction,
    transaction_builder::TransactionBuilder,
    types::{
        account_address::AccountAddress,
        chain_id::ChainId,
        transaction::{EntryFunction, TransactionPayload},
    },
};

use once_cell::sync::Lazy;
use std::str::FromStr;
use std::time::{SystemTime, UNIX_EPOCH};
use url::Url;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(short, long)]
    name: String,

    #[arg(short, long, default_value = "")]
    uri: String,
}

static NODE_URL: Lazy<Url> = Lazy::new(|| {
    Url::from_str(
        std::env::var("APTOS_NODE_URL")
            .as_ref()
            .map(|s| s.as_str())
            .unwrap_or("https://fullnode.devnet.aptoslabs.com"),
    )
    .unwrap()
});

static PRIVATE_KEY: Lazy<String> = Lazy::new(|| {
    std::env::var("PRIVATE_KEY")
        .as_ref()
        .map(|s| s.as_str())
        .unwrap_or("")
        .to_owned()
});

static MODULE_ADDRESS: Lazy<String> = Lazy::new(|| {
    std::env::var("MODULE_ADDRESS")
        .as_ref()
        .map(|s| s.as_str())
        .unwrap_or("")
        .to_owned()
});

#[tokio::main]
async fn main() -> Result<()> {
    let args: Args = Args::parse();
    let mut code_cat = get_account().await?;
    let api_client = Client::new(NODE_URL.clone());

    if args.uri.is_empty() {
        let resource = get_account_resource_data(&mut code_cat).await;
        let handle = &resource["codes"]["handle"].as_str().unwrap()[2..];
        let uri = api_client
            .get_table_item(
                AccountAddress::from_hex(handle)?,
                "0x1::string::String",
                &format!("{}::codecat::Code", MODULE_ADDRESS.clone()),
                args.name,
            )
            .await?
            .inner()["uri"]
            .as_str()
            .map(|s| s.to_owned())
            .context("not found uri")?;
        println!("{}", uri);
    } else {
        let txn_hash = add_code(&mut code_cat, args.name, args.uri).await?;
        api_client
            .wait_for_transaction(&txn_hash)
            .await
            .context("Failed when waiting for the transfer transaction")?;
    }

    Ok(())
}

async fn get_account_resource_data(from_account: &mut LocalAccount) -> serde_json::Value {
    let api_client = Client::new(NODE_URL.clone());

    match api_client
        .get_account_resource(
            from_account.address(),
            &format!("{}::codecat::CodeList", MODULE_ADDRESS.clone()),
        )
        .await
    {
        Err(_) => {
            let txn_hash = register(from_account).await.unwrap();
            api_client.wait_for_transaction(&txn_hash).await.unwrap();
            api_client
                .get_account_resource(
                    from_account.address(),
                    &format!("{}::codecat::CodeList", MODULE_ADDRESS.clone()),
                )
                .await
                .unwrap()
                .inner()
                .to_owned()
                .unwrap()
                .data
        },
        Ok(resource) => resource.inner().to_owned().unwrap().data,
    }
}

async fn get_account() -> Result<LocalAccount> {
    let api_client = Client::new(NODE_URL.clone());
    let mut code_cat = LocalAccount::from_private_key(&hex::decode(PRIVATE_KEY.clone())?, 0)?;
    let sequence_number = api_client
        .get_account(code_cat.address())
        .await?
        .inner()
        .sequence_number;
    *code_cat.sequence_number_mut() = sequence_number;
    Ok(code_cat)
}

async fn register(from_account: &mut LocalAccount) -> Result<PendingTransaction> {
    let api_client = Client::new(NODE_URL.clone());
    let chain_id = api_client
        .get_index()
        .await
        .context("Failed to get chain ID")?
        .inner()
        .chain_id;
    let transaction_builder = TransactionBuilder::new(
        TransactionPayload::EntryFunction(EntryFunction::new(
            ModuleId::new(from_account.address(), Identifier::new("codecat").unwrap()),
            Identifier::new("register").unwrap(),
            vec![],
            vec![],
        )),
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs()
            + 100,
        ChainId::new(chain_id),
    )
    .sender(from_account.address())
    .sequence_number(from_account.sequence_number())
    .max_gas_amount(2000)
    .gas_unit_price(100);
    let signed_txn = from_account.sign_with_transaction_builder(transaction_builder);
    Ok(api_client
        .submit(&signed_txn)
        .await
        .context("Failed to submit transfer transaction")?
        .into_inner())
}

async fn add_code(
    from_account: &mut LocalAccount,
    name: String,
    uri: String,
) -> Result<PendingTransaction> {
    let api_client = Client::new(NODE_URL.clone());
    let chain_id = api_client
        .get_index()
        .await
        .context("Failed to get chain ID")?
        .inner()
        .chain_id;
    let transaction_builder = TransactionBuilder::new(
        TransactionPayload::EntryFunction(EntryFunction::new(
            ModuleId::new(from_account.address(), Identifier::new("codecat").unwrap()),
            Identifier::new("add_code").unwrap(),
            vec![],
            vec![
                bcs::to_bytes(&name.into_bytes()).unwrap(),
                bcs::to_bytes(&uri.into_bytes()).unwrap(),
            ],
        )),
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs()
            + 100,
        ChainId::new(chain_id),
    )
    .sender(from_account.address())
    .sequence_number(from_account.sequence_number())
    .max_gas_amount(2000)
    .gas_unit_price(100);
    let signed_txn = from_account.sign_with_transaction_builder(transaction_builder);
    Ok(api_client
        .submit(&signed_txn)
        .await
        .context("Failed to submit transfer transaction")?
        .into_inner())
}
