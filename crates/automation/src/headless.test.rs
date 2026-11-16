#[cfg(test)]
mod end_merge_tests {
    use crate::headless::{Filesystem, Headless};
    use serde_json::json;

    #[cfg(unix)]
    #[test]
    fn trusted_reads_are_bounded_and_regular_files_only() {
        let dir = std::env::temp_dir().join(format!("materialcraft-read-cap-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let small = dir.join("small.txt");
        std::fs::write(&small, b"ok").unwrap();
        assert_eq!(crate::headless::read_file(&Filesystem::TrustedLocal, small.to_str().unwrap()).unwrap(), b"ok");

        let directory = dir.join("directory");
        std::fs::create_dir(&directory).unwrap();
        let err = crate::headless::read_file(&Filesystem::TrustedLocal, directory.to_str().unwrap()).unwrap_err().to_string();
        assert!(err.contains("not a regular file"), "{err}");

        let huge = dir.join("huge.bin");
        std::fs::File::create(&huge).unwrap().set_len(crate::workspace::MAX_READ_BYTES + 1).unwrap();
        let err = crate::headless::read_file(&Filesystem::TrustedLocal, huge.to_str().unwrap()).unwrap_err().to_string();
        assert!(err.contains("too large"), "{err}");

        std::fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn two_param_requests_are_two_undo_steps() {
        let mut h = Headless::new(Headless::default_session(), Filesystem::TrustedLocal);
        h.command_run("file.new", json!({"template": "empty"})).unwrap();
        let id = h.command_run("graph.node.add", json!({"def": "levels"})).unwrap()["id"].as_u64().unwrap();
        let sets = |h: &Headless| h.session.history().iter().filter(|l| *l == "Set parameter").count();
        h.command_run("graph.node.param", json!({"id": id, "param": "in_low", "value": [0.3, 0.3, 0.3, 0.3]})).unwrap();
        h.command_run("graph.node.param", json!({"id": id, "param": "in_low", "value": [0.4, 0.4, 0.4, 0.4]})).unwrap();
        assert_eq!(sets(&h), 2, "separate requests must not merge");
    }
}
