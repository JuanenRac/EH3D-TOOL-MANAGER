//! The sample crib of the repository is what the pictures of the window show: it must load, and the window of a picture must open it without reading anything of the person's.

use eh3d_tool_manager::lang::Lang;
use eh3d_tool_manager::store;
use eh3d_tool_manager::window::{Status, Window};
use std::path::PathBuf;

fn sample() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("examples")
        .join("crib.json")
}

#[test]
fn the_sample_crib_loads_with_tools_of_every_kind() {
    let crib = store::load(&sample()).expect("the sample crib loads");
    assert_eq!(crib.tools.len(), 9);
    let mut kinds: Vec<&str> = crib.tools.iter().map(|t| t.kind.key()).collect();
    kinds.sort_unstable();
    kinds.dedup();
    assert_eq!(kinds.len(), 6);
}

#[test]
fn the_window_of_a_picture_opens_the_sample_with_default_settings() {
    let w = Window::for_shot(Some(sample()), Lang::Es);
    assert_eq!(w.crib.tools.len(), 9);
    assert!(matches!(w.status, Status::Opened { tools: 9, .. }));
    assert_eq!(w.lang, Lang::Es);
    assert_eq!(w.settings, eh3d_tool_manager::settings::Settings::default());
    assert!(!w.dirty);
}

#[test]
fn the_window_of_a_picture_without_a_file_has_an_empty_crib_and_reads_nothing() {
    let w = Window::for_shot(None, Lang::En);
    assert!(w.crib.tools.is_empty());
    assert!(matches!(w.status, Status::Ready));
    assert_eq!(w.file_text, "crib.json");
}
