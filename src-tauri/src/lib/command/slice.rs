use std::{
  fs::File,
  io::{BufReader, BufWriter},
  path::Path,
  time::Instant,
};

use anyhow::{Result, anyhow};
use csv::{Reader, ReaderBuilder, Writer, WriterBuilder};

use crate::utils::{CsvOptions, Selection};

#[derive(Debug)]
pub enum SliceMode {
  Left,
  Right,
  StartStop,
  Nth,
  Nmax,
  None,
}

impl From<&str> for SliceMode {
  fn from(mode: &str) -> Self {
    match mode {
      "left" => SliceMode::Left,
      "right" => SliceMode::Right,
      "ss" => SliceMode::StartStop,
      "nth" => SliceMode::Nth,
      "nmax" => SliceMode::Nmax,
      _ => SliceMode::None,
    }
  }
}

impl SliceMode {
  fn to_str(&self) -> &'static str {
    match self {
      SliceMode::Left => "left",
      SliceMode::Right => "right",
      SliceMode::StartStop => "ss",
      SliceMode::Nth => "nth",
      SliceMode::Nmax => "nmax",
      SliceMode::None => "none",
    }
  }
}

pub async fn slice_column_with_nchar(
  mut rdr: Reader<BufReader<File>>,
  mut wtr: Writer<BufWriter<File>>,
  select_column: &str,
  n: usize,
  mode: &str,
) -> Result<()> {
  let headers = rdr.headers()?.clone();

  let sel = Selection::from_headers(rdr.byte_headers()?, &[select_column][..])?;

  let mut new_headers = headers.clone();
  let new_column_name = format!("{}_nchar", select_column);
  new_headers.push_field(&new_column_name);

  wtr.write_record(&new_headers)?;

  for result in rdr.records() {
    let record = result?;

    if let Some(value) = record.get(sel.first_indices()?) {
      let slice_n = if mode == "left" {
        value.chars().take(n).collect::<String>()
      } else {
        value
          .chars()
          .rev()
          .take(n)
          .collect::<String>()
          .chars()
          .rev()
          .collect::<String>()
      };

      let mut new_record = record.clone();
      new_record.push_field(&slice_n);

      wtr.write_record(&new_record)?;
    }
  }

  Ok(wtr.flush()?)
}

pub async fn slice_column_with_ss(
  mut rdr: Reader<BufReader<File>>,
  mut wtr: Writer<BufWriter<File>>,
  select_column: &str,
  start: usize,
  stop: usize,
) -> Result<()> {
  let headers = rdr.headers()?.clone();

  let sel = Selection::from_headers(rdr.byte_headers()?, &[select_column][..])?;

  let mut new_headers = headers.clone();
  let new_column_name = format!("{}_ss", select_column);
  new_headers.push_field(&new_column_name);

  wtr.write_record(&new_headers)?;

  for result in rdr.records() {
    let record = result?;

    if let Some(value) = record.get(sel.first_indices()?) {
      let start_index = start - 1;
      let end_index = stop.min(value.chars().count());

      let slice_nm = value
        .chars()
        .skip(start_index)
        .take(end_index - start_index)
        .collect::<String>();

      let mut new_record = record.clone();
      new_record.push_field(&slice_nm);

      wtr.write_record(&new_record)?;
    }
  }

  Ok(wtr.flush()?)
}

pub async fn slice_column_with_nth(
  mut rdr: Reader<BufReader<File>>,
  mut wtr: Writer<BufWriter<File>>,
  select_column: &str,
  n: usize,
  slice_sep: &str,
) -> Result<()> {
  let mut headers = rdr.headers()?.clone();

  let sel = Selection::from_headers(rdr.byte_headers()?, &[select_column][..])?;

  let new_column_name = format!("{}_nth", select_column);
  headers.push_field(&new_column_name);
  wtr.write_record(&headers)?;

  for result in rdr.records() {
    let record = result?;
    if let Some(value) = record.get(sel.first_indices()?) {
      let split_parts: Vec<&str> = value.split(slice_sep).collect();
      let selected_part = if split_parts.len() >= n {
        split_parts[n - 1]
      } else {
        ""
      };

      let mut new_record = record.clone();
      new_record.push_field(selected_part);
      wtr.write_record(&new_record)?;
    }
  }

  Ok(wtr.flush()?)
}

