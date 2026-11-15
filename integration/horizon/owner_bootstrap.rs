fn allomancy_owner_bootstrap(
    level: &mut Level,
    world: &std::path::Path,
) -> std::ops::ControlFlow<Result<()>> {
    match allomancy_world_guard::before_level_write(world, level) {
        Ok(()) => std::ops::ControlFlow::Continue(()),
        Err(reason) => {
            log::warn!("Allomancy owner: protected write rejected");
            std::ops::ControlFlow::Break(Err(reason.into()))
        }
    }
}
