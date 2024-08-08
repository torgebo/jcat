//! Functionality for cat command
use crate::compress::{read_from_ctype, write_from_ctype, CType, ContentType};
use crate::typ::Data;
use anyhow::{Context, Result};
use std::fs::File;
use std::io::{BufReader, BufWriter};
use std::path::{Path, PathBuf};

pub fn cat_dir<T>(files_dir: &Path, outfile_dir: &Path, write_compression: &CType) -> Result<()>
where
    T: 'static + serde::de::DeserializeOwned + serde::Serialize,
{
    let mut elements: Data<T> = Data::default();

    let mut paths: Vec<PathBuf> = std::fs::read_dir(files_dir)
        .with_context(|| format!("unable to read directory {files_dir:?}"))?
        .map(|de| de.map(|p| p.path()))
        .filter(|resp| resp.as_ref().map_or(true, |p| p.is_file()))
        .collect::<Result<Vec<_>, _>>()?;
    paths.sort();

    for path in paths {
        let f =
            File::open(&path).with_context(|| format!("unable to open file at path {path:?}"))?;
        let br = BufReader::new(f);

        if let ContentType::Json(ctype) = ContentType::from(&path) {
            let decoder = read_from_ctype(br, &ctype);
            let entry: T = serde_json::from_reader(decoder)
                .with_context(|| format!("unable to read json from opened file {path:?}"))?;
            elements.data.push(entry);
        }
    }

    if elements.data.is_empty() {
        return Ok(());
    }

    let outfile_path = get_output_path(outfile_dir, write_compression);
    let out_file = File::create(&outfile_path)
        .with_context(|| format!("unable to create file {outfile_path:?}"))?;
    let buf_writer = BufWriter::new(out_file);
    let encoder = write_from_ctype(buf_writer, write_compression);
    serde_json::to_writer(encoder, &elements)
        .with_context(|| format!("unable to write json to opened file {outfile_path:?}"))
}

/// Given output directory, calculate path and extension
fn get_output_path(outfile_dir: &Path, ct: &CType) -> PathBuf {
    let base = "out";
    let mut outfile_path = outfile_dir.join(base);
    outfile_path.set_extension(ct.get_extension());
    outfile_path
}

pub fn compute_outdir_paths(
    len_hint: usize,
    base_in_dir: &Path,
    base_out_dir: &Path,
    process_paths: &[PathBuf],
) -> Result<Vec<PathBuf>> {
    let mut outdir_paths: Vec<PathBuf> = Vec::<PathBuf>::with_capacity(len_hint);

    for path in process_paths.iter() {
        let rel_path = &path
            .strip_prefix(base_in_dir)
            .with_context(|| format!("unable to strip prefix {base_in_dir:?} of path {path:?}"))?;
        let outdir_path = base_out_dir.join(rel_path);
        outdir_paths.push(outdir_path);
    }

    Ok(outdir_paths)
}

pub fn collect_paths(in_dir: &Path) -> Result<Vec<PathBuf>> {
    let mut to_process: Vec<PathBuf> = Vec::<_>::with_capacity(16);
    let mut to_expand: Vec<PathBuf> = Vec::<_>::with_capacity(16);
    to_expand.push(in_dir.to_path_buf());
    while let Some(cur_dir) = to_expand.pop() {
        let mut subdirs: Vec<PathBuf> = cur_dir
            .read_dir()
            .with_context(|| format!("unable to read {cur_dir:?}"))?
            .map(|rde| rde.map(|de| de.path()))
            .filter(|resp| resp.as_ref().map_or(true, |p| p.is_dir()))
            .collect::<Result<Vec<_>, _>>()?;
        subdirs.sort();
        to_expand.extend_from_slice(&subdirs);
        to_process.push(cur_dir);
    }

    Ok(to_process)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn get_output_path_by_extensions() {
        let base_path = Path::new("/tmp");
        let ctypes = vec![CType::Raw, CType::Gzip, CType::Snappy];
        let answers = vec!["/tmp/out.json", "/tmp/out.json.gz", "/tmp/out.json.sz"];
        for (ct, ans) in ctypes.iter().zip(answers) {
            let out_path = get_output_path(base_path, &ct);
            assert_eq!(
                out_path.to_str().expect("test: utf-8 deserializable path"),
                ans
            );
        }
    }
}
