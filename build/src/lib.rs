#[derive(Debug, Clone, Default)]
pub struct Build {
    compiler: Option<String>,
    filenames: Vec<std::path::PathBuf>,
    flags: Vec<String>,
}

pub const TARGET: &str = include_str!(concat!(env!("OUT_DIR"), "/../output"));

impl Build {
    pub fn new() -> Build {
        Build::default()
    }

    pub fn file<S: AsRef<std::path::Path>>(&mut self, path: S) -> &mut Build {
        self.filenames.push(path.as_ref().to_path_buf());
        self
    }

    pub fn flag<S: AsRef<str>>(&mut self, path: S) -> &mut Build {
        self.flags.push(path.as_ref().to_owned());
        self
    }

    pub fn compiler<S: AsRef<str>>(&mut self, name: S) -> &mut Build {
        self.compiler = Some(name.as_ref().to_owned());
        self
    }

    pub fn link(&self, name: &str) {
        let mut build = cc::Build::new();
        build.files(&self.filenames);

        for flag in self.flags.iter() {
            build.flag(flag);
        }

        match &self.compiler {
            Some(c) => build.compiler(c),
            None => build.compiler("clang"),
        };

        build.compile(name);
    }
}

#[cfg(test)]
mod tests {
    use crate::*;

    #[test]
    fn it_works() {
        let _ = std::fs::create_dir("./test");
        std::env::set_var("OUT_DIR", "test");
        std::env::set_var("TARGET", TARGET);
        std::env::set_var("OPT_LEVEL", "");

        Build::new().file("test.ll").link("test");

        let lib = std::path::PathBuf::from("./test/libtest.a");
        assert!(lib.exists());

        let obj = std::path::PathBuf::from("./test/test.o");
        assert!(obj.exists())
    }
}
