use materialcraft_engine::{View3d, execute, new_session};
use serde_json::json;

#[test]
fn normal_map_convention_is_inspectable_and_resets_to_directx() {
    let mut session = new_session();

    assert!(session.view3d.normal_y_flip);
    assert_eq!(materialcraft_engine::nodes::find("normal").unwrap().param("opengl").unwrap().default.as_bool(), Some(false));

    execute(&mut session, "view.3d", &json!({"normalYFlip": false})).unwrap();
    let inspection = execute(&mut session, "session.inspect", &json!({})).unwrap();

    assert_eq!(inspection["view3d"]["normalYFlip"], false);

    let saved = serde_json::to_value(&session.view3d).unwrap();

    assert!(!serde_json::from_value::<View3d>(saved).unwrap().normal_y_flip);
    assert!(serde_json::from_value::<View3d>(json!({"scene": "sphere"})).unwrap().normal_y_flip);

    execute(&mut session, "view.3d.reset", &json!({})).unwrap();

    assert!(session.view3d.normal_y_flip);
}
