#[macro_export]
macro_rules! error {
    (game -> $game: ident, message -> $msg: expr, status -> $status_code: expr) => {{
        $game.send(Messages::Error { message: $msg }).await;

        Ok(Response::Error {
            message: $msg,
            code: $status_code,
        })
    }};
}
