use revm_inspectors::tracing::{TracingInspector, TracingInspectorConfig};
use alloy::{
    rpc::types::eth::{BlockNumberOrTag, Header, TransactionRequest}, 
    transports::Transport,
    providers::Provider,
    network::Network, 
};
use revm::{
    primitives::{CfgEnv, CfgEnvWithHandlerCfg, EnvWithHandlerCfg, SpecId, U256}, 
    db::{alloydb::AlloyDB, CacheDB}, 
    inspector_handle_register, 
    Evm 
};
use eyre::{OptionExt, Result};


pub fn make_inspector() -> TracingInspector {
    TracingInspector::new(TracingInspectorConfig::all())
}

pub fn make_evm_with_env<'a, 'b: 'a, T, N, P>(
    provider: &'b P,
    tx_request: &TransactionRequest,
    block_header: Header,
    inspector: &'a mut TracingInspector,
) -> Result<Evm<'a, &'a mut TracingInspector, CacheDB<AlloyDB<T, N, &'b P>>>> 
    where T: Clone + Transport, N: Network, P: Provider<T, N>
{
    let mut evm = make_evm(provider, block_header.number.unwrap().into(), inspector);
    let env = make_env_with_cfg_handler(tx_request, block_header)?;
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

fn make_env_with_cfg_handler(
    tx_request: &TransactionRequest,
    block_header: Header,
) -> Result<EnvWithHandlerCfg> {
    let block_env = crate::utils::blockheader_to_blockenv(&block_header)?;
    let base_fee = U256::wrapping_from(block_header.base_fee_per_gas.expect("Missing base fee"));
    let tx_env = crate::utils::txrequest_to_txenv(&tx_request, base_fee)?;

    let chain_id = tx_request.chain_id.unwrap_or_default();
    let cfgh = make_cfg_with_handler_cfg(chain_id);
    let env = EnvWithHandlerCfg::new_with_cfg_env(cfgh, block_env, tx_env);

    Ok(env)
}

fn make_alloy_cached_db<T: Clone + Transport, N: Network, P: Provider<T, N>>(
    provider: P, 
    block_num: BlockNumberOrTag,
) -> CacheDB<AlloyDB<T, N, P>> {
    let block_num = block_num.as_number().expect("Block number expected");
    let block_num_state = BlockNumberOrTag::Number(block_num - 1);
    let alloy_db = AlloyDB::new(provider, block_num_state.into());
    CacheDB::new(alloy_db)
}

fn make_cfg_with_handler_cfg(chain_id: u64) -> CfgEnvWithHandlerCfg {
    let mut cfg = CfgEnv::default();
    cfg.chain_id = chain_id;

    let mut cfgh = CfgEnvWithHandlerCfg::new_with_spec_id(cfg, SpecId::LATEST);
    cfgh.handler_cfg.is_optimism = chain_id == 10;

    cfgh
}