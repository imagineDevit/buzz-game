pub(crate) mod db;
pub(crate) mod entities;
pub(crate) mod repositories;
pub(crate) mod utils;

#[cfg(test)]
mod tests {
    use super::repositories::*;
    use crate::config::app::init_config;
    use crate::data::db::{clear_db, create_db_pool, get_connection, init_db};
    use crate::data::entities::Player;
    use rstest::*;

    #[fixture]
    async fn repository() -> PlayerRepository {
        let config = init_config().await.unwrap();
        let pool = create_db_pool(&config).unwrap().clone();
        let pool_clone = pool.clone();

        tokio::spawn(async move {
            let connection = get_connection(&pool_clone).await.unwrap();
            clear_db(&connection).await.unwrap();
            init_db(&connection).await.unwrap();
        })
        .await
        .unwrap();

        PlayerRepository::new(pool.clone())
    }

    #[fixture(name = "Joe".to_string())]
    fn player(name: String) -> Player {
        Player::with_name(name)
    }

    #[rstest]
    #[case("Audrey".to_string())]
    #[trace]
    async fn insert_test(
        #[future]
        #[notrace]
        repository: PlayerRepository,
        #[with("Audrey".to_string())] player: Player,
        #[case] name: String,
    ) {
        let repo: PlayerRepository = repository.await;

        let result = repo.insert(&player).await;

        assert!(result.is_ok());

        let saved_player = result.unwrap();

        assert_eq!(name, saved_player.name);

        assert_eq!(0, saved_player.score);
    }

    #[rstest]
    #[should_panic]
    #[trace]
    async fn insert_test_fails(
        #[future]
        #[notrace]
        repository: PlayerRepository,
        #[with("Karl".to_string())] player: Player,
    ) {
        let repo: PlayerRepository = repository.await;

        let _ = repo.insert(&player).await.unwrap();
        let _ = repo.insert(&player).await.unwrap();
    }

    #[rstest]
    #[case("Chlo".to_string())]
    #[trace]
    async fn exist_by_test(
        #[future]
        #[notrace]
        repository: PlayerRepository,
        #[with("Chlo".to_string())] player: Player,
        #[case] name: String,
    ) {
        let repo: PlayerRepository = repository.await;

        let _ = repo.insert(&player).await.unwrap();

        let exist = repo.exist_by(SearchAttributes::Name(name)).await.unwrap();

        assert!(exist);
    }

    #[rstest]
    #[case("Chlo".to_string())]
    #[trace]
    async fn not_exist_by_test(
        #[future]
        #[notrace]
        repository: PlayerRepository,
        #[case] name: String,
    ) {
        let repo: PlayerRepository = repository.await;

        let exist = repo.exist_by(SearchAttributes::Name(name)).await.unwrap();

        assert!(!exist);
    }

    #[rstest]
    #[case("Armel".to_string())]
    #[trace]
    async fn find_by_test(
        #[future]
        #[notrace]
        repository: PlayerRepository,
        #[with("Armel".to_string())] player: Player,
        #[case] name: String,
    ) {
        let repo: PlayerRepository = repository.await;
        repo.insert(&player).await.unwrap();
        let option = repo.find_by(SearchAttributes::Name(name)).await.unwrap();
        assert!(option.is_some());
    }
}
