use rocket::http::ContentType;
use rocket::http::Status;
use rocket::local::asynchronous::Client;
use std::fs::File;
use std::io::prelude::*;

#[rocket::async_test]
async fn template_responds_with_image() {
    tracing_subscriber::fmt().init();

    let client = Client::tracked(super::rocket().unwrap())
        .await
        .expect("valid rocket");

    let res = client
        .post("/template")
        .header(ContentType::JSON)
        .remote("127.0.0.1:8000".parse().unwrap())
        .body(
            r#"{"name": "test", "width": 1280, "height": 720, "ctx": {"name": "Bob", "age": 30 }}"#,
        )
        .dispatch()
        .await;

    assert_eq!(res.status(), Status::Ok);
    assert!(res.body().is_some());

    let bytes = res.into_bytes().await.unwrap();

    let mut file = File::create("test.png").unwrap();
    file.write_all(&bytes).unwrap();
}
