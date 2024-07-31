use anyhow::Result;
use cairo_lang_defs::patcher::{PatchBuilder, RewriteNode};
use cairo_lang_defs::plugin::{
    MacroPlugin, MacroPluginMetadata, PluginDiagnostic, PluginGeneratedFile, PluginResult,
};
use cairo_lang_plugins::plugins::HasItemsInCfgEx;
use cairo_lang_semantic::plugin::PluginSuite;
use cairo_lang_syntax::node::ast::MaybeModuleBody;
use cairo_lang_syntax::node::db::SyntaxGroup;
use cairo_lang_syntax::node::helpers::QueryAttrs;
use cairo_lang_syntax::node::{ast, Terminal, TypedStablePtr, TypedSyntaxNode};
use cairo_lang_utils::unordered_hash_map::UnorderedHashMap;
use scarb::compiler::plugin::builtin::BuiltinStarkNetPlugin;
use scarb::compiler::plugin::{CairoPlugin, CairoPluginInstance};
use scarb::core::{PackageId, PackageName, SourceId};
use semver::Version;
use url::Url;

use crate::syntax_utils;

pub const PACKAGE_NAME: &str = "cairo_plugin_demo";
pub const MY_ATTR: &str = "custom::contract";

#[derive(Debug, Default)]
pub struct BuiltinDemoPlugin;

impl BuiltinDemoPlugin {
    pub fn handle_mod(
        &self,
        db: &dyn SyntaxGroup,
        module_ast: &ast::ItemModule,
        metadata: &MacroPluginMetadata<'_>,
    ) -> PluginResult {
        if !module_ast.has_attr(db, MY_ATTR) {
            return PluginResult {
                code: None,
                diagnostics: vec![],
                remove_original_item: false,
            };
        }

        let name = module_ast.name(db).text(db);

        let mut diagnostics = vec![];

        if let MaybeModuleBody::Some(body) = module_ast.body(db) {
            let mut builder = PatchBuilder::new(db, module_ast);

            let mut body_nodes: Vec<_> = body
                .items(db)
                .elements(db)
                .iter()
                .flat_map(|el| {
                    if let ast::ModuleItem::Impl(impl_ast) = el {
                        let mut impl_nodes = vec![];

                        let impl_name = impl_ast.name(db).text(db);
                        if impl_name == "bad" {
                            // Try a plugin diagnostic to check diagnostic handling by LS.
                            diagnostics.push(PluginDiagnostic::error(
                                impl_ast.stable_ptr().untyped(),
                                "Invalid impl name".to_string(),
                            ));
                        }

                        // Map the existing impl node as it will be replaced, to have correct
                        // diagnostics.
                        let impl_node = RewriteNode::Mapped {
                            node: Box::new(RewriteNode::Text(format!(
                                "{} impl {} of {} {{\n",
                                impl_ast.attributes(db).as_syntax_node().get_text(db),
                                impl_name,
                                impl_ast.trait_path(db).as_syntax_node().get_text(db),
                            ))),
                            origin: impl_ast.as_syntax_node().span_without_trivia(db),
                        };

                        impl_nodes.push(impl_node);

                        if let ast::MaybeImplBody::Some(impl_body) = impl_ast.body(db) {
                            let body_nodes: Vec<_> = impl_body
                                .iter_items_in_cfg(db, metadata.cfg_set)
                                .flat_map(|el| {
                                    if let ast::ImplItem::Function(ref fn_ast) = el {
                                        rewrite_function(db, fn_ast.clone())
                                    } else {
                                        vec![RewriteNode::Copied(el.as_syntax_node())]
                                    }
                                })
                                .collect();

                            // Also mapping the body nodes to have correct diagnostics.
                            let mapped_node = RewriteNode::Mapped {
                                node: Box::new(RewriteNode::interpolate_patched(
                                    "$body$",
                                    &UnorderedHashMap::from([(
                                        "body".to_string(),
                                        RewriteNode::new_modified(body_nodes),
                                    )]),
                                )),
                                origin: impl_ast.as_syntax_node().span_without_trivia(db),
                            };

                            impl_nodes.push(mapped_node);
                        }

                        impl_nodes.push(RewriteNode::Text("\n}".to_string()));
                        return impl_nodes;
                    }

                    // Other items are copied as is.
                    vec![RewriteNode::Copied(el.as_syntax_node())]
                })
                .collect();

            // Add a standalone struct.
            body_nodes.append(&mut vec![RewriteNode::Text("\nstruct S {}\n".to_string())]);

            builder.add_modified(RewriteNode::interpolate_patched(
                "
            #[starknet::contract]
            mod $name$ {

                $body$

            }
            ",
                &UnorderedHashMap::from([
                    ("name".to_string(), RewriteNode::Text(name.to_string())),
                    ("body".to_string(), RewriteNode::new_modified(body_nodes)),
                ]),
            ));

            let (code, code_mappings) = builder.build();

            //dbg!(&code_mappings);

            return PluginResult {
                code: Some(PluginGeneratedFile {
                    name: name.clone(),
                    content: code,
                    aux_data: None,
                    code_mappings,
                }),
                diagnostics,
                // Remove the original one as we've replaced it with our own modified node.
                remove_original_item: true,
            };
        }

        PluginResult::default()
    }
}

