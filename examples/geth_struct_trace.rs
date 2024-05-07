use alloy::{
    providers::{Provider, ReqwestProvider}, 
    primitives::fixed_bytes, 
    rpc::types::{
        trace::geth::{GethDebugTracingOptions, GethDefaultTracingOptions},
        eth::{BlockNumberOrTag, Transaction}, 
    },
};
use eyre::Result;


#[tokio::main]
async fn main() -> Result<()> {
    let provider_url = "https://eth.drpc.org";
    let target_tx_hash = fixed_bytes!("2eca348f548244099f420147ab27b00bf9f42e0978e253aa440ac4502822c1ab");

    let provider = ReqwestProvider::new_http(provider_url.parse()?);
    let tx: Transaction = provider.get_transaction_by_hash(target_tx_hash).await?;
    let block_num = BlockNumberOrTag::Number(tx.block_number.unwrap());
    let mut tracing_opt = GethDebugTracingOptions::default();
    tracing_opt.config = GethDefaultTracingOptions::default()
        .with_disable_memory(false)
        .with_enable_memory(true)
        .with_disable_stack(false);

    let traces = poor_mans_tracer::geth_trace(
        provider, 
        tx.into(), 
        block_num, 
        tracing_opt
    ).await?;

    println!("{:#?}", traces);
    Ok(())
}

pub fn get_http_provider(url: &str) -> Result<ReqwestProvider> {
    Ok(ReqwestProvider::new_http(url.parse()?))
}