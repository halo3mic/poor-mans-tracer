use revm_inspectors::tracing::{TracingInspector, TracingInspectorConfig};
use alloy::{
    rpc::types::eth::{BlockNumberOrTag, TransactionRequest},
    providers::Provider,
    transports::Transport,
    network::Network,
};
use revm::{
    primitives::{
        CfgEnv, CfgEnvWithHandlerCfg, EnvWithHandlerCfg, SpecId, U256
    }, 
    db::{alloydb::AlloyDB, CacheDB}, 
    inspector_handle_register, 
    Evm 
};
use eyre::Result;


pub fn make_inspector() -> TracingInspector {
    TracingInspector::new(TracingInspectorConfig::all())
}

pub async fn make_evm_with_env<'a, 'b: 'a, T, N, P>(
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

fn make_alloy_cached_db<T: Clone + Transport, N: Network, P: Provider<T, N>>(
    provider: P, 
    block_num: BlockNumberOrTag,
) -> CacheDB<AlloyDB<T, N, P>> {
    let block_num_state = BlockNumberOrTag::Number(block_num.as_number().unwrap() - 1); // todo: consider if this is okay to be here
    let alloy_db = AlloyDB::new(provider, block_num_state.into());
    CacheDB::new(alloy_db)
}

async fn make_env_with_cfg_handler<T: Clone + Transport, N: Network, P: Provider<T, N>>(
    provider: P,
    tx_request: TransactionRequest,
    block_number: BlockNumberOrTag,
) -> Result<EnvWithHandlerCfg> {
    let block = provider.get_block_by_number(block_number, false).await?.unwrap();
    let block_env = crate::utils::blockheader_to_blockenv(&block.header)?;
    let base_fee = U256::wrapping_from(block.header.base_fee_per_gas.expect("Missing base fee"));
    let tx_env = crate::utils::txrequest_to_txenv(&tx_request, base_fee)?;

    let cfgh = CfgEnvWithHandlerCfg::new_with_spec_id(CfgEnv::default(), SpecId::LATEST);
    let env = EnvWithHandlerCfg::new_with_cfg_env(cfgh, block_env, tx_env);

    Ok(env)
}