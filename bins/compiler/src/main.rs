use anyhow::Result;
use camino::Utf8PathBuf;
use scarb::compiler::{CompilerRepository, Profile};
use scarb::core::{Config, TargetKind};
use scarb::ops::{CompileOpts, FeaturesOpts, FeaturesSelector};
use std::env;

use demo_plugin::{compiler::DemoCompiler, plugin::CairoPluginRepository, scarb_funcs};

use clap::Parser;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    /// Path to the Scarb.toml file
    #[arg(short, long, value_name = "MANIFEST_FILE_PATH")]
    manifest_path: Utf8PathBuf,
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    let manifest = Utf8PathBuf::from(
        std::fs::canonicalize(cli.manifest_path)?
            .as_os_str()
            .to_string_lossy()
            .to_string(),
    );

    let mut compilers = CompilerRepository::std();
    compilers.add(Box::new(DemoCompiler)).unwrap();

    let config = Config::builder(manifest)
        .log_filter_directive(env::var_os("SCARB_LOG"))
        .profile(Profile::DEV)
        .offline(false)
        .cairo_plugins(CairoPluginRepository::default().into())
        .ui_verbosity(scarb_ui::Verbosity::Verbose)
        .compilers(compilers)
        .build()?;

    let opts = CompileOpts {
        include_target_kinds: vec![],
        include_target_names: vec![],
        exclude_target_kinds: vec![TargetKind::TEST],
        features: FeaturesOpts {
            features: FeaturesSelector::AllFeatures,
            no_default_features: false,
        },
    };

    scarb_funcs::compile_workspace(&config, opts)?;

    Ok(())
}
