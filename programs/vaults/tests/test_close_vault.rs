mod common;

use crate::common::cookies::init_new_test;

#[tokio::test(flavor = "multi_thread")]
async fn close_vault() {
    let mut test = init_new_test().await.ok().unwrap();
}
