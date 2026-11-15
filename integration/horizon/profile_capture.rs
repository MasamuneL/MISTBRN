fn allomancy_profile_capture(launch: &Launch) -> std::ops::ControlFlow<Result<()>> {
    let result = std::env::current_dir()
        .map_err(|_| "Allomancy owner root unavailable".to_owned())
        .and_then(|root| {
            horizon_client::save::allomancy_world_guard::capture_owner(
                &root.join("saves").join(&launch.folder),
                horizon_client::lan::playerdata::uuid_ints(launch.owner),
            )
        });
    match result {
        Ok(()) => std::ops::ControlFlow::Continue(()),
        Err(reason) => {
            log::warn!("Allomancy owner: context capture rejected");
            std::ops::ControlFlow::Break(Err(reason.into()))
        }
    }
}
