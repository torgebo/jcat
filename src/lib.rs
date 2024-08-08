//! Entry point functions
use anyhow::{Context, Result};
use indicatif::ParallelProgressIterator;
use rayon::iter::{IntoParallelRefIterator, ParallelIterator};
use rayon::prelude::*;
use std::path::PathBuf;

mod cat;
pub mod compress;
mod typ;

/// Run concatenation on all files in `base_in_dir` and write to `out_dir`.
/// If `recursive` is true, it will produce one output file for each subdirectory
/// of `base_in_dir` where `.json` content is found.
/// Otherwise, it will act on the files (directly) within the given folder.
///
/// If you want to concatenate the JSON objects as-provided, call this function
/// for T=serde_json::Value (as is done from `main` entrypoint).
/// However, by providing a custom type, you can implement custom validation and extraction.
pub fn run_cat<T>(
    base_in_dir: PathBuf,
    out_dir: PathBuf,
    recursive: bool,
    write_compression: compress::CType,
) -> Result<()>
where
    T: 'static + serde::de::DeserializeOwned + serde::Serialize,
{
    let process_paths: Vec<PathBuf> = if recursive {
        cat::collect_paths(&base_in_dir).with_context(|| format!("no such path {base_in_dir:?}"))?
    } else {
        vec![base_in_dir.clone()]
    };

    let outdir_paths =
        cat::compute_outdir_paths(process_paths.len(), &base_in_dir, &out_dir, &process_paths)?;

    for outdir_path in outdir_paths.iter() {
        if !std::fs::exists(outdir_path)
            .with_context(|| format!("unable to check existence of {outdir_path:?}"))?
        {
            std::fs::create_dir_all(outdir_path)
                .with_context(|| format!("unable to recursively create dir {outdir_path:?}"))?;
        }
    }

    let p_in = process_paths.par_iter();
    let p_out = outdir_paths.par_iter();

    p_in.zip(p_out)
        .progress_count(process_paths.len() as u64)
        .map(|(dir_in, dir_out)| {
            cat::cat_dir::<T>(dir_in, dir_out, &write_compression)
                .with_context(
		    || format!("Failed cat-ing from dir {dir_in:?} to {dir_out:?} using compression {write_compression:?}"))
        }).collect::<Result<Vec<_>, _>>()?;
    Ok(())
}

pub fn run_data_definition() -> Result<()> {
    let df = include_str!("./typ.rs");
    println!("{df}");
    Ok(())
}
