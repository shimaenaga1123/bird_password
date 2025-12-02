use actix_cors::Cors;
use actix_web::*;
use polars::prelude::*;
use rand::seq::SliceRandom;
use rand::Rng;
use std::fs::File;
use std::io::{BufRead, BufReader, Write};
use std::path::Path;
use std::sync::Arc;
use tracing::{error, info, Level, Span};
use tracing_actix_web::{RootSpanBuilder, TracingLogger};
use tracing_subscriber::FmtSubscriber;

const CSV_PATH: &str = "birds.csv";
const TXT_PATH: &str = "bird_names.txt";

struct AppState {
    bird_names: Vec<String>,
}

struct CustomRootSpanBuilder;

impl RootSpanBuilder for CustomRootSpanBuilder {
    fn on_request_start(request: &dev::ServiceRequest) -> Span {
        let peer_ip = request
            .connection_info()
            .peer_addr()
            .unwrap_or("unknown")
            .to_string();

        let real_ip = request
            .headers()
            .get("CF-Connecting-IP")
            .or_else(|| request.headers().get("X-Forwarded-For"))
            .and_then(|v| v.to_str().ok())
            .map(|s| s.split(',').next().unwrap_or(s).trim().to_string())
            .unwrap_or(peer_ip);

        tracing::info_span!(
            "request",
            method = %request.method(),
            path = %request.path(),
            ip = %real_ip,
        )
    }

    fn on_request_end<B: actix_web::body::MessageBody>(
        span: Span,
        outcome: &Result<dev::ServiceResponse<B>, actix_web::Error>,
    ) {
        match outcome {
            Ok(response) => match response.status().as_u16() {
                200 => tracing::info!("completed"),
                _ => tracing::warn!("completed with status {}", response.status()),
            },
            Err(error) => {
                tracing::error!(error = %error, "failed");
            }
        }
    }
}

fn extract_csv_to_txt() -> anyhow::Result<()> {
    info!("CSV 파일을 처리하여 텍스트 파일로 변환합니다...");

    let df = LazyCsvReader::new(CSV_PATH)
        .finish()?
        .select([col("English name")])
        .collect()?;

    let english_names = df.column("English name")?.str()?;

    let mut file = File::create(TXT_PATH)?;

    let mut count = 0;
    for name_opt in english_names {
        if let Some(name) = name_opt {
            if name.len() < 3 || name.len() > 15 || name.contains("(") {
                continue;
            }

            let clean_name = name.replace(" ", "").replace("-", "").replace("'", "");

            writeln!(file, "{}", clean_name)?;
            count += 1;
        }
    }

    info!(
        "총 {}개의 새 이름을 추출하여 '{}'에 저장했습니다.",
        count, TXT_PATH
    );
    Ok(())
}

fn load_words_from_txt() -> anyhow::Result<Vec<String>> {
    let file = File::open(TXT_PATH)?;
    let reader = BufReader::new(file);

    let words: Result<Vec<_>, _> = reader.lines().collect();
    Ok(words?)
}

#[get("/generate")]
async fn generate_password(data: web::Data<Arc<AppState>>) -> impl Responder {
    let mut rng = rand::thread_rng();

    let selected_words: Vec<&String> = data.bird_names.choose_multiple(&mut rng, 4).collect();

    let mut parts = Vec::new();

    for word in selected_words {
        let digit = rng.gen_range(0..10);
        let part = format!("{}{}", word, digit);
        parts.push(part);
    }

    let password = parts.join("-");

    HttpResponse::Ok().body(password)
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    FmtSubscriber::builder()
        .pretty()
        .with_file(false)
        .with_line_number(false)
        .with_max_level(Level::INFO)
        .with_target(false)
        .with_writer(std::io::stderr)
        .init();

    if !Path::new(TXT_PATH).exists() {
        if let Err(e) = extract_csv_to_txt() {
            error!("데이터 추출 실패: {}", e);
            return Ok(());
        }
    }

    let birds = match load_words_from_txt() {
        Ok(data) => {
            info!("메모리에 {}개의 단어를 로드했습니다.", data.len());
            data
        }
        Err(e) => {
            error!("텍스트 파일 로드 실패: {}", e);
            return Ok(());
        }
    };

    let app_state = Arc::new(AppState { bird_names: birds });

    HttpServer::new(move || {
        let cors = Cors::default()
            .allow_any_origin()
            .allowed_methods(vec!["GET"])
            .max_age(86400);
        App::new()
            .wrap(cors)
            .wrap(TracingLogger::<CustomRootSpanBuilder>::new())
            .app_data(web::Data::new(app_state.clone()))
            .service(generate_password)
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}
