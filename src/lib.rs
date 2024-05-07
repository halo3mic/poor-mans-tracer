use revm_inspectors::tracing::{TracingInspector, TracingInspectorConfig};
use alloy::{
    rpc::types::trace::geth::{GethDebugTracingOptions, GethTrace},
    rpc::types::eth::{BlockNumberOrTag, TransactionRequest},
    providers::{Provider, ReqwestProvider},
    transports::Transport,
    network::Network,
};
use revm::{
    primitives::{
        CfgEnv, CfgEnvWithHandlerCfg, EnvWithHandlerCfg, ResultAndState, SpecId, U256
    }, 
    db::{alloydb::AlloyDB, CacheDB}, 
    inspector_handle_register, 
    Database, Evm 
};
use eyre::Result;

mod response_handlers;
mod utils;

// todo: Make optimism tracing an option and test it works
// todo add readme

pub async fn geth_trace(
    provider: ReqwestProvider, 
    tx_request: TransactionRequest, 
    block_num: BlockNumberOrTag,
    tracing_opt: GethDebugTracingOptions,
) -> Result<GethTrace> {
    let mut inspector = make_inspector();
    let evm = make_evm_with_env(&provider, tx_request, block_num, &mut inspector).await?;
    let (result, db, _env) = execute(evm)?;
    let trace = response_handlers::handle_response(result, db, inspector, tracing_opt)?;
    Ok(trace)
}

fn make_alloy_cached_db<T: Clone + Transport, N: Network, P: Provider<T, N>>(
    provider: P, 
    block_num: BlockNumberOrTag,
) -> CacheDB<AlloyDB<T, N, P>> {
    let block_num_state = BlockNumberOrTag::Number(block_num.as_number().unwrap() - 1); // todo: consider if this is okay to be here
    let alloy_db = AlloyDB::new(provider, block_num_state.into());
    CacheDB::new(alloy_db)
}

fn make_inspector() -> TracingInspector {
    TracingInspector::new(TracingInspectorConfig::all())
}

async fn make_evm_with_env<'a, 'b: 'a, T, N, P>(
    provider: &'b P,
    tx_request: TransactionRequest,
    block_num: BlockNumberOrTag,
    inspector: &'a mut TracingInspector,
) -> Result<Evm<'a, &'a mut TracingInspector, CacheDB<AlloyDB<T, N, &'b P>>>> 
    where T: Clone + Transport, N: Network, P: Provider<T, N>
{
    let mut evm = make_evm(provider, block_num, inspector);
    let env = make_env_with_cfg_handler(provider, tx_request, block_num).await?;
    evm.context.evm.env = env.env;
    Ok(evm)
}

fn make_evm<'a, 'b: 'a, T, N, P>(
    provider: &'b P, 
    block_num: BlockNumberOrTag,
    inspector: &'a mut TracingInspector,
) -> Evm<'a, &'a mut TracingInspector, CacheDB<AlloyDB<T, N, &'b P>>> 
    where T: Clone + Transport, N: Network, P: Provider<T, N>
{
    let db = make_alloy_cached_db(provider, block_num);
    Evm::builder()
        .with_db(db)
        .with_external_context(inspector)
        .append_handler_register(inspector_handle_register)
        .build()
}

async fn make_env_with_cfg_handler<T: Clone + Transport, N: Network, P: Provider<T, N>>(
    provider: P,
    tx_request: TransactionRequest,
    block_number: BlockNumberOrTag,
) -> Result<EnvWithHandlerCfg> {
    let block = provider.get_block_by_number(block_number, false).await?.unwrap();
    let block_env = utils::blockheader_to_blockenv(&block.header)?;
    let base_fee = U256::wrapping_from(block.header.base_fee_per_gas.expect("Missing base fee"));
    let tx_env = utils::txrequest_to_txenv(&tx_request, base_fee)?;

    let cfgh = CfgEnvWithHandlerCfg::new_with_spec_id(CfgEnv::default(), SpecId::LATEST);
    let env = EnvWithHandlerCfg::new_with_cfg_env(cfgh, block_env, tx_env);

    Ok(env)
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