impl CairoPlugin for BuiltinDemoPlugin {
    fn id(&self) -> PackageId {
        let url =
            Url::parse(format!("https://github.com/glihm/{}", PACKAGE_NAME).as_str()).unwrap();
        let version = "0.2.0";

        let source_id = SourceId::for_git(
            &url,
            &scarb::core::GitReference::Tag(format!("v{version}").into()),
        )
        .unwrap();
        //.with_precise(rev.to_string())
        //.unwrap();

        PackageId::new(
            PackageName::new(PACKAGE_NAME),
            Version::parse(version).unwrap(),
            source_id,
        )
    }

    fn instantiate(&self) -> Result<Box<dyn CairoPluginInstance>> {
        Ok(Box::new(BuiltinDemoPluginInstance))
    }
}

struct BuiltinDemoPluginInstance;
impl CairoPluginInstance for BuiltinDemoPluginInstance {
    fn plugin_suite(&self) -> PluginSuite {
        demo_plugin_suite()
    }
}

pub fn demo_plugin_suite() -> PluginSuite {
    let mut suite = PluginSuite::default();

    suite.add_plugin::<BuiltinDemoPlugin>();

    suite
}

impl MacroPlugin for BuiltinDemoPlugin {
    fn generate_code(
        &self,
        db: &dyn SyntaxGroup,
        item_ast: ast::ModuleItem,
        metadata: &MacroPluginMetadata<'_>,
    ) -> PluginResult {
        match item_ast {
            ast::ModuleItem::Module(module_ast) => {
                if module_ast.has_attr(db, MY_ATTR) {
                    self.handle_mod(db, &module_ast, metadata)
                } else {
                    PluginResult::default()
                }
            }
            _ => PluginResult::default(),
        }
    }

    fn declared_attributes(&self) -> Vec<String> {
        vec![MY_ATTR.to_string()]
    }
}

#[derive(Debug)]
pub struct CairoPluginRepository(scarb::compiler::plugin::CairoPluginRepository);

impl Default for CairoPluginRepository {
    fn default() -> Self {
        let mut repo = scarb::compiler::plugin::CairoPluginRepository::empty();
        repo.add(Box::new(BuiltinDemoPlugin)).unwrap();
        repo.add(Box::new(BuiltinStarkNetPlugin)).unwrap();
        Self(repo)
    }
}

impl From<CairoPluginRepository> for scarb::compiler::plugin::CairoPluginRepository {
    fn from(val: CairoPluginRepository) -> Self {
        val.0
    }
}

pub fn rewrite_function(db: &dyn SyntaxGroup, fn_ast: ast::FunctionWithBody) -> Vec<RewriteNode> {
    let fn_name = fn_ast.declaration(db).name(db).text(db);
    let return_type = fn_ast
        .declaration(db)
        .signature(db)
        .ret_ty(db)
        .as_syntax_node()
        .get_text(db);

    let params_str = rewrite_parameters(db, fn_ast.declaration(db).signature(db).parameters(db));

    let declaration_node = RewriteNode::Mapped {
        node: Box::new(RewriteNode::Text(format!(
            "fn {}({}) {} {{\n",
            fn_name, params_str, return_type
        ))),
        origin: fn_ast
            .declaration(db)
            .as_syntax_node()
            .span_without_trivia(db),
    };

    // Add some new statements inside the function before user's ones.
    let additional_node1 = RewriteNode::Text("let a = 32;\n".to_string());
    let additional_node2 = RewriteNode::Text("let _b = a + 4;\n".to_string());

    let func_nodes = fn_ast
        .body(db)
        .statements(db)
        .elements(db)
        .iter()
        .map(|e| RewriteNode::Mapped {
            node: Box::new(RewriteNode::from(e.as_syntax_node())),
            origin: e.as_syntax_node().span_without_trivia(db),
        })
        .collect::<Vec<_>>();

    let mut nodes = vec![declaration_node, additional_node1, additional_node2];
    nodes.extend(func_nodes);
    nodes.push(RewriteNode::Text("\n}".to_string()));

    nodes
}

/// Rewrites the parameters of a function by replacing `r: R` to `ref self: ContractState` if present,
/// otherwise adding `self: @ContractState` at the beginning.
///
/// # Arguments
///
/// * `db` - The syntax group.
/// * `param_list` - The list of parameters.
/// * `fn_diagnostic_item` - The diagnostic item.
///
/// # Returns
///
/// * The rewritten parameters as a string.
pub fn rewrite_parameters(db: &dyn SyntaxGroup, param_list: ast::ParamList) -> String {
    let mut use_ref = false;

    let mut params = param_list
        .elements(db)
        .iter()
        .filter_map(|param| {
            let param_info = syntax_utils::get_parameter_info(db, param.clone());

            if &param_info.name == "r" && &param_info.param_type == "R" {
                use_ref = true;
                None
            } else {
                Some(param.as_syntax_node().get_text(db))
            }
        })
        .collect::<Vec<_>>();

    if use_ref {
        params.insert(0, "ref self: ContractState".to_string());
    } else {
        params.insert(0, "self: @ContractState".to_string());
    }

    params.join(", ")
}
