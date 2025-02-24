#[macro_export]
macro_rules! async_test {
    ($name:ident, $body:expr) => {
        #[tokio::test]
        async fn $name() {
            crate::test_config::setup_test_env();
            let services = crate::test_utils::TestServices::new().await;
            
            // Run the test
            let result = $body(&services).await;
            
            // Cleanup
            services.cleanup().await;
            
            // Propagate any test failures
            if let Err(e) = result {
                panic!("Test failed: {}", e);
            }
        }
    };
}

#[macro_export]
macro_rules! setup_test_db {
    () => {{
        use crate::test_utils::TestDb;
        TestDb::new().await
    }};
}

#[macro_export]
macro_rules! with_test_services {
    ($services:ident, $body:expr) => {{
        let $services = crate::test_utils::TestServices::new().await;
        let result = $body;
        $services.cleanup().await;
        result
    }};
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_utils::TestServices;

    async_test!(test_macro_works, |_services| async {
        Ok(())
    });

    #[tokio::test]
    async fn test_with_services_macro() {
        with_test_services!(services, async {
            assert!(services.database.pool.ping().await.is_ok());
            Ok::<_, Box<dyn std::error::Error>>(())
        })
        .await
        .unwrap();
    }
} 