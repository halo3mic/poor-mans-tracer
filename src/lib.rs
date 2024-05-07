use alloy::{
    rpc::types::trace::geth::{GethDebugTracingOptions, GethTrace},
    rpc::types::eth::{BlockNumberOrTag, TransactionRequest},
    providers::ReqwestProvider,
};
use revm::{
    primitives::{EnvWithHandlerCfg, ResultAndState}, 
    Database, Evm 
};
use eyre::Result;

mod response_handlers;
mod makers;
mod utils;

// todo: Make optimism tracing an option and test it works
// todo add readme

pub async fn geth_trace(
    provider: ReqwestProvider, 
    tx_request: TransactionRequest, 
    block_num: BlockNumberOrTag,
    tracing_opt: GethDebugTracingOptions,
) -> Result<GethTrace> {
    let mut inspector = makers::make_inspector();
    let evm = makers::make_evm_with_env(&provider, tx_request, block_num, &mut inspector).await?;
    let (result, db, _env) = execute(evm)?;
    let trace = response_handlers::handle_response(result, db, inspector, tracing_opt)?;
    Ok(trace)
}

fn execute<EXT, DB>(mut evm: Evm<EXT, DB>) -> Result<(ResultAndState, DB, EnvWithHandlerCfg)> 
where
    DB: Database,
    DB::Error: 'static + Send + Sync + std::error::Error,
{
    let result = evm.transact()?;
    let (db, env) = evm.into_db_and_env_with_handler_cfg();
    Ok((result, db, env))
}
