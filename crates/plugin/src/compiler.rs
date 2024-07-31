use std::io::Write;
use std::iter::zip;
use std::ops::DerefMut;

use anyhow::{anyhow, Context, Result};
use cairo_lang_compiler::db::RootDatabase;
use cairo_lang_defs::db::DefsGroup;
use cairo_lang_filesystem::db::FilesGroup;
use cairo_lang_formatter::format_string;
use cairo_lang_starknet::compile::compile_prepared_db;
use cairo_lang_starknet::contract::find_contracts;
use cairo_lang_utils::UpcastMut;
use camino::Utf8Path;
use scarb::compiler::helpers::{build_compiler_config, collect_main_crate_ids};
use scarb::compiler::{CairoCompilationUnit, Compiler};
use scarb::core::{TargetKind, Workspace};

pub const SOURCES_DIR: &str = "src";

#[derive(Debug)]
pub struct DemoCompiler;

impl Compiler for DemoCompiler {
    fn target_kind(&self) -> TargetKind {
        TargetKind::new("demo")
    }

    fn compile(
        &self,
        unit: CairoCompilationUnit,
        db: &mut RootDatabase,
        ws: &Workspace<'_>,
    ) -> Result<()> {
        let target_dir = unit.target_dir(ws);
        let sources_dir = target_dir.child(Utf8Path::new(SOURCES_DIR));

        let compiler_config = build_compiler_config(&unit, ws);

        let main_crate_ids = collect_main_crate_ids(&unit, db);

        let contracts = find_contracts(db.upcast_mut(), &main_crate_ids);

        let contracts = contracts.iter().collect::<Vec<_>>();

        let classes = { compile_prepared_db(db, &contracts, compiler_config)? };

        for (decl, class) in zip(contracts, classes) {
            let contract_full_path = decl.module_id().full_path(db.upcast_mut());

            // save expanded contract source file
            if let Ok(file_id) = db.module_main_file(decl.module_id()) {
                if let Some(file_content) = db.file_content(file_id) {
                    let src_file_name = format!("{contract_full_path}.cairo").replace("::", "_");

                    let mut file =
                        sources_dir.open_rw(src_file_name.clone(), "source file", ws.config())?;
                    file.write(format_string(db, file_content.to_string()).as_bytes())
                        .with_context(|| {
                            format!("failed to serialize contract source: {contract_full_path}")
                        })?;
                } else {
                    return Err(anyhow!(
                        "failed to get source file content: {contract_full_path}"
                    ));
                }
            } else {
                return Err(anyhow!("failed to get source file: {contract_full_path}"));
            }

            let file_name = format!("{contract_full_path}.sierra.json");
            let mut file = target_dir.open_rw(file_name.clone(), "class file", ws.config())?;
            serde_json::to_writer_pretty(file.deref_mut(), &class).with_context(|| {
                format!("failed to serialize contract artifact: {contract_full_path}")
            })?;
        }

        Ok(())
    }
}
