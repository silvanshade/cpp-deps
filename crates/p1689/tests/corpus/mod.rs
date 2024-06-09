use crate::common::*;

#[cfg(feature = "parsing")]
mod parsing {
    use super::*;

    #[test]
    fn gcc() -> BoxResult<()> {
        let expected_failures = [
            "p1689-1.exp.ddi",
            "p1689-4.exp.ddi",
            "p1689-file-default.exp.ddi",
            "p1689-target-default.exp.ddi",
        ];
        let val = std::env::var("CARGO_MANIFEST_DIR")?;
        let mut dir = camino::Utf8PathBuf::from(val);
        assert_eq!(Some("p1689"), dir.file_name());
        dir.extend(["..", "..", "corpus", "gcc", "gcc", "testsuite", "g++.dg", "modules"]);
        'outer: for result in std::fs::read_dir(dir)? {
            let dir_entry = result?;
            if dir_entry.file_type()?.is_file() {
                let path = dir_entry.path();
                let path = path.as_os_str().to_string_lossy();
                if path.to_lowercase().ends_with(".ddi") {
                    let dep_file = std::fs::read_to_string(dir_entry.path())?;
                    let input = dep_file.as_bytes();
                    let state = p1689::r5::parsers::State::default();
                    let mut stream = p1689::r5::parsers::ParseStream::new(&path, input, state);
                    match p1689::r5::parsers::dep_file(&mut stream) {
                        Err(err) => {
                            for name in expected_failures {
                                if path.ends_with(name) {
                                    continue 'outer;
                                }
                            }
                            panic!("{err}");
                        },
                        Ok(_val) => {
                            // println!("{val:#?}");
                        },
                    }
                }
            }
        }
        Ok(())
    }
}

#[cfg(all(feature = "serde", feature = "deserialize"))]
mod serde {
    use super::*;

    #[test]
    fn gcc() -> BoxResult<()> {
        let expected_failures = [
            "p1689-1.exp.ddi",
            "p1689-4.exp.ddi",
            "p1689-file-default.exp.ddi",
            "p1689-target-default.exp.ddi",
        ];
        let val = std::env::var("CARGO_MANIFEST_DIR")?;
        let mut dir = camino::Utf8PathBuf::from(val);
        assert_eq!(Some("p1689"), dir.file_name());
        dir.extend(["..", "..", "corpus", "gcc", "gcc", "testsuite", "g++.dg", "modules"]);
        'outer: for result in std::fs::read_dir(dir)? {
            let dir_entry = result?;
            if dir_entry.file_type()?.is_file() {
                let path = dir_entry.path();
                let path = path.as_os_str().to_string_lossy();
                if path.to_lowercase().ends_with(".ddi") {
                    let dep_file = std::fs::read_to_string(dir_entry.path())?;
                    match serde_json::from_str::<p1689::r5::DepFile>(&dep_file) {
                        Err(err) => {
                            for name in expected_failures {
                                if path.ends_with(name) {
                                    continue 'outer;
                                }
                            }
                            panic!("{err}");
                        },
                        Ok(_val) => {
                            // println!("{val:#?}");
                        },
                    }
                }
            }
        }
        Ok(())
    }
}
