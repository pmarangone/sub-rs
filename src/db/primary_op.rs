use anyhow::Error;
use postgres::{Client, NoTls};
use tracing::info;

use crate::models::user::GeneralParams;

pub fn create_connection() -> Result<postgres::Client, Error> {
    let mut client = Client::connect(
        "host=localhost port=5432 dbname=database user=user password=password",
        NoTls,
    )?;
    Ok(client)
}

pub fn create_table(client: &mut Client) -> Result<(), Error> {
    client.batch_execute(
        "
    CREATE TABLE IF NOT EXISTS person (
        id      SERIAL PRIMARY KEY,
        name    TEXT NOT NULL,
        surname TEXT NOT NULL,
        description TEXT NOT NULL,
        age INTEGER NOT NULL
    )
    ",
    )?;

    Ok(())
}

pub fn insert_into_table(mut client: Client, user: GeneralParams) -> Result<(), Error> {
    let age: i32 = user.age as i32;
    client.execute(
        "INSERT INTO person (name, surname, description, age) VALUES ($1, $2, $3, $4)",
        &[&user.name, &user.surname, &user.description, &age],
    )?;

    Ok(())
}

pub fn query_all(mut client: Client) -> Result<(), Error> {
    for row in client.query("SELECT id, name, surname FROM person", &[])? {
        let id: i32 = row.get(0);
        let name: &str = row.get(1);
        let surname: &str = row.get(2);

        info!("{} {} {}", id, name, surname);
        // let data: Option<&[u8]> = row.get(2);
    }

    Ok(())
}
