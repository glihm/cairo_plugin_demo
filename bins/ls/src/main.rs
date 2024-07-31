use cairo_lang_language_server::Tricks;
use clap::Parser;
use demo_plugin::plugin::demo_plugin_suite;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {}

fn main() {
    let _args = Args::parse();

    let mut tricks = Tricks::default();
    tricks.extra_plugin_suites = Some(&|| vec![demo_plugin_suite()]);
    cairo_lang_language_server::start_with_tricks(tricks);
}
