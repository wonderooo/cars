use clap::Parser;

mod cli {
    use clap::{Parser, Subcommand};

    #[derive(Parser)]
    #[command(
        name = "cars app manager",
        about = "Cli tool for managing state of cars app services including: kafka, postgres and minio"
    )]
    pub(crate) struct Args {
        #[command(subcommand)]
        pub command: Command,
    }

    #[derive(Subcommand)]
    pub(crate) enum Command {
        Kafka {
            #[clap(subcommand)]
            cmd: KafkaCommand,
        },
        Postgres {
            #[clap(subcommand)]
            cmd: PostgresCommand,
        },
        Minio {
            #[clap(subcommand)]
            cmd: MinioCommand,
        },
    }

    #[derive(Subcommand)]
    pub(crate) enum KafkaCommand {
        DeleteTopics,
        CreateTopics,
        RecreateTopics,
        CreateAbsentTopics,
    }

    #[derive(Subcommand)]
    pub(crate) enum PostgresCommand {
        Migrate,
        RevertAll,
        Redo,
    }

    #[derive(Subcommand)]
    pub(crate) enum MinioCommand {
        CreateBucket,
        DeleteBucket,
        RecreateBucket,
        CreateAbsentBucket,
    }
}

mod kafka {
    use common::config::CONFIG;
    use common::kafka::KafkaAdmin;

    const TOPICS_WITHOUT_OPTS: &[&str] = &[
        "copart_cmd_lot_search",
        "copart_cmd_lot_images",
        "copart_response_lot_search",
        "copart_response_lot_images",
        "copart_cmd_auction",
    ];

    const TOPICS_WITH_OPTS: &[(&str, &[(&str, &str)])] = &[(
        "copart_response_lot_image_blobs",
        &[
            ("max.message.bytes", "100000000"),
            ("retention.ms", "1800000"),
        ],
    )];

    pub(crate) async fn crate_topics() {
        println!("Creating topics");
        let admin = KafkaAdmin::new(CONFIG.kafka.url.to_owned());
        for topic in TOPICS_WITHOUT_OPTS {
            admin
                .create_topic(topic)
                .await
                .expect("failed to create topic");
        }

        for (topic, opts) in TOPICS_WITH_OPTS {
            admin
                .create_topic_with_options(topic, &opts.iter().cloned().collect())
                .await
                .expect("failed to create topic");
        }
        println!("Topics created");
    }

    pub(crate) async fn delete_topics() {
        println!("Deleting topics");
        let admin = KafkaAdmin::new(CONFIG.kafka.url.to_owned());
        for topic in TOPICS_WITHOUT_OPTS {
            admin
                .delete_topic(topic)
                .await
                .expect("failed to delete topic");
        }

        for (topic, _) in TOPICS_WITH_OPTS {
            admin
                .delete_topic(topic)
                .await
                .expect("failed to delete topic");
        }
        println!("Topics deleted");
    }

    pub(crate) async fn recrate_topics() {
        println!("Recreating topics");
        let admin = KafkaAdmin::new(CONFIG.kafka.url.to_owned());
        for topic in TOPICS_WITHOUT_OPTS {
            admin
                .recreate_topic(topic)
                .await
                .expect("failed to delete topic");
        }

        for (topic, opts) in TOPICS_WITH_OPTS {
            admin
                .recreate_topic_with_opts(topic, &opts.iter().cloned().collect())
                .await
                .expect("failed to delete topic");
        }
        println!("Topics recreated");
    }

    pub(crate) async fn create_absent_topics() {
        println!("Creating absent topics");
        let admin = KafkaAdmin::new(CONFIG.kafka.url.to_owned());
        for topic in TOPICS_WITHOUT_OPTS {
            admin
                .create_absent_topic(topic)
                .await
                .expect("failed to delete topic");
        }

        for (topic, opts) in TOPICS_WITH_OPTS {
            admin
                .create_absent_topic_with_opts(topic, &opts.iter().cloned().collect())
                .await
                .expect("failed to delete topic");
        }
        println!("Absent topics created");
    }
}

mod postgres {
    use common::persistence::PG_POOL;
    use diesel_async::AsyncMigrationHarness;
    use diesel_migrations::{embed_migrations, EmbeddedMigrations, MigrationHarness};

    pub const PG_MIGRATIONS: EmbeddedMigrations = embed_migrations!("src/persistence/migrations");

    pub(crate) async fn migrate() {
        println!("Running migrations");
        let conn = PG_POOL.get().await.expect("failed to get pg connection");
        let mut harness = AsyncMigrationHarness::new(conn);
        harness
            .run_pending_migrations(PG_MIGRATIONS)
            .expect("failed to run migrations");
        println!("Database migrated")
    }

