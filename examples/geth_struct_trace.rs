use alloy::{
    rpc::types::{
        trace::geth::{GethDebugTracingOptions, GethDefaultTracingOptions, GethTrace},
        eth::{BlockNumberOrTag, Transaction}, 
    },
    providers::{Provider, ReqwestProvider},
    primitives::fixed_bytes,
};
use eyre::{OptionExt, Result};


#[tokio::main]
async fn main() -> Result<()> {
    let provider_url = "https://optimism.drpc.org";
    let target_tx_hash = fixed_bytes!("7de2a03a5aefff675b524abdc4c36ad0965595373f831a759124c10fd30cdaf1");

    let provider = ReqwestProvider::new_http(provider_url.parse()?);
    let tx: Transaction = provider.get_transaction_by_hash(target_tx_hash).await?;
    let block_num = BlockNumberOrTag::Number(tx.block_number.ok_or_eyre("Missing block number")?);
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
    println!("✨ Done!");

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
    
    println!("Traces written to: {}", file_path.display());
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