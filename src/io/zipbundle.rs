// src/io/zipbundle.rs -- I/O on files in a Zipped-up "bundle"
// Copyright 2016 the Tectonic Project
// Licensed under the MIT License.

use std::ffi::OsStr;
use std::fs::File;
use std::io::{Cursor, Read, Seek};
use std::path::Path;
use zip::result::ZipError;
use zip::ZipArchive;

use errors::Result;
use super::{InputHandle, InputOrigin, IoProvider, OpenResult};
use status::StatusBackend;


pub struct ZipBundle<R: Read + Seek> {
    zip: ZipArchive<R>
}


impl<R: Read + Seek> ZipBundle<R> {
    pub fn new (reader: R) -> Result<ZipBundle<R>> {
        Ok(ZipBundle {
            zip: ZipArchive::new(reader)?
        })
    }
}


impl ZipBundle<File> {
    pub fn open (path: &Path) -> Result<ZipBundle<File>> {
        Self::new(File::open(path)?)
    }
}


impl<R: Read + Seek> IoProvider for ZipBundle<R> {
    fn input_open_name(&mut self, name: &OsStr, _status: &mut StatusBackend) -> OpenResult<InputHandle> {
        // We need to be able to look at other items in the Zip file while
        // reading this one, so the only path forward is to read the entire
        // contents into a buffer right now. RAM is cheap these days.

        // If `name` cannot be converted to Unicode, we return NotAvailable. I
        // *think* that's what we should do.

        let namestr = match name.to_str() {
            Some(s) => s,
            None => return OpenResult::NotAvailable
        };

        let mut zipitem = match self.zip.by_name (namestr) {
            Ok(f) => f,
            Err(e) => {
                return match e {
                    ZipError::Io(sube) => OpenResult::Err(sube.into()),
                    ZipError::FileNotFound => OpenResult::NotAvailable,
                    _ => OpenResult::Err(e.into()),
                }
            }
        };

        let mut buf = Vec::with_capacity(zipitem.size() as usize);

        if let Err(e) = zipitem.read_to_end(&mut buf) {
            return OpenResult::Err(e.into());
        }

        OpenResult::Ok(InputHandle::new(name, Cursor::new(buf), InputOrigin::Other))
    }
}