    pub(crate) async fn revert_all() {
        println!("Reverting all migrations");
        let conn = PG_POOL.get().await.expect("failed to get pg connection");
        let mut harness = AsyncMigrationHarness::new(conn);
        harness
            .revert_all_migrations(PG_MIGRATIONS)
            .expect("failed to revert migrations");
        println!("Database reverted")
    }

    pub(crate) async fn redo() {
        println!("Redoing all migrations");
        let conn = PG_POOL.get().await.expect("failed to get pg connection");
        let mut harness = AsyncMigrationHarness::new(conn);
        harness
            .revert_all_migrations(PG_MIGRATIONS)
            .expect("failed to revert migrations");
        harness
            .run_pending_migrations(PG_MIGRATIONS)
            .expect("failed to run migrations");
        println!("Database redone")
    }
}

mod minio {
    use common::bucket::policies::public_bucket_policy;
    use common::bucket::MINIO_CLIENT;
    use minio::s3::types::S3Api;

    const BUCKET_NAME: &str = "lot-images";

    pub(crate) async fn create_bucket() {
        println!("Creating bucket");
        MINIO_CLIENT
            .create_bucket(BUCKET_NAME)
            .send()
            .await
            .expect("failed to create bucket");
        MINIO_CLIENT
            .put_bucket_policy(BUCKET_NAME)
            .config(public_bucket_policy(BUCKET_NAME))
            .send()
            .await
            .expect("failed to set bucket policy");
        println!("Bucket created");
    }

    pub(crate) async fn delete_bucket() {
        println!("Deleting bucket");
        MINIO_CLIENT
            .delete_and_purge_bucket(BUCKET_NAME)
            .await
            .expect("failed to delete bucket");
        println!("Bucket deleted");
    }

    pub(crate) async fn recreate_bucket() {
        println!("Recreating bucket");
        MINIO_CLIENT
            .delete_and_purge_bucket(BUCKET_NAME)
            .await
            .expect("failed to delete bucket");
        MINIO_CLIENT
            .create_bucket(BUCKET_NAME)
            .send()
            .await
            .expect("failed to create bucket");
        MINIO_CLIENT
            .put_bucket_policy(BUCKET_NAME)
            .config(public_bucket_policy(BUCKET_NAME))
            .send()
            .await
            .expect("failed to set bucket policy");
        println!("Bucket recreated");
    }

    pub(crate) async fn create_absent_bucket() {
        println!("Creating absent bucket");
        let bucket_exist_response = MINIO_CLIENT
            .bucket_exists(BUCKET_NAME)
            .send()
            .await
            .expect("failed to check bucket");

        if bucket_exist_response.exists {
            println!("Bucket already exists");
            return;
        }

        MINIO_CLIENT
            .create_bucket(BUCKET_NAME)
            .send()
            .await
            .expect("failed to create bucket");
        MINIO_CLIENT
            .put_bucket_policy(BUCKET_NAME)
            .config(public_bucket_policy(BUCKET_NAME))
            .send()
            .await
            .expect("failed to set bucket policy");
        println!("Bucket created");
    }
}

#[tokio::main]
async fn main() {
    let args = cli::Args::parse();
    match args.command {
        cli::Command::Kafka { cmd } => dispatch_kafka(cmd).await,
        cli::Command::Postgres { cmd } => dispatch_postgres(cmd).await,
        cli::Command::Minio { cmd } => dispatch_minio(cmd).await,
    }
}

async fn dispatch_kafka(cmd: cli::KafkaCommand) {
    match cmd {
        cli::KafkaCommand::DeleteTopics => kafka::delete_topics().await,
        cli::KafkaCommand::CreateTopics => kafka::crate_topics().await,
        cli::KafkaCommand::RecreateTopics => kafka::recrate_topics().await,
        cli::KafkaCommand::CreateAbsentTopics => kafka::create_absent_topics().await,
    }
}

async fn dispatch_postgres(cmd: cli::PostgresCommand) {
    match cmd {
        cli::PostgresCommand::Migrate => postgres::migrate().await,
        cli::PostgresCommand::RevertAll => postgres::revert_all().await,
        cli::PostgresCommand::Redo => postgres::redo().await,
    };
}

async fn dispatch_minio(cmd: cli::MinioCommand) {
    match cmd {
        cli::MinioCommand::CreateBucket => minio::create_bucket().await,
        cli::MinioCommand::DeleteBucket => minio::delete_bucket().await,
        cli::MinioCommand::RecreateBucket => minio::recreate_bucket().await,
        cli::MinioCommand::CreateAbsentBucket => minio::create_absent_bucket().await,
    }
}
