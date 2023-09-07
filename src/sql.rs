use postgres::{Client, NoTls};

pub fn setup(config: toml::Table) -> Client {
    let db_url = config.get("db_url").unwrap().as_str().unwrap();
    let mut client = Client::connect(db_url, NoTls).unwrap();
    client
        .execute(
            "CREATE TABLE IF NOT EXISTS auth (
            id VARCHAR(36),
            email VARCHAR(256),
            token VARCHAR(36)
        )",
            &[],
        )
        .unwrap();
    client
}
