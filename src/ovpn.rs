extern crate reqwest;
extern crate tempfile;
extern crate zip;

use zip::ZipArchive;

use std::error::Error;
use std::fs;
use std::fs::File;
use std::io;
use std::io::BufReader;
use std::io::BufWriter;
use std::path;
use std::path::Path;

use crate::common;
use crate::common::Options;

pub fn handler_ovpn_zip(url: &str, opt: &Options) -> Result<(), Box<Error>> {
    let mut tmp_file = tempfile::tempfile()?;

    let base_folder = opt.base_folder;
    if !Path::new(base_folder).exists() {
        common::vprint(
            opt.verbose,
            format!("{} folder doesn't exist, creating a new one", &base_folder).as_str(),
        );
        fs::create_dir(&base_folder)?;
    }

    common::vprint(opt.verbose, "Starting ovpn download...");
    let _ = reqwest::get(url)?.copy_to(&mut tmp_file);
    common::vprint(opt.verbose, "Done with ovpn download");

    common::vprint(opt.verbose, "Starting with ovpn unzip...");
    let mut zip = ZipArchive::new(tmp_file)?;
    for i in 0..zip.len() {
        let file = zip.by_index(i)?;
        let file_name = file.name();
        if let Some(s) = opt.country.to_str() {
            let matches: Vec<_> = file_name.matches(s).collect();
            if matches.len() < 1 {
                continue;
            }
        }
        let split_name: Vec<_> = file_name.split(path::MAIN_SEPARATOR).collect();
        let new_file_path = Path::new(&file_name).strip_prefix(split_name.get(0).unwrap())?;
        let new_file_fullpath = Path::new(base_folder).join(new_file_path);
        let mut file_read = BufReader::new(file);
        let mut file_write = BufWriter::new(File::create(new_file_fullpath)?);
        let _ = io::copy(&mut file_read, &mut file_write);
    }
    common::vprint(opt.verbose, "Done with ovpn unzip");

    Ok(())
}
