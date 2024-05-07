use alloy::{
    rpc::types::{eth::{BlockNumberOrTag, TransactionRequest}, 
    trace::geth::{GethDebugTracingCallOptions, GethTrace}}, 
    transports::Transport,
    providers::Provider, 
    network::Network,
};
use revm::{
    primitives::{EnvWithHandlerCfg, ResultAndState}, 
    Database, Evm 
};
use eyre::Result;

use crate::{
    response_handlers,
    makers,
};

// todo: support state and account overrides
pub async fn geth_trace<P, T, N>(
    provider: &P, 
    tx_request: &TransactionRequest, 
    block_num: BlockNumberOrTag,
    tracing_opt: GethDebugTracingCallOptions,
) -> Result<GethTrace> 
    where P: Provider<T, N>, T: Transport + Clone, N: Network
{
    let mut inspector = makers::make_inspector();
    let evm = makers::make_evm_with_env(provider, tx_request, block_num, &mut inspector).await?;
    let (result, db, _env) = execute(evm)?;
    let trace = response_handlers::handle_response(result, db, inspector, tracing_opt.tracing_options)?;
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
