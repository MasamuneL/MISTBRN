fn allomancy_preflight(request: &WorldRequest) -> std::ops::ControlFlow<Result<PreparedWorld>> {
    match allomancy_world_guard::validate_world(&request.save_root) {
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
