
// testing endp from our main.rs
//#[cfg(test)]
//mod tests {
    //use crate::health_check;

    //#[tokio::test] 
    //async fn h_c_succeeds() {
        //let response = health_check().await;
        //assert!(response.status().is_success())
    //}
//}


#[tokio::test] 
async fn healt_check_works() {
    spawn_app();
    let client = reqwest::Client::new();

    // Act
    let response = client 
        .get("http://127.0.0.1:8000/health_check")
        .send()
        .await
        .expect("Failed to execute request");
    assert!(response.status().is_success());
}

// No .await call, therefore no need for `spawn_app` to be async now.
// We are also running tests, so it is not worth it to propagate errors:
// if we fail to perform the required setup we can just panic and crash 
// all the things.
fn spawn_app() {
    let server = zero2prod::run().expect("Failed to bind address");
    let _ = tokio::spawn(server);
}
