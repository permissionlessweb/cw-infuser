

pub fn custom_mock_app() -> App {
    AppBuilder::new()
        .with_stargate(StargazeKeeper)
        .build(no_init)
}