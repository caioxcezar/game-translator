fn main() {
    glib_build_tools::compile_resources(
        &["resources"],
        "resources/game_translator.gresource.xml",
        "game_translator.gresource",
    );
}
