use std::sync::Arc;

use p1689::r5;

#[non_exhaustive]
pub enum CompilerError {
    UnknownFamily,
    VarError {
        var: std::ffi::OsString,
        err: std::env::VarError,
    },
}

#[non_exhaustive]
pub struct Compiler {
    pub tool: crate::vendor::cc::Tool,
    pub family: CompilerFamily,
    pub out_dir: Arc<r5::Utf8PathBuf>,
}
impl Compiler {
    pub fn new(cc: &crate::vendor::cc::Build) -> Result<Self, CompilerError> {
        let tool = cc.get_compiler();
        let family = CompilerFamily::try_from(&tool)?;
        let out_dir = std::env::var("OUT_DIR").map_err(|err| {
            let var = std::ffi::OsString::from("OUT_DIR");
            CompilerError::VarError { var, err }
        })?;
        let out_dir = r5::Utf8PathBuf::from(out_dir);
        let out_dir = Arc::from(out_dir);
        Ok(Self { tool, family, out_dir })
    }

    pub fn dep_file_ext(&self) -> &str {
        match self.family {
            CompilerFamily::Clang => "ddi",
            CompilerFamily::Gnu => "ddi",
            CompilerFamily::Msvc => todo!(),
        }
    }

    pub fn obj_file_ext(&self) -> &str {
        match self.family {
            CompilerFamily::Clang => "o",
            CompilerFamily::Gnu => "o",
            CompilerFamily::Msvc => todo!(),
        }
    }
}

impl core::fmt::Debug for CompilerError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "CompilerError::")?;
        match self {
            CompilerError::UnknownFamily => write!(f, "UnknownFamily")?,
            CompilerError::VarError { var, err } => f
                .debug_struct("VarError")
                .field("var", &var)
                .field("err", &err)
                .finish()?,
        }
        Ok(())
    }
}
impl core::fmt::Display for CompilerError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Could not detect compiler family")
    }
}
impl std::error::Error for CompilerError {}

#[derive(Clone, Copy)]
pub enum CompilerFamily {
    Clang,
    Gnu,
    Msvc,
}
#[cfg(feature = "cc")]
impl TryFrom<&crate::vendor::cc::Tool> for CompilerFamily {
    type Error = CompilerError;

    fn try_from(tool: &crate::vendor::cc::Tool) -> Result<Self, Self::Error> {
        if tool.is_like_gnu() {
            return Ok(CompilerFamily::Gnu);
        }
        if tool.is_like_clang() {
            return Ok(CompilerFamily::Clang);
        }
        if tool.is_like_msvc() {
            return Ok(CompilerFamily::Msvc);
        }
        Err(CompilerError::UnknownFamily)
    }
}