pub async fn slice_column_with_nmax(
  mut rdr: Reader<BufReader<File>>,
  mut wtr: Writer<BufWriter<File>>,
  select_column: &str,
  n: usize,
  slice_sep: &str,
) -> Result<()> {
  let mut headers = rdr.headers()?.clone();

  let sel = Selection::from_headers(rdr.byte_headers()?, &[select_column][..])?;

  let mut first_record = true;
  for result in rdr.records() {
    let record = result?;
    if let Some(value) = record.get(sel.first_indices()?) {
      let split_parts: Vec<&str> = value.split(slice_sep).collect();
      if first_record {
        for i in 1..=n {
          headers.push_field(&format!("{}_nmax{}", select_column, i));
        }
        wtr.write_record(&headers)?;
        first_record = false;
      }

      let mut new_record = record.clone();
      for i in 0..n {
        if i < split_parts.len() {
          new_record.push_field(split_parts[i]);
        } else {
          new_record.push_field("");
        }
      }

      wtr.write_record(&new_record)?;
    }
  }

  Ok(wtr.flush()?)
}

pub async fn perform_slice<P: AsRef<Path> + Send + Sync>(
  path: P,
  select_column: &str,
  n: usize,
  m: usize,
  slice_sep: &str,
  mode: SliceMode,
) -> Result<()> {
  if n < 1 {
    return Err(anyhow!("slice/start must be greater than 1"));
  } else if n > m && mode.to_str() == "ss" {
    return Err(anyhow!("stop must be greater than or equal to start"));
  }

  let csv_options = CsvOptions::new(&path);
  let sep = csv_options.detect_separator()?;

  let parent_path = path.as_ref().parent().unwrap().to_str().unwrap();
  let file_stem = path.as_ref().file_stem().unwrap().to_str().unwrap();
  let output_path = format!("{parent_path}/{file_stem}.slice.csv");

  let rdr = ReaderBuilder::new()
    .delimiter(sep)
    .from_reader(csv_options.skip_csv_rows()?);

  let buf_writer = BufWriter::with_capacity(256_000, File::create(output_path)?);
  let wtr = WriterBuilder::new().delimiter(sep).from_writer(buf_writer);

  match mode {
    SliceMode::Left => {
      slice_column_with_nchar(rdr, wtr, select_column, n, SliceMode::Left.to_str()).await?
    }
    SliceMode::Right => {
      slice_column_with_nchar(rdr, wtr, select_column, n, SliceMode::Right.to_str()).await?
    }
    SliceMode::StartStop => slice_column_with_ss(rdr, wtr, select_column, n, m).await?,
    SliceMode::Nth => slice_column_with_nth(rdr, wtr, select_column, n, slice_sep).await?,
    SliceMode::Nmax => slice_column_with_nmax(rdr, wtr, select_column, n, slice_sep).await?,
    SliceMode::None => {}
  }

  Ok(())
}

#[tauri::command]
pub async fn slice(
  path: String,
  select_column: String,
  n: String,
  m: String,
  slice_sep: String,
  mode: String,
) -> Result<String, String> {
  let start_time = Instant::now();

  let slice_mode: SliceMode = mode.as_str().into();

  match perform_slice(
    path,
    select_column.as_str(),
    n.parse::<usize>().map_err(|e| e.to_string())?,
    m.parse::<usize>().map_err(|e| e.to_string())?,
    slice_sep.as_str(),
    slice_mode,
  )
  .await
  {
    Ok(_) => {
      let end_time = Instant::now();
      let elapsed_time = end_time.duration_since(start_time).as_secs_f64();
      Ok(format!("{elapsed_time:.2}"))
    }
    Err(err) => Err(format!("{err}")),
  }
}
