use anyhow::Result;
use camino::Utf8PathBuf;
use scarb::compiler::{CompilerRepository, Profile};
use scarb::core::{Config, TargetKind};
use scarb::ops::{CompileOpts, FeaturesOpts, FeaturesSelector};
use std::env;

use compiler::DemoCompiler;
use plugin::CairoPluginRepository;

pub mod compiler;
pub mod plugin;
pub mod scarb_funcs;

fn main() -> Result<()> {
    let manifest = Utf8PathBuf::from(
        std::fs::canonicalize("./demo_code/Scarb.toml")?
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
        include_targets: vec![],
        exclude_targets: vec![TargetKind::TEST],
        features: FeaturesOpts {
            features: FeaturesSelector::AllFeatures,
            no_default_features: false,
        },
    };

    scarb_funcs::compile_workspace(&config, opts)?;

    Ok(())
}
