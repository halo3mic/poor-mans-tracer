use revm::primitives::{U256, BlockEnv, TxEnv, TransactTo};
use alloy::{
    rpc::types::eth::{TransactionRequest, Header},
    primitives::TxKind,
};
use eyre::{OptionExt, Result};


macro_rules! only_set_if_some {
    ($val1:expr, $val2:expr, $op:expr) => {
        if let Some(x) = $val2 {
            $val1 = $op(x);
        }
    };
    ($val1:expr, $val2:expr) => {
        if let Some(x) = $val2 {
            $val1 = x;
        }
    };
}

pub fn txrequest_to_txenv(tx_request: &TransactionRequest, base_fee: U256) -> Result<TxEnv> {
    let mut tx_env = TxEnv::default();
    tx_env.gas_price = tx_request.gas_price
        .map(U256::wrapping_from)
        .unwrap_or(U256::ZERO);
    if tx_env.gas_price == U256::ZERO {
        tx_env.gas_price = tx_request.max_priority_fee_per_gas
            .map(|x| U256::wrapping_from(x) + base_fee).ok_or_eyre("Missing gas pricing")?;
    }
    only_set_if_some!(tx_env.gas_limit, tx_request.gas, |x| x as u64);
    only_set_if_some!(tx_env.value, tx_request.value, U256::wrapping_from);
    only_set_if_some!(tx_env.data, tx_request.input.data.clone().or(tx_request.input.input.clone())); // todo: avoid cloning
    only_set_if_some!(tx_env.caller, tx_request.from);
    only_set_if_some!(tx_env.transact_to, tx_request.to, |x| match x {
        TxKind::Call(addr) => TransactTo::Call(addr),
        TxKind::Create => TransactTo::Create,
    });
    
    // todo: add access list
    tx_env.chain_id = tx_request.chain_id;
    tx_env.nonce = tx_request.nonce;

    Ok(tx_env)
}


pub fn blockheader_to_blockenv(block_header: &Header) -> Result<BlockEnv> {
    Ok(BlockEnv {
        basefee: U256::wrapping_from(block_header.base_fee_per_gas.ok_or_eyre("Missing base_fee_per_gas")?),
        timestamp: U256::wrapping_from(block_header.timestamp),
        difficulty: U256::wrapping_from(block_header.difficulty),
        number: U256::wrapping_from(block_header.number.ok_or_eyre("Missing block number")?),
        gas_limit: U256::wrapping_from(block_header.gas_limit),
        coinbase: block_header.miner,
        prevrandao: block_header.mix_hash,
        ..Default::default()
        // todo: blob_excess_gas_and_price
        // blob_excess_gas_and_price: {
        //     if let Some(excess_gas) = block_header.excess_blob_gas {
        //         let blob_gasprice = block_header.blob_fee().unwrap();
        //         Some(BlobExcessGasAndPrice { excess_blob_gas: excess_gas as u64, blob_gasprice })
        //     } else {
        //         None
        //     }

        // },
    })
}