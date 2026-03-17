use migration::{
    Migrator,
    data::{Database, Direction, MigratorWithData, Options, Runner},
};
use std::process::ExitCode;
use trustify_module_storage::config::StorageConfig;

#[derive(clap::Subcommand, Debug, Clone)]
#[allow(clippy::large_enum_variant)]
pub enum Data {
    /// List all data migrations
    List,
    /// Run a list of migrations
    Run(Run),
}

impl Data {
    pub async fn run(self) -> anyhow::Result<ExitCode> {
        match self {
            Self::List => {
                for m in Migrator::data_migrations() {
                    println!("{}", m.name());
                }
                Ok(ExitCode::SUCCESS)
            }
            Self::Run(run) => run.run().await,
        }
    }
}

#[derive(clap::Args, Debug, Clone)]
pub struct Run {
    /// Migration direction to run
    #[arg(
        long,
        value_enum,
        default_value_t = Direction::Up,
        overrides_with = "down"
    )]
    direction: Direction,

    /// Shortcut for `--direction down`
    #[arg(long, action = clap::ArgAction::SetTrue, overrides_with = "direction")]
    down: bool,

    // from sea_orm
    #[arg(
        global = true,
        short = 's',
        long,
        env = "DATABASE_SCHEMA",
        long_help = "Database schema\n \
                    - For MySQL and SQLite, this argument is ignored.\n \
                    - For PostgreSQL, this argument is optional with default value 'public'.\n"
    )]
    database_schema: Option<String>,

    // from sea_orm
    #[arg(
        global = true,
        short = 'u',
        long,
        env = "DATABASE_URL",
        help = "Database URL"
    )]
    database_url: Option<String>,

    #[arg()]
    migrations: Vec<String>,

    #[command(flatten)]
    options: Options,

    #[command(flatten)]
    storage: StorageConfig,
}

impl Run {
    fn direction(&self) -> Direction {
        if self.down {
            Direction::Down
        } else {
            self.direction
        }
    }

    #[allow(clippy::expect_used)]
    pub async fn run(self) -> anyhow::Result<ExitCode> {
        let direction = self.direction();
        let storage = self.storage.into_storage(false).await?;

        Runner {
            direction,
            storage,
            migrations: self.migrations,
            database: Database::Config {
                url: self
                    .database_url
                    .expect("Environment variable 'DATABASE_URL' not set"),
                schema: self.database_schema,
            },
            options: self.options,
        }
        .run::<Migrator>()
        .await?;

        Ok(ExitCode::SUCCESS)
    }
}
