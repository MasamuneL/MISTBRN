fn allomancy_preflight(
    request: &WorldRequest,
    options: &Options,
) -> std::ops::ControlFlow<Result<PreparedWorld>> {
    use horizon_client::save::allomancy_world_guard::{self, OwnerMode};
    let mode = match options.execution_role {
        crate::game::execution_role::ExecutionRole::IntegratedServer => OwnerMode::Integrated,
        crate::game::execution_role::ExecutionRole::DedicatedServer => OwnerMode::Dedicated,
        _ => OwnerMode::Unsupported,
    };
    match allomancy_world_guard::validate_world(&request.save_root, mode) {
        Ok(()) => {
            log::info!("Allomancy preflight: accept");
            std::ops::ControlFlow::Continue(())
        }
        Err(reason) => {
            log::warn!("Allomancy preflight: reject");
            std::ops::ControlFlow::Break(Err(reason.into()))
        }
    }
}
