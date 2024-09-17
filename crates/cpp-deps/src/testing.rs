pub mod corpus {
    use crate::r5;

    pub mod item {
        use super::*;

        pub fn bar() -> (&'static r5::Utf8Path, &'static str) {
            let path = "bar.ddi".into();
            let json = r#"{
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
            }"#;
            (path, json)
        }

        pub fn foo_part1() -> (&'static r5::Utf8Path, &'static str) {
            let path = "foo-part1.ddi".into();
            let json = r#"{
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
            }"#;
            (path, json)
        }

        pub fn foo_part2() -> (&'static r5::Utf8Path, &'static str) {
            let path = "foo-part2.ddi".into();
            let json = r#"{
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
            }"#;
            (path, json)
        }

        pub fn foo() -> (&'static r5::Utf8Path, &'static str) {
            let path = "foo.ddi".into();
            let json = r#"{
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
            }"#;
            (path, json)
        }

        pub fn main() -> (&'static r5::Utf8Path, &'static str) {
            let path = "main.ddi".into();
            let json = r#"{
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
            }"#;
            (path, json)
        }

        pub fn foo_cycle() -> (&'static r5::Utf8Path, &'static str) {
            let path = "foo.ddi".into();
            let json = r#"{
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
            }"#;
            (path, json)
        }

        pub fn bar_cycle() -> (&'static r5::Utf8Path, &'static str) {
            let path = "bar.ddi".into();
            let json = r#"{
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
            }"#;
            (path, json)
        }
    }
}
