use anyhow::Result;
use cairo_lang_defs::patcher::{PatchBuilder, RewriteNode};
use cairo_lang_defs::plugin::{
    MacroPlugin, MacroPluginMetadata, PluginGeneratedFile, PluginResult,
};
use cairo_lang_semantic::plugin::PluginSuite;
use cairo_lang_syntax::node::ast::MaybeModuleBody;
use cairo_lang_syntax::node::db::SyntaxGroup;
use cairo_lang_syntax::node::helpers::QueryAttrs;
use cairo_lang_syntax::node::{ast, Terminal, TypedSyntaxNode};
use cairo_lang_utils::unordered_hash_map::UnorderedHashMap;
use scarb::compiler::plugin::builtin::BuiltinStarkNetPlugin;
use scarb::compiler::plugin::{CairoPlugin, CairoPluginInstance};
use scarb::core::{PackageId, PackageName, SourceId};
use semver::Version;
use url::Url;
use cairo_lang_syntax::attribute::structured::{
    AttributeArg, AttributeArgVariant, AttributeStructurize,
};

pub const PACKAGE_NAME: &str = "cairo_plugin_demo";
pub const MY_ATTR: &str = "custom::contract";

#[derive(Debug, Default)]
pub struct BuiltinDemoPlugin;

impl BuiltinDemoPlugin {
    pub fn handle_mod(&self, db: &dyn SyntaxGroup, module_ast: &ast::ItemModule) -> PluginResult {
        if !module_ast.has_attr(db, MY_ATTR) {
            return PluginResult {
                code: None,
                diagnostics: vec![],
                remove_original_item: false,
            };
        }

        let custom_attr = module_ast.find_attr(db, MY_ATTR).unwrap();
        let origin = custom_attr.as_syntax_node().span_without_trivia(db);

        let name = module_ast.name(db).text(db);

        //let mut diagnostics = vec![];

        if let MaybeModuleBody::Some(body) = module_ast.body(db) {
            let mut builder = PatchBuilder::new(db, module_ast);

            let mut body_nodes: Vec<_> = body
                .items(db)
                .elements(db)
                .iter()
                .flat_map(|el| {
                    if let ast::ModuleItem::Impl(impl_ast) = el {
                        // The error was that, we need to have a Mapped node here. To ensure the origin
                        // is correctly taken from the replaced node instead of the origin being the
                        // attribute.
                        return vec![RewriteNode::Mapped {
                            node: Box::new(RewriteNode::Text("impl A of B {}".to_string())),
                            origin: impl_ast.as_syntax_node().span_without_trivia(db),
                        }];

                        // Case 2:
                        // Even if we try to add the node directly to the builder,
                        // the diagnostic pointer will be incorrect.
                        // builder.add_modified(RewriteNode::Text("impl A of B {}".to_string()));
                        // return vec![];
                    }

                    vec![RewriteNode::Copied(el.as_syntax_node())]
                })
                .collect();

            body_nodes.append(&mut vec![RewriteNode::Text("struct S {}".to_string())]);

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

            println!("{:?}", code_mappings);

            return PluginResult {
                code: Some(PluginGeneratedFile {
                    name: name.clone(),
                    content: code,
                    aux_data: None,
                    code_mappings,
                }),
                diagnostics: vec![],
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
        let version = "0.1.0";
        // TODO: update this once pushed.
        // let rev = "1";

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
        _metadata: &MacroPluginMetadata<'_>,
    ) -> PluginResult {
        match item_ast {
            ast::ModuleItem::Module(module_ast) => self.handle_mod(db, &module_ast),
            _ => PluginResult::default(),
        }
    }

    fn declared_attributes(&self) -> Vec<String> {
        vec![MY_ATTR.to_string()]
    }
}

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
