use revm_inspectors::tracing::{TracingInspector, TracingInspectorConfig};
use alloy::{
    rpc::types::eth::{BlockNumberOrTag, TransactionRequest},
    rpc::types::trace::geth::{GethDebugTracingOptions, GethTrace},
    providers::{Provider, ReqwestProvider},
    transports::Transport,
    network::Network,
};
use revm::{
    db::{alloydb::AlloyDB, CacheDB}, 
    interpreter::InstructionResult, 
    inspector_handle_register, 
    primitives::{
        CfgEnv, CfgEnvWithHandlerCfg, EnvWithHandlerCfg, ExecutionResult, Output, 
        ResultAndState, SpecId, U256
    }, 
    Database, Evm 
};
use eyre::{Ok, Result};

mod utils;

// todo: change so that dependencies point to forked github repos
// todo: Make optimism tracing an option and test it works
/** Upstream comments
     * Why are tx env and block env set twice??
     * The tx env is missing access list and some other fields
     * How is caching done without alloy db?
     */


pub async fn geth_trace(
    provider: ReqwestProvider, 
    tx_request: TransactionRequest, 
    block_num: BlockNumberOrTag,
    tracing_opt: GethDebugTracingOptions,
) -> Result<GethTrace> {
    let mut inspector = make_inspector();
    let mut evm = make_evm(&provider, block_num, &mut inspector);
    let env = make_env(&provider, tx_request, block_num).await?;
    evm.context.evm.env = env.env;
    let (result, _db, _env) = execute(evm)?;
    let trace = handle_response(result, inspector, tracing_opt);
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

fn make_evm<'a, T, N, P>(
    provider: P, 
    block_num: BlockNumberOrTag,
    inspector: &mut TracingInspector,
) -> Evm<'a, &mut TracingInspector, CacheDB<AlloyDB<T, N, P>>> 
    where T: Clone + Transport, N: Network, P: Provider<T, N>
{
    let db = make_alloy_cached_db(provider, block_num);
    Evm::builder()
        .with_db(db)
        .with_external_context(inspector)
        .append_handler_register(inspector_handle_register)
        .build()
}

async fn make_env<T: Clone + Transport, N: Network, P: Provider<T, N>>(
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

fn handle_response(
    result: ResultAndState, 
    inspector: TracingInspector,
    tracing_opt: GethDebugTracingOptions,
) -> GethTrace {
    match tracing_opt.tracer {
        None => {
            // todo: rm exit reason
            let ( gas_used, output, .. ) = match result.result {
                ExecutionResult::Success { gas_used, output, reason, .. } => {
                    ( gas_used, Some(output), reason.into() )
                },
                ExecutionResult::Revert { gas_used, output } => {
                    ( gas_used, Some(Output::Call(output)), InstructionResult::Revert )
                },
                ExecutionResult::Halt { gas_used, reason } => {
                    ( gas_used, None, reason.into() )
                },
            };
            let return_val = output.map(|out| out.data().clone()).unwrap_or_default();
            let trace = inspector.into_geth_builder().geth_traces(
                gas_used, 
                return_val, 
                tracing_opt.config,
            );
            trace.into()
        },
        // todo: impl for CallTracer and StateDiffTracer and Mux tracer
        _ => unimplemented!(),
    }
}
