use crate::{cli, db};

pub fn initialize_app(cli: &cli::Cli) -> anyhow::Result<()> {
    let existed = cli.db.exists();
    let conn = db::create_connection(&cli.db)?;
    db::init_schema(&conn)?;
    if existed {
        println!("Reinitialized db at: {}", cli.db.display());
    } else {
        println!("Successfully initialized db at: {}", cli.db.display());
    }
    Ok(())
}
