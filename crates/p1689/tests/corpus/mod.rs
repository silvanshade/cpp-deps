use crate::common::*;

const GCC_EXPECTED_FAILURES: &[&str] = &["p1689-4.exp.ddi"];

fn gcc_preprocess(haystack: &str) -> BoxResult<'static, Cow<str>> {
    // NOTE: Remove non-standard GCC directive
    let rx = Regex::new(r#""__P1689_unordered__".*,.*\n"#)?;
    let rep = "";
    Ok(rx.replace(haystack, rep))
}

fn process_ddi_files<P, F>(sub_dir: &Path, expected_failures: &[&str], preprocess: P, f: F) -> BoxResult<'static, ()>
where
    P: Fn(&str) -> BoxResult<'static, Cow<str>>,
    F: for<'i> Fn(&'i str, &'i str) -> BoxResult<'i, ()>,
{
    let val = std::env::var("CARGO_MANIFEST_DIR")?;
    let mut dir = PathBuf::from(val);
    assert_eq!(Some(OsStr::new("p1689")), dir.file_name());
    dir = dir.join(sub_dir);
    'outer: for result in std::fs::read_dir(dir)? {
        let dir_entry = result?;
        if dir_entry.file_type()?.is_file() {
            let path = dir_entry.path();
            let path_str = path.as_os_str().to_string_lossy();
            if path_str.to_lowercase().ends_with(".ddi") {
                let dep_file = std::fs::read_to_string(dir_entry.path())?;
                let dep_file = preprocess(&dep_file)?;
                match f(&path_str, &dep_file) {
                    Err(err) => {
                        for name in expected_failures {
                            if path.ends_with(name) {
                                continue 'outer;
                            }
                        }
                        panic!("{err}");
                    },
                    Ok(_val) => {},
                };
            }
        }
    }
    Ok(())
}

#[cfg(feature = "parsing")]
mod parsing {
    use super::*;

    #[test]
    fn gcc() -> BoxResult<'static, ()> {
        let sub_dir = Path::new("../../corpus/gcc/gcc/testsuite/g++.dg/modules");
        process_ddi_files(sub_dir, GCC_EXPECTED_FAILURES, gcc_preprocess, |path_str, dep_file| {
            let input = dep_file.as_bytes();
            let state = p1689::r5::parsers::State::default();
            let mut stream = p1689::r5::parsers::ParseStream::new(path_str, input, state);
            p1689::r5::parsers::dep_file(&mut stream)?;
            Ok(())
        })
    }
}

#[cfg(all(feature = "serde", feature = "deserialize"))]
mod serde {
    use super::*;

    #[test]
    fn gcc() -> BoxResult<'static, ()> {
        let sub_dir = Path::new("../../corpus/gcc/gcc/testsuite/g++.dg/modules");
        process_ddi_files(sub_dir, GCC_EXPECTED_FAILURES, gcc_preprocess, |_path_str, dep_file| {
            serde_json::from_str::<p1689::r5::DepFile>(dep_file)?;
            Ok(())
        })
    }
}
