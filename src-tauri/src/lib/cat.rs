use std::{error::Error, fs::File, num::NonZeroUsize, path::Path, time::Instant};

use polars::{
  frame::DataFrame,
  lazy::dsl::{functions::concat_lf_diagonal, lit},
  prelude::{
    CsvWriter, CsvWriterOptions, IntoLazy, LazyCsvReader, LazyFileListReader, LazyFrame, SerWriter,
    UnionArgs,
  },
};

use crate::{
  excel::{ExcelReader, ToPolarsDataFrame},
  xlsx_writer::write_xlsx,
};

fn concat_all(path: String, sep: String, memory: bool) -> Result<(), Box<dyn Error>> {
  /* concat csv and excel files into a xlsx or csv file */
  let sep = if sep == "\\t" {
    b'\t'
  } else {
    sep.into_bytes()[0]
  };

  let vec_path: Vec<&str> = path.split('|').collect();

  let mut lfs = Vec::new();

  for file in vec_path.iter() {
    let filename = Path::new(file).file_name().unwrap().to_str().unwrap();

    let file_extension = match Path::new(file).extension() {
      Some(ext) => ext.to_string_lossy().to_lowercase(),
      None => return Err(("File extension not found").into()),
    };

    let lf = match file_extension.as_str() {
      "parquet" => LazyFrame::scan_parquet(file, Default::default())?,
      "xls" | "xlsx" | "xlsm" | "xlsb" | "ods" => {
        let mut excel_reader = ExcelReader::new(file);
        let df: DataFrame = excel_reader.worksheet_range_at(0)?.to_df()?;
        df.lazy()
      }
      _ => {
        let csv_reader = LazyCsvReader::new(file)
          .with_has_header(true)
          .with_missing_is_null(true)
          .with_separator(sep)
          .with_infer_schema_length(Some(0))
          .with_low_memory(false)
          .finish()?;

        csv_reader
      }
    };

    let lf = lf.with_column(lit(filename).alias("FileName"));
    lfs.push(lf);
  }

  let parent_path = Path::new(&vec_path[0])
    .parent()
    .map(|parent| parent.to_string_lossy())
    .unwrap();
  let current_time = chrono::Local::now().format("%Y-%m-%d-%H%M%S");

  let cat_lf = concat_lf_diagonal(
    lfs,
    UnionArgs {
      parallel: true,
      rechunk: true,
      to_supertypes: true,
      diagonal: true,
      from_partitioned_ds: false,
    },
  )?;

  if memory {
    let mut cat_df = cat_lf.collect()?;
    let row_len = cat_df.shape().0;
    if row_len < 104_0000 {
      let save_path = format!("{}/cat_{}.xlsx", parent_path, current_time);
      write_xlsx(cat_df, save_path.into())?;
    } else {
      let save_path = format!("{}/cat_{}.csv", parent_path, current_time);
      CsvWriter::new(File::create(save_path)?)
        .with_separator(sep)
        .finish(&mut cat_df)?;
    }
  } else {
    let save_path = format!("{}/cat_{}.csv", parent_path, current_time);
    cat_lf.sink_csv(
      save_path,
      CsvWriterOptions {
        include_bom: false,
        include_header: true,
        batch_size: NonZeroUsize::new(1024).unwrap(),
        maintain_order: false,
        serialize_options: polars::prelude::SerializeOptions {
          date_format: None,
          time_format: None,
          datetime_format: None,
          float_scientific: None,
          float_precision: None,
          separator: sep,
          quote_char: b'"',
          null: String::new(),
          line_terminator: "\n".into(),
          quote_style: Default::default(),
        },
      },
    )?;
  }

  Ok(())
}

#[tauri::command]
pub async fn concat(file_path: String, sep: String, memory: bool, window: tauri::Window) {
  let start_time = Instant::now();

  match (async { concat_all(file_path, sep, memory) }).await {
    Ok(result) => result,
    Err(err) => {
      eprintln!("concat error: {err}");
      window.emit("cat_err", &err.to_string()).unwrap();
      err.to_string();
    }
  };

  let end_time = Instant::now();
  let elapsed_time = end_time.duration_since(start_time).as_secs_f64();
  let runtime = format!("{elapsed_time:.2} s");
  window.emit("runtime", runtime).unwrap();
}
