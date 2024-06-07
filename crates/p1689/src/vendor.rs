#[cfg(feature = "camino")]
pub mod camino {
    pub type Utf8Path = ::camino::Utf8Path;
    pub type Utf8PathBuf = ::camino::Utf8PathBuf;
}

#[cfg(not(feature = "camino"))]
pub mod camino {
    use alloc::string::String;

    pub type Utf8Path = str;
    pub type Utf8PathBuf = String;
}
