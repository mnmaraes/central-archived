use tokio_postgres::{ NoTls, Transaction, Error };
use dotenv::dotenv;
use std::env::var;

use super::store_creation_error::StoreCreationError;

pub struct Store {
    client: tokio_postgres::Client,
}

// Public Methods
impl Store {
    pub async fn get_default_store() -> Result<Store, StoreCreationError> {
        dotenv().ok();

        Self::get_store("DATABASE_URL").await
    }
}

// Private Methods
impl Store {
    async fn get_store(addr_var: &str) -> Result<Store, StoreCreationError> {
        let addr = match var(addr_var) {
            Ok(addr) => addr,
            Err(e) => return Err(StoreCreationError::VarError(e))
        };

        let (client, connection) = match tokio_postgres::connect(&addr, NoTls).await {
            Ok(t) => t,
            Err(e) => return Err(StoreCreationError::ConnectionError(e))
        };

        tokio::spawn(async move {
            if let Err(e) = connection.await {
                eprintln!("connection error: {}", e);
            }
        });

        Ok(Store {client})
    }
}


// Test Only Functions
impl Store {
    #[allow(dead_code)]
    async fn get_default_test_store() -> Result<Store, StoreCreationError> {
        dotenv().ok();

        Self::get_store("TEST_DATABASE_URL").await
    }

    #[allow(dead_code)]
    async fn set_up_test_database(& mut self) -> Result<(), StoreCreationError> {
        //1. Create Table
        let ct_stmt = self.client.prepare("CREATE TABLE cats (
                                                                  id SERIAL PRIMARY KEY,
                                                                  name VARCHAR NOT NULL,
                                                                  has_spots BOOLEAN NOT NULL
                                                                )").await?;

        self.client.execute(&ct_stmt, &[]).await?;

        // 2. Populate Table
        let pop_stmt = self.client.prepare("INSERT INTO cats(name, has_spots)
                                          VALUES 
                                                ('Berry', true),
                                                ('Lady', false),
                                                ('Bach', false),
                                                ('Simba', false),
                                                ('Lady', true)").await?;


        self.client.execute(&pop_stmt, &[]).await?;

        Ok(())
    }

    #[allow(dead_code)]
    async fn tear_down_test_database(& mut self) -> Result<(), StoreCreationError> {
        //1. Drop Table
        let drop_stmt = self.client.prepare("DROP TABLE cats").await?;

        self.client.execute(&drop_stmt, &[]).await?;

        Ok(())
    }

    #[allow(dead_code)]
    async fn count_test_cats_by_name<'a>(&'a mut self, name: &str) -> Result<i32, StoreCreationError> {
        let count_stmt = self.client.prepare("select count(*)::INT from cats where name=$1::TEXT")
            .await?;

        Ok(self.client.query_one(&count_stmt, &[&name])
            .await?
            .try_get(0)?)
    }       
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::runtime::Runtime;
    use core::future::Future;

    struct Cats {
        id: i32,
        name: String,

        has_spots: bool
    }

    #[test]
    fn can_create_default_store() {
        run_store_test(async {
            if let Err(e) = Store::get_default_test_store().await {
                assert!(false, "Store Creation Error: {}", e);
            }
        })               
    }

    #[test]
    fn can_access_db() {
        let result: Result<i32, StoreCreationError> = run_propped_store_tests(async {
            let mut store = Store::get_default_test_store().await?;

            Ok(store.count_test_cats_by_name("Berry").await?)
        }).expect("Set up Error");

        assert_eq!(result.unwrap(), 1);

        let result: Result<i32, StoreCreationError> = run_propped_store_tests(async {
            let mut store = Store::get_default_test_store().await?;

            Ok(store.count_test_cats_by_name("Lady").await?)
        }).expect("Set up Error");

        assert_eq!(result.unwrap(), 2);
    }

    // Test Setup
    fn run_propped_store_tests<F>(test: F) -> Result<F::Output, StoreCreationError>
        where F: Future
    {
        let mut rt = Runtime::new().unwrap();

        rt.block_on(async {
            // Setup
            let mut store = Store::get_default_test_store().await?;
            store.set_up_test_database().await?;

            // Test
            let result = test.await;

            // Teardown
             store.tear_down_test_database().await?;

            Ok(result)
        })
    }

    fn run_store_test<F>(test: F) 
        where F: Future 
    {
        let mut rt = Runtime::new().unwrap();

        rt.block_on(async {
            test.await;
        });
    }
}
