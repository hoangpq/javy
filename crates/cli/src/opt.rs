use wizer::Wizer;
use anyhow::{Error, Result, bail};
use tempfile::NamedTempFile;
use std::{fs, io::Write};
use std::process::{Command, ExitStatus};

pub(crate) struct Optimizer {
    pub wasm: Vec<u8>,
}

impl Optimizer {
    pub fn new(wasm: &[u8]) -> Self {
        Self { wasm: Vec::from(wasm) }
    }

    pub fn initialize(&mut self) -> Result<&mut Self, Error> {
        self.wasm = Wizer::new()
            .allow_wasi(true)
            .inherit_env(true)
            .dir(".")
            .run(&self.wasm)?;
        Ok(self)
    }

    pub fn strip(&mut self) -> Result<&mut Self, Error> {
        let mut file = NamedTempFile::new()?;
        file.write_all(&self.wasm)?;

        let output = Command::new("wasm-strip")
            .arg(&file.path())
            .output()?;

        if !output.status.success() {
            bail!(format!("Couldn't apply wasm-strip: {:?}", output.stderr));
        }

        self.wasm = fs::read(file.path())?;

        Ok(self)
    }

    // TODO
    // Add setters that better represent the optimization passes
    // and apply them via `wasm-opt` when requesting the final binary
    pub fn passes(&mut self) -> Result<&mut Self, Error> {
        let mut file = NamedTempFile::new()?;
        let out_file = file.path().with_extension("wasm-opt.wasm");

        file.write_all(&self.wasm)?;

        let output = Command::new("wasm-opt")
            .arg(file.path())
            .arg("-O3")
            .arg("--dce")
            .arg("-o")
            .arg(&out_file)
            .output()?;

        if !output.status.success() {
            bail!(format!("Couldn't apply wasm-opt: {:?}", output.stderr));
        }

        self.wasm = fs::read(out_file)?;
        Ok(self)
    }

    pub fn wasm(&self) -> Vec<u8> {
        self.wasm.clone()
    }
}