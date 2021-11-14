#[macro_export]
macro_rules! routes {
    (post -> $path: expr, $service:ident, $f: expr) => {{
        path!("game" / $path)
            .and(post())
            .and(Routes::with_service($service))
            .and(warp::body::json())
            .and_then($f)
    }};

    (get -> $service: ident, $f: expr) => {{
        path!("game")
            .and(get())
            .and(Routes::with_service($service))
            .and(warp::query::<AddPlayerQuery>())
            .and_then($f)
    }};
}
