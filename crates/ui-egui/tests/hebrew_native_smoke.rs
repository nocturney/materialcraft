//! Hebrew UI smoke: native file round-trip and a real offscreen egui render.
//! Run the GPU test with MC_SNAPSHOT_DIR=/tmp/... cargo test --test hebrew_native_smoke -- --ignored --nocapture.
//! The snapshot is an actual UI render, not a mockup or a release-acceptance claim.

use materialcraft_ui_egui::{MaterialCraftApp, UiState, i18n};
use serde_json::json;

fn hebrew_app() -> MaterialCraftApp {
    let mut ui = UiState::default();
    ui.prefs.language = "he".to_owned();
    ui.prefs.show_home = false;
    MaterialCraftApp::headless(materialcraft_engine::new_session(), ui)
}

fn round_trip(dir: &std::path::Path) -> MaterialCraftApp {
    std::fs::create_dir_all(dir).expect("snapshot/output directory");
    let path = dir.join("artcraft-hebrew-roundtrip.sbs");

    let mut writer = hebrew_app();
    assert_eq!(i18n::current().code, "he");
    assert!(i18n::is_rtl(), "Hebrew must use RTL direction");
    assert_eq!(i18n::tr("File"), "קובץ");
    writer.session.pkg.name = "בדיקת שמירה בעברית".to_owned();
    writer.run("file.saveAs", json!({"path": path})).expect("save Hebrew-named package");
    assert!(path.is_file(), "the native save must produce an actual package");

    // Open the saved document in a fresh app instance, avoiding a false
    // positive from in-memory state in the writer.
    let mut reader = hebrew_app();
    reader.run("file.open", json!({"path": path})).expect("reopen native package");
    assert_eq!(reader.session.pkg.name, "בדיקת שמירה בעברית");
    assert_eq!(reader.ui.prefs.language, "he");
    assert_eq!(i18n::current().code, "he");
    reader
}

#[test]
fn hebrew_native_open_save_round_trip() {
    let dir = std::env::temp_dir().join(format!("materialcraft-hebrew-smoke-{}", std::process::id()));
    let _app = round_trip(&dir);
    println!("PASS: native Hebrew save/reopen at {}", dir.display());
}

#[test]
#[ignore = "needs an offscreen GPU; run explicitly with --ignored"]
fn hebrew_native_rtl_offscreen_screenshot() {
    use egui::{accesskit::Role, vec2};
    use egui_kittest::{
        Harness,
        kittest::{By, Queryable},
    };

    let dir = std::path::PathBuf::from(std::env::var_os("MC_SNAPSHOT_DIR").expect("set MC_SNAPSHOT_DIR to a writable output directory"));
    let app = round_trip(&dir);
    let mut harness: Harness<'static, MaterialCraftApp> = Harness::builder().with_size(vec2(1440.0, 900.0)).with_pixels_per_point(1.0).wgpu().build_ui_state(
        |ui, app: &mut MaterialCraftApp| {
            app.step(&ui.ctx().clone());
            app.draw(ui);
        },
        app,
    );
    harness.run_steps(8);
    assert!(harness.query(By::new().role(Role::Button).label("קובץ")).is_some(), "the native UI must expose a translated File menu");
    let image = harness.render().expect("offscreen egui render");
    let output = dir.join("materialcraft-hebrew-native-rtl.png");
    image.save(&output).expect("write screenshot");
    assert!(output.metadata().expect("saved screenshot").len() > 1_000);
    println!("PASS: native Hebrew egui RTL screenshot {}", output.display());
}