use alloy::{
    transports::{TransportResult, Transport, TransportErrorKind},
    rpc::types::{eth::{BlockNumberOrTag, TransactionRequest},
    trace::geth::{GethDebugTracingCallOptions, GethTrace}},
    providers::{Provider, RootProvider},
    network::Network,
};
use crate::tracer;


#[derive(Clone)]
pub struct LocalTraceProvider<T, N>
    where T: Transport + Clone, N: Network
{
    root_provider: RootProvider<T, N>
}

impl<T, N> LocalTraceProvider<T, N> 
    where T: Transport + Clone, N: Network
{
    pub fn new(root_provider: RootProvider<T, N>) -> Self {
        Self { root_provider }
    }

    pub async fn debug_trace_call(
        &self,
        tx: TransactionRequest,
        block: BlockNumberOrTag,
        trace_options: GethDebugTracingCallOptions,
    ) -> TransportResult<GethTrace> {
        tracer::geth_trace(&self, &tx, block, trace_options).await
            .map_err(|err| TransportErrorKind::custom_str(&err.to_string()))
    }
}

impl<T, N> Provider<T, N> for LocalTraceProvider<T, N> 
    where T: Transport + Clone, N: Network
{
    fn root(&self) -> &RootProvider<T, N> {
        &self.root_provider
    }
}