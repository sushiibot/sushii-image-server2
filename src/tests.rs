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
            r#"{
                "name": "rank",
                "width": 500,
                "height": 400,
                "ctx": {
                    "CONTENT_COLOR": "0, 184, 148",
                    "CONTENT_OPACITY": "1",
                    "AVATAR_URL": "https://cdn.discordapp.com/avatars/150443906511667200/afaddec4029eafd36e30fb62efe7bfad.png",
                    "REP": "123",
                    "FISHIES": "456",
                    "USERNAME": "test#1234",
                    "PATRON_EMOJI": "",
                    "XP_PROGRESS": "75",
                    "LEVEL": "10",
                    "GLOBAL_LEVEL": "15",
                    "CURR_LEVEL_XP": "125",
                    "LEVEL_XP_REQ": "1200",
                    "DAILY": "60 / 67",
                    "WEEKLY": "87 / 892",
                    "MONTHLY": "113 / 5139",
                    "ALL": "489 / 33067"
                }
            }"#
        )
        .dispatch()
        .await;

    assert_eq!(res.status(), Status::Ok);
    assert!(res.body().is_some());

    let bytes = res.into_bytes().await.unwrap();

    let mut file = File::create("test.png").unwrap();
    file.write_all(&bytes).unwrap();
}
