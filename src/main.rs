use color_eyre::eyre::Result;
use reqwest;
use sqlx::postgres::PgPoolOptions;
use tracing::{error, info, Level};
use tracing_subscriber::FmtSubscriber;
//use tokio_stream::StreamExt;
use again::RetryPolicy;
use futures::stream::StreamExt;
use once_cell::sync::Lazy;
use std::time::Duration;

#[derive(sqlx::FromRow)]
struct ImageInfo {
    slug: String,
    id: i32,
}

static POLICY: Lazy<RetryPolicy> = Lazy::new(|| {
    RetryPolicy::exponential(Duration::from_millis(100))
        .with_max_delay(Duration::from_secs(10))
        .with_jitter(false)
});

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // benchmark
    let start = std::time::Instant::now();

    color_eyre::install()?;
    let subscriber = FmtSubscriber::builder()
        // all spans/events with a level higher than TRACE (e.g, debug, info, warn, etc.)
        // will be written to stdout.
        .with_max_level(Level::ERROR)
        // completes the builder.
        .finish();

    tracing::subscriber::set_global_default(subscriber).expect("setting default subscriber failed");
    // todo get pkmn slug from database
    // create urls using that name
    // try to get each url successively
    // save to a file

    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect("postgres://postgres:postgres@localhost/champs")
        .await
        .unwrap();
    let _ = tokio::fs::create_dir("images").await;

    //let all_pokemon_info = get_image_info(&pool);
    let all_pokemon_info = sqlx::query_as::<_, ImageInfo>(
        "
    SELECT pokemon_name AS slug, pokemon_id AS id FROM pokemon;
        ",
    )
    .fetch(&pool);

    //let test : Option<ImageInfo> = all_pokemon_info.next().await;

    all_pokemon_info
        .for_each_concurrent(40, process_pokemon)
        .await;

    println!("Complete in {:#?}", start.elapsed());

    Ok(())
}

async fn process_pokemon(info: Result<ImageInfo, sqlx::Error>) {
    match info {
        Err(e) => {
            error!("Sql error {}", e);
        }
        Ok(data) => {
            let urls = create_url_list(&data);
            match fetch_image(urls).await {
                Err(e) => {
                    error!("No images found at any url: {:?}", e);
                }
                Ok(response) => match save_image(&data, response).await {
                    Err(e) => {
                        error!("The image could not be saved: {}", e);
                    }
                    Ok(()) => {}
                },
            }
        }
    }
}

fn _get_image_info<'a>(pool: &'a sqlx::Pool<sqlx::Postgres>) -> impl StreamExt + 'a {
    sqlx::query_as::<_, ImageInfo>(
        "
    SELECT pokemon_name, pokemon_id FROM pokemon;
        ",
    )
    .fetch(pool)
}

fn create_url_list(image_info: &ImageInfo) -> Vec<String> {
    let gen09 = format!(
        "https://img.pokemondb.net/sprites/scarlet-violet/normal/{}.png",
        image_info.slug
    );
    let bdsp = format!(
        "https://img.pokemondb.net/sprites/brilliant-diamond-shining-pearl/normal/{}.png",
        image_info.slug
    );
    let bank = format!(
        "https://img.pokemondb.net/sprites/home/normal/{}.png",
        image_info.slug
    );

    Vec::from([gen09, bdsp, bank])
}

async fn fetch_image(urls: Vec<String>) -> Result<reqwest::Response, Vec<reqwest::Error>> {
    let mut iter = urls.into_iter();
    let mut errors = Vec::new();
    while let Some(url) = iter.next() {
        match POLICY
            .retry(|| reqwest::get(&url))
            .await
            .and_then(|resp| resp.error_for_status())
        {
            Ok(image) => {
                for e in errors {
                    info!("url {} did not work\nfallback used", e);
                }
                return Ok(image);
            }
            Err(e) => {
                errors.push(e);
            }
        }
    }
    Err(errors)
}

async fn save_image(
    image_info: &ImageInfo,
    resp: reqwest::Response,
) -> Result<(), Box<dyn std::error::Error>> {
    let filename = format!("images/{}.png", image_info.id);
    let mut file = tokio::fs::File::create(filename).await?;
    let mut byte_stream = resp.bytes_stream();

    while let Some(item) = byte_stream.next().await {
        tokio::io::copy(&mut item?.as_ref(), &mut file).await?;
    }
    Ok(())
}
