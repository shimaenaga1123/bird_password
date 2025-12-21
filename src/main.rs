use cfg_if::cfg_if;

cfg_if! {
    if #[cfg(feature = "ssr")] {
        use actix_web::*;
        use actix_files::Files;
        use leptos::*;
        use leptos_actix::{generate_route_list, LeptosRoutes};
        use bird_password::*;
        use polars::prelude::*;
        use std::fs::File;
        use std::io::Write;
        use std::path::Path;
        use tracing::{info, error, Level};
        use tracing_subscriber::FmtSubscriber;

        const CSV_PATH: &str = "birds.csv";
        const TXT_PATH: &str = "bird_names.txt";

        fn extract_csv_to_txt() -> anyhow::Result<()> {
            info!("Processing CSV to TXT...");
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
            info!("Extracted {} names to '{}'", count, TXT_PATH);
            Ok(())
        }

        #[actix_web::main]
        async fn main() -> std::io::Result<()> {
            FmtSubscriber::builder()
                .with_max_level(Level::INFO)
                .with_writer(std::io::stderr)
                .init();

            if !Path::new(TXT_PATH).exists() {
                if let Err(e) = extract_csv_to_txt() {
                    error!("Failed to extract data: {}", e);
                }
            }

            let conf = get_configuration(None).await.unwrap();
            let addr = conf.leptos_options.site_addr;
            let routes = generate_route_list(App);

            HttpServer::new(move || {
                let leptos_options = &conf.leptos_options;
                let site_root = &leptos_options.site_root;

                App::new()
                    .route("/api/{tail:.*}", leptos_actix::handle_server_fns())
                    .leptos_routes(leptos_options.to_owned(), routes.to_owned(), App)
                    .service(Files::new("/", site_root))
            })
            .bind(&addr)?
            .run()
            .await
        }
    } else {
        pub fn main() {
        }
    }
}
