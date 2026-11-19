use materialcraft_automation::headless::{Filesystem, Headless, RenderOptions};
use materialcraft_engine::{
    execute,
    image::{Image, Kind},
    new_session,
    nodes::atomic::height_to_normal,
};
use materialcraft_render::{Lighting, Maps, Scene, Settings, render};
use serde_json::json;

#[test]
fn generated_normal_conventions_produce_the_same_surface_when_matched() {
    let height = Image::gray(32, 32, |x, y| ((x as f32 * 0.15).sin() + (y as f32 * 0.25).sin()) * 0.25 + 0.5);
    let opengl_normal = height_to_normal(&height, 8.0, false, false, false);
    let directx_normal = height_to_normal(&height, 8.0, true, false, false);
    let normal_node = materialcraft_engine::nodes::find("normal").unwrap();

    assert_eq!(normal_node.param("opengl").unwrap().default.as_bool(), Some(false));

    let maps = |normal| Maps { normal: Some(normal), base_color: Some(Image::filled(4, 4, Kind::Color, [0.5, 0.5, 0.5, 1.0])), ..Maps::default() };
    let (scene, light) = (Scene::builtin("sphere"), Lighting::new("studio"));
    let settings = Settings::default();
    let expected = render(&scene, &maps(directx_normal.clone()), &light, &Settings { normal_y_flip: true, ..settings }, 64, 48);
    let matched = render(&scene, &maps(opengl_normal), &light, &Settings { normal_y_flip: false, ..settings }, 64, 48);

    assert!(expected.iter().zip(&matched).all(|(actual, expected)| actual.abs_diff(*expected) <= 1));

    let default_output = render(&scene, &maps(directx_normal.clone()), &light, &settings, 64, 48);

    assert!(expected.iter().zip(&default_output).all(|(actual, expected)| actual.abs_diff(*expected) <= 1));

    let mismatched = render(&scene, &maps(directx_normal), &light, &Settings { normal_y_flip: false, ..settings }, 64, 48);

    assert!(expected.iter().zip(&mismatched).filter(|(actual, expected)| actual.abs_diff(**expected) > 2).count() > 20);
    assert!(settings.normal_y_flip);
}

#[test]
fn headless_render_honors_the_normal_y_command() {
    let mut session = new_session();
    session.preview_size = Some(64);
    let node = execute(&mut session, "graph.node.add", &json!({"def": "uniform", "params": {"color": [0.5, 0.75, 1.0, 1.0]}})).unwrap()["id"].as_u64().unwrap();
    execute(&mut session, "graph.node.add", &json!({"def": "output", "connectFrom": {"node": node}, "params": {"usage": 1}})).unwrap();
    let mut headless = Headless::new(session, Filesystem::Denied);
    let options = RenderOptions { width: Some(64), height: Some(48), ..RenderOptions::default() };
    let directx_render = headless.render_png(&options).unwrap().png;

    execute(&mut headless.session, "view.3d", &json!({"normalYFlip": false})).unwrap();
    let opengl_render = headless.render_png(&options).unwrap().png;

    assert_ne!(directx_render, opengl_render);

    execute(&mut headless.session, "graph.node.param", &json!({"id": node, "param": "color", "value": [0.5, 0.25, 1.0, 1.0]})).unwrap();

    assert_eq!(directx_render, headless.render_png(&options).unwrap().png);

    execute(&mut headless.session, "view.3d.reset", &json!({})).unwrap();

    assert_eq!(opengl_render, headless.render_png(&options).unwrap().png);
}
