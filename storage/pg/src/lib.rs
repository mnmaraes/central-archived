pub mod storage {
    pub trait Storable {
        type IdType;
    }

    pub trait Storage {}

    pub enum Stored<'s, T: Storable, S: Storage> {
        Pointer(T::IdType, &'s S),
        Value(T),
        Error(String),
    }

    pub mod async_pg {
        use tokio_postgres::NoTls;
        use dotenv::dotenv;

        use std::fmt;
        use std::env::{var, VarError};
        use std::error::Error;

        pub struct Store {
            client: tokio_postgres::Client,
        }

        #[derive(Debug)]
        pub enum StoreCreationError {
            VarError(VarError),
            ConnectionError(tokio_postgres::error::Error)
        }

        impl fmt::Display for StoreCreationError {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                match self {
                    StoreCreationError::VarError(e) => e.fmt(f),
                    StoreCreationError::ConnectionError(e) => e.fmt(f)
                }
            }
        }

        impl Error for StoreCreationError {
            fn description(&self) -> &str {
                match self {
                    StoreCreationError::VarError(e) => e.description(),
                    StoreCreationError::ConnectionError(e) => e.description()
                }
            }

            fn source(&self) -> Option<&(dyn Error + 'static)> {
                match self {
                    StoreCreationError::VarError(e) => Some(e),
                    StoreCreationError::ConnectionError(e) => Some(e)
                }
            }

        }

        impl Store {
            pub async fn get_default_store() -> Result<Store, StoreCreationError> {
                dotenv().ok();

                let addr = match var("DATABASE_URL") {
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

        #[cfg(test)]
        mod tests {
            use super::*;
            use tokio::runtime::Runtime;

            #[test]
            fn can_create_default_store() {
                let mut rt = Runtime::new().unwrap();

                rt.block_on(async {
                    if let Err(e) = Store::get_default_store().await {
                        assert!(false, "Store Creation Error: {}", e);
                    }
                })               
            }
        }
    }

    #[cfg(test)]
    mod tests {
        #[test]
        fn it_works() {
            assert_eq!(2 + 2, 4);
        }
    }
}
