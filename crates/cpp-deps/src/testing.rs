pub mod corpus {
    use crate::r5;

    pub struct Entry {
        pub path: &'static r5::Utf8Path,
        pub json: &'static str,
        pub primary_output: &'static r5::Utf8Path,
    }

    pub mod entry {
        use super::*;

        pub fn bar() -> Entry {
            Entry {
                path: "bar.ddi".into(),
                json: r#"{
                    "rules": [
                        {
                            "primary-output": "bar.o",
                            "provides": [
                                {
                                    "logical-name": "bar",
                                    "is-interface": true
                                }
                            ],
                            "requires": [
                            ]
                        }
                    ],
                    "version": 0,
                    "revision": 0
                }"#,
                primary_output: "bar.o".into(),
            }
        }

        pub fn foo_part1() -> Entry {
            Entry {
                path: "foo-part1.ddi".into(),
                json: r#"{
                    "rules": [
                        {
                            "primary-output": "foo-part1.o",
                            "provides": [
                                {
                                    "logical-name": "foo:part1",
                                    "is-interface": true
                                }
                            ],
                            "requires": [
                            ]
                        }
                    ],
                    "version": 0,
                    "revision": 0
                }"#,
                primary_output: "foo-part1.o".into(),
            }
        }

        pub fn foo_part2() -> Entry {
            Entry {
                path: "foo-part2.ddi".into(),
                json: r#"{
                    "rules": [
                        {
                            "primary-output": "foo-part2.o",
                            "provides": [
                                {
                                    "logical-name": "foo:part2",
                                    "is-interface": true
                                }
                            ],
                            "requires": [
                            ]
                        }
                    ],
                    "version": 0,
                    "revision": 0
                }"#,
                primary_output: "foo-part2.o".into(),
            }
        }

        pub fn foo() -> Entry {
            Entry {
                path: "foo.ddi".into(),
                json: r#"{
                    "rules": [
                        {
                            "primary-output": "foo.o",
                            "provides": [
                                {
                                    "logical-name": "foo",
                                    "is-interface": true
                                }
                            ],
                            "requires": [
                                {
                                    "logical-name": "bar"
                                },
                                {
                                    "logical-name": "foo:part2"
                                },
                                {
                                    "logical-name": "foo:part1"
                                }
                            ]
                        }
                    ],
                    "version": 0,
                    "revision": 0
                }"#,
                primary_output: "foo.o".into(),
            }
        }

        pub fn main() -> Entry {
            Entry {
                path: "main.ddi".into(),
                json: r#"{
                    "rules": [
                        {
                            "primary-output": "main.o",
                            "requires": [
                                {
                                    "logical-name": "foo"
                                },
                                {
                                    "logical-name": "bar"
                                }
                            ]
                        }
                    ],
                    "version": 0,
                    "revision": 0
                }"#,
                primary_output: "main.o".into(),
            }
        }

        pub fn foo_cycle() -> Entry {
            Entry {
                path: "foo.ddi".into(),
                json: r#"{
                    "rules": [
                        {
                            "primary-output": "foo.o",
                            "provides": [
                                {
                                    "logical-name": "foo",
                                    "is-interface": true
                                }
                            ],
                            "requires": [
                                {
                                    "logical-name": "bar"
                                }
                            ]
                        }
                    ],
                    "version": 0,
                    "revision": 0
                }"#,
                primary_output: "foo.o".into(),
            }
        }

        pub fn bar_cycle() -> Entry {
            Entry {
                path: "bar.ddi".into(),
                json: r#"{
                    "rules": [
                        {
                            "primary-output": "bar.o",
                            "provides": [
                                {
                                    "logical-name": "bar",
                                    "is-interface": true
                                }
                            ],
                            "requires": [
                                {
                                    "logical-name": "foo"
                                }
                            ]
                        }
                    ],
                    "version": 0,
                    "revision": 0
                }"#,
                primary_output: "bar.o".into(),
            }
        }
    }
}

pub mod setup {
    use std::path::Path;

    pub fn build_script_env(out_dir: &Path) -> Option<()> {
        let out_dir = out_dir.as_os_str().to_str()?;
        std::env::set_var("OPT_LEVEL", "3");
        std::env::set_var("TARGET", "x86_64-unknown-linux-gnu");
        std::env::set_var("HOST", "x86_64-unknown-linux-gnu");
        std::env::set_var("OUT_DIR", out_dir);
        Some(())
    }
}
