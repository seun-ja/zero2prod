use std::net::TcpListener;

use crate::{
    authentication::reject_anonymous_users,
    configuration::{DatabaseSettings, Settings},
    email_client::EmailClient,
    route::{
        admin_dashboard, change_password, change_password_form, health_check::health_check,
        home::home, log_out, login, login_form, newsletters::publish_newsletter,
        subscriptions::subscribe, subscriptions_confirm::confirm,
    },
};
use actix_session::{storage::RedisSessionStore, SessionMiddleware};
use actix_web::{
    cookie::Key,
    dev::Server,
    web::{self, Data},
    App, HttpServer,
};
use actix_web_flash_messages::{storage::CookieMessageStore, FlashMessagesFramework};
use actix_web_lab::middleware::from_fn;
use secrecy::{ExposeSecret, Secret};
use sqlx::{postgres::PgPoolOptions, PgPool};
use tracing_actix_web::TracingLogger;

pub struct Application {
    port: u16,
    server: Server,
}

impl Application {
    pub async fn build(configuration: Settings) -> Result<Self, std::io::Error> {
        let connection_pool = get_connection_pool(&configuration.database)
            .await
            .expect("Failed to connect to Postgres.");

        let sender_email = configuration
            .email_client
            .sender()
            .expect("Invalid sender email address");

        let timeout = configuration.email_client.timeout();

        let email_client = EmailClient::new(
            configuration.email_client.base_url,
            sender_email,
            configuration.email_client.authorization_token,
            timeout,
        );

        let address = format!(
            "{}:{}",
            configuration.application.host, configuration.application.port
        );
        let listener = TcpListener::bind(address).expect("Failed to bind random port");
        let port = listener.local_addr().unwrap().port();
        let server = run(
            listener,
            connection_pool,
            email_client,
            configuration.application.base_url,
            configuration.application.hmac_secret,
            configuration.redis_uri,
        )
        .await
        .unwrap();

        Ok(Self { port, server })
    }

    pub fn port(&self) -> u16 {
        self.port
    }

    pub async fn run_until_stopped(self) -> Result<(), std::io::Error> {
        self.server.await
    }
}

pub async fn get_connection_pool(configuration: &DatabaseSettings) -> Result<PgPool, sqlx::Error> {
    PgPoolOptions::new()
        .acquire_timeout(std::time::Duration::from_secs(2))
        .connect_with(configuration.with_db())
        .await
}

pub async fn run(
    listener: TcpListener,
    db_pool: PgPool,
    email_client: EmailClient,
    base_url: String,
    hmac_secret: Secret<String>,
    redis_uri: Secret<String>,
) -> Result<Server, anyhow::Error> {
    let db_pool = Data::new(db_pool);
    let email_client = Data::new(email_client);
    let base_url = Data::new(ApplicationBaseUrl(base_url));
    let secret_key = Key::from(b"02 6a 0e ea 12 9d 4f 58 92 15 f9 16 a0 0b 34 09 07 e1 ed 4e b5 4d 45 43 eb 39 bd fd 7a dd 9b 23 73 e7 a1 4d 32 41 b0 18 4b ca 98 18 55 11 ee 7b 77 a3 d3 40 7b f0 a6 b6 f3 44 a1 85 9b 1a 53 7a 3c 3e ee b1 f2 56 26 7e f5 9b 42 6c ce c1 3d 14 a3 66 36 3e 55 ef 7f 6a 2a b3 b3 fa b2 e0 f8 45 bf 5c da 3c 32 b9 b0 73 f2 48 f7 c0 dc ec ef 7b b1 a9 ed 09 72 6a 0a e8 cb fb 89 ba f5 f3 31 bc 78 5e c2 fb 6b fe 03 ed eb 44 a5 e5 c6 2d af bf 0d c4 5f e6 25 99 61 71 69 06 94 54 91 df b1 fd a1 a2 8b 7c ae 6f 50 4f 4f fe 20 68 c1 bc f0 b7 46 00 2c a0 a0 b1 42 82 81 24 f2 48 12 8d 27 af 6e 3a bc 37 7d 17 b2 3c 8c d1 18 32 7f 61 07 f6 eb 01 da 1c 93 cf 2f 95 e9 67 19 e4 be 28 3a 5c 2e f1 a9 ae e2 64 09 dc 5e fc d4 0d 8c 21 97 93 4c 7a 19 38 87 18 b2 c4 60 6e 2e 11 61 22 ac ec 68 73 1d 62 8f 89 13 de c2 b9 a5 0d b0 66 fc 1f 56 4b 1b 86 32 60 f7 b7 fc d8 e5 b8 bb df 5e 74 d8 37 c5 66 a5 5a 79 f3 74 e5 bf a4 40 e5 fb 21 c6 96 12 75 e9 d5 c1 06 97 4e 81 20 15 26 cb 32 be c3 65 20 d5 3f f2 84 9c 11 90 3e 2c 26 98 99 3b 52 69 48 f8 43 f2 4f bc 32 b9 12 01 e9 34 a6 81 f2 01 70 fb 17 72 6a 2a ec 9a 7f f0 25 5c 03 19 d1 95 98 e2 d0 93 b5 fa f0 a6 1b 9b e2 35 93 59 7b a9 ce 17 27 e1 60 25 06 dc c9 0a 79 27 99 2c 82 6f ee 66 30 a0 f1 db 17 a1 ce 12 a6 5e f9 dd 4b 58 23 ba 1d 86 19 9f 45 5e 92 1a 52 ff 62 0b 1c 70 c7 7f ff d3 f0 da b3 13 97 3b 9d 30 56 03 ea 2e 12 44 94 e7 d1 5d 38 b6 96 59 da 86 f3 79 b7 d8 99 64 9f 98 6f b3 ea 25 07 e7 b4 0c ff af 6f 6f 8e bb fb 7b 29 c8 0f a7 7f 6b 4a 56 49 67 97 d7 a3 e0 6c b8 3c ef aa a8 f3 80 b9 3e c4 
    ");    
    let message_store = CookieMessageStore::builder(secret_key.clone()).build();
    let message_framework = FlashMessagesFramework::builder(message_store).build();
    let redis_store = RedisSessionStore::new(redis_uri.expose_secret()).await?;
    let server = HttpServer::new(move || {
        App::new()
            .wrap(message_framework.clone())
            .wrap(SessionMiddleware::new(
                redis_store.clone(),
                secret_key.clone(),
            ))
            .wrap(TracingLogger::default())
            .route("/", web::get().to(home))
            .route("/login", web::get().to(login_form))
            .route("/login", web::post().to(login))
            .route("/health_check", web::get().to(health_check))
            .route("/subscriptions", web::post().to(subscribe))
            .route("/subscriptions/confirm", web::get().to(confirm))
            .route("/newsletters", web::post().to(publish_newsletter))
            .service(
                web::scope("/admin")
                    .wrap(from_fn(reject_anonymous_users))
                    .route("/dashboard", web::get().to(admin_dashboard))
                    .route("/password", web::get().to(change_password_form))
                    .route("/password", web::post().to(change_password))
                    .route("/logout", web::post().to(log_out)),
            )
            .app_data(db_pool.clone())
            .app_data(email_client.clone())
            .app_data(base_url.clone())
            .app_data(Data::new(HmacSecret(hmac_secret.clone())))
    })
    .listen(listener)?
    .run();

    Ok(server)
}

pub struct ApplicationBaseUrl(pub String);

#[derive(Clone)]
pub struct HmacSecret(pub Secret<String>);
