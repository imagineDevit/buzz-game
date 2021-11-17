#[macro_export]
macro_rules! routes {
    (post -> $path: expr, $service:ident, $game_info: ident, $f: expr) => {{
        path!("game" / $path)
            .and(post())
            .and(Routes::with_service($service))
            .and(Routes::with_game_info($game_info))
            .and(warp::body::json())
            .and_then($f)
    }};

    (get -> $service: ident, $game_info: ident, $f: expr) => {{
        path!("game")
            .and(get())
            .and(Routes::with_service($service))
            .and(Routes::with_game_info($game_info))
            .and(warp::query::<AddPlayerQuery>())
            .and_then($f)
    }};
}
