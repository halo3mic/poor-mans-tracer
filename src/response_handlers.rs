use revm_inspectors::tracing::TracingInspector;
use alloy::rpc::types::trace::geth::{
    GethDebugBuiltInTracerType, 
    GethDebugTracingOptions,
    GethDebugTracerType,
    GethTrace,
};
use revm::{
    primitives::{ExecutionResult, Output, ResultAndState}, 
    interpreter::InstructionResult, 
    DatabaseRef, 
};
use eyre::Result;


pub fn handle_response<T>(
    result: ResultAndState,
    db: T,
    inspector: TracingInspector,
    tracing_opt: GethDebugTracingOptions,
) -> Result<GethTrace> 
where
    T: DatabaseRef,
    T::Error: 'static + Send + Sync + std::error::Error,
{
    match tracing_opt.tracer {
        None => handle_struct_trace(result, inspector, tracing_opt),
        Some(GethDebugTracerType::BuiltInTracer(built_in_tracer)) => {
            match built_in_tracer {
                GethDebugBuiltInTracerType::CallTracer => handle_call_trace(result, inspector, tracing_opt),
                GethDebugBuiltInTracerType::PreStateTracer => handle_prestate_trace(result, inspector, tracing_opt, db),
                _ => unimplemented!("The provided type of built-in tracer not supported yet"),
            }
        }, 
        Some(GethDebugTracerType::JsTracer(_)) => unimplemented!("Custom tracer not supported yet"),
    }
}

fn handle_struct_trace(
    result: ResultAndState,
    inspector: TracingInspector,
    tracing_opt: GethDebugTracingOptions,
) -> Result<GethTrace> {
    let (gas_used, output, _) = handle_execution_result(result.result);
    let return_val = output.map(|out| out.data().clone()).unwrap_or_default();
    let trace = inspector.into_geth_builder().geth_traces(
        gas_used, 
        return_val, 
        tracing_opt.config,
    );
    Ok(trace.into())
}

fn handle_call_trace(
    result: ResultAndState,
    inspector: TracingInspector,
    tracing_opt: GethDebugTracingOptions,
) -> Result<GethTrace> {
    let (gas_used, ..) = handle_execution_result(result.result);
    let call_config = tracing_opt.tracer_config
        .into_call_config()
        .map_err(|e| eyre::eyre!("Failed to convert config to call config: {}", e))?;
    let trace = inspector.into_geth_builder().geth_call_traces(
        call_config,
        gas_used,
    );
    Ok(trace.into())
}

fn handle_prestate_trace<T>(
    result: ResultAndState,
    inspector: TracingInspector,
    tracing_opt: GethDebugTracingOptions,
    db: T,
) -> Result<GethTrace> 
where
    T: DatabaseRef,
    T::Error: 'static + Send + Sync + std::error::Error,
{
    let prestate_config = tracing_opt.tracer_config
        .into_pre_state_config()
        .map_err(|e| eyre::eyre!("Failed to convert config to prestate config: {}", e))?;
    let trace = inspector.into_geth_builder().geth_prestate_traces(
        &result,
        prestate_config,
        db,
    )?;
    Ok(trace.into())
}

fn handle_execution_result(exe_result: ExecutionResult) -> (u64, Option<Output>, InstructionResult) {
    let (gas_used, output, halt_reason) = match exe_result {
        ExecutionResult::Success { gas_used, output, reason, .. } => {
            (gas_used, Some(output), reason.into())
        },
        ExecutionResult::Revert { gas_used, output } => {
            (gas_used, Some(Output::Call(output)), InstructionResult::Revert)
        },
        ExecutionResult::Halt { gas_used, reason } => {
            (gas_used, None, reason.into())
        },
    };
    (gas_used, output, halt_reason)
}