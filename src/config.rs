pub(crate) mod app;
pub(crate) mod db;

#[cfg(test)]
mod tests {

    use super::app::*;
    use crate::errors::error::CustomError;
    use rstest::*;

    #[fixture]
    async fn init_config_fixture() -> Result<AppConfig, crate::errors::error::CustomError> {
        init_config().await
    }

    #[rstest(init_config_fixture as config)]
    async fn init_config_test(#[future] config: Result<AppConfig, CustomError>) {
        let conf: Result<AppConfig, CustomError> = config.await;

        assert!(conf.is_ok());

        let app_config = conf.unwrap();

        assert_eq!("buzz-game-test".to_string(), app_config.name);

        assert_eq!(
            "postgres://postgres:hsedjame@localhost:5432/buzzz".to_string(),
            app_config.db.to_string()
        )
    }
}
