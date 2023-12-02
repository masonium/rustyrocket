use anyhow::Result;
use ron::ser::PrettyConfig;
use rustyrocket::obstacle::spawner_settings::SpawnerSettings;

fn main() -> Result<()> {
    let ss = SpawnerSettings::new();
    let f = std::fs::File::options()
        .create(true)
        .write(true)
        .open("assets/levels/base.spawner.ron")?;
    ron::ser::to_writer_pretty(f, &ss, PrettyConfig::default().struct_names(true))?;

    Ok(())
}
