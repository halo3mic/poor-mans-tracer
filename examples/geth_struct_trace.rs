use alloy::{
    rpc::types::{
        trace::geth::{GethDebugTracingOptions, GethDefaultTracingOptions, GethTrace},
        eth::{BlockNumberOrTag, Transaction}, 
    },
    providers::{Provider, ReqwestProvider},
    primitives::fixed_bytes,
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

    write_traces_to_file(traces)?;
    println!("Done! ✨");

    Ok(())
}


use std::fs::{self, File};
use std::io::Write;
use std::path::Path;


const TEMP_OUT_DIR: &str = ".temp_out";
const TRACES_FILE: &str = "traces.json";

fn write_traces_to_file(traces: GethTrace) -> Result<()> {
    let dir_path = Path::new(TEMP_OUT_DIR);
    let file_path = dir_path.join(TRACES_FILE);
    ensure_dir_exists(dir_path)?;
    write_data_to_file(&file_path, &serde_json::to_vec_pretty(&traces)?)?;
    Ok(())
}

fn ensure_dir_exists(path: &Path) -> Result<()> {
    if !path.exists() {
        fs::create_dir_all(path)?;
    }
    Ok(())
}

fn write_data_to_file(path: &Path, data: &[u8]) -> Result<()> {
    let mut file = File::create(path)?;
    file.write_all(data)?;
    Ok(())
}