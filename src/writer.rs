use crate::{Error, Result};
use std::fs::{File, OpenOptions};
use std::io::{self, Write};
use std::path::Path;

#[derive(Debug, Default)]
pub struct WriterBuilder<'a> {
    stdout_append: bool,
    stdout: Option<&'a Path>,
    stderr_append: bool,
    stderr: Option<&'a Path>,
}

impl<'a> WriterBuilder<'a> {
    pub fn stdout_new<P: AsRef<Path>>(self, path: &'a P) -> Self {
        Self {
            stdout_append: false,
            stdout: Some(path.as_ref()),
            ..self
        }
    }

    pub fn stdout_append<P: AsRef<Path>>(self, path: &'a P) -> Self {
        Self {
            stdout_append: true,
            stdout: Some(path.as_ref()),
            ..self
        }
    }

    pub fn stderr_new<P: AsRef<Path>>(self, path: &'a P) -> Self {
        Self {
            stderr_append: false,
            stderr: Some(path.as_ref()),
            ..self
        }
    }

    pub fn stderr_append<P: AsRef<Path>>(self, path: &'a P) -> Self {
        Self {
            stderr_append: true,
            stderr: Some(path.as_ref()),
            ..self
        }
    }

    pub fn build(self) -> Result<Writer> {
        let Self {
            stdout_append,
            stdout,
            stderr_append,
            stderr,
        } = self;

        Ok(Writer {
            stdout: stdout.map(open(stdout_append)).transpose()?,
            stderr: stderr.map(open(stderr_append)).transpose()?,
        })
    }
}

type OpenFile = Box<dyn Fn(&Path) -> Result<File>>;

fn open(append: bool) -> OpenFile {
    Box::new(move |path: &Path| {
        OpenOptions::new()
            .write(true)
            .append(append)
            .create(true)
            .truncate(!append)
            .open(path)
            .map_err(Error::from)
    })
}

#[derive(Debug)]
pub struct Writer {
    stdout: Option<File>,
    stderr: Option<File>,
}

impl Writer {
    pub fn builder<'a>() -> WriterBuilder<'a> {
        WriterBuilder::default()
    }

    pub fn write<T: AsRef<[u8]>>(&mut self, buf: T) -> Result<()> {
        if let Some(f) = self.stdout.as_mut() {
            f.write_all(buf.as_ref())?;
        } else {
            io::stdout().write_all(buf.as_ref())?;
        }
        Ok(())
    }

    pub fn writeln<T: AsRef<[u8]>>(&mut self, buf: T) -> Result<()> {
        self.write(buf)?;
        self.write(b"\n")
    }

    pub fn ewrite<T: AsRef<[u8]>>(&mut self, buf: T) -> Result<()> {
        if let Some(f) = self.stderr.as_mut() {
            f.write_all(buf.as_ref())?;
        } else {
            io::stderr().write_all(buf.as_ref())?;
        }
        Ok(())
    }

    pub fn ewriteln<T: AsRef<[u8]>>(&mut self, buf: T) -> Result<()> {
        self.ewrite(buf)?;
        self.ewrite(b"\n")
    }
}
