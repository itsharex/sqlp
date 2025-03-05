use std::{fs::File, path::Path, time::Instant};

use anyhow::Result;
use csv::{ByteRecord, ReaderBuilder};
use tauri::{Emitter, Window};

use crate::utils::CsvOptions;

pub async fn count_rows<P: AsRef<Path> + Send + Sync>(path: P) -> Result<u64> {
  let csv_options = CsvOptions::new(&path);

  let count = match csv_options.indexed()? {
    Some(idx) => idx.count(),
    None => count_record(&path, true, &csv_options)?,
  };

  Ok(count)
}

/// Used to check for counting errors caused by double quotation marks in CSV files
pub async fn count_check<P: AsRef<Path> + Send + Sync>(path: P) -> Result<u64> {
  let csv_options = CsvOptions::new(&path);
  let count_count_record_true = count_record(&path, true, &csv_options)?;
  let count_count_record_false = count_record(&path, false, &csv_options)?;

  let max_count = std::cmp::max(count_count_record_true, count_count_record_false);
  let min_count = std::cmp::min(count_count_record_true, count_count_record_false);

  Ok(max_count - min_count)
}

fn count_record<P: AsRef<Path> + Send + Sync>(
  path: P,
  quoting: bool,
  csv_options: &CsvOptions<P>,
) -> Result<u64> {
  let mut rdr = ReaderBuilder::new()
    .delimiter(csv_options.detect_separator()?)
    .quoting(quoting)
    .from_reader(File::open(&path)?);

  let mut record = ByteRecord::new();
  let mut count: u64 = 0;
  while rdr.read_byte_record(&mut record)? {
    count += 1;
  }

  Ok(count)
}

#[tauri::command]
pub async fn count(path: String, mode: String, window: Window) -> Result<String, String> {
  let start_time = Instant::now();
  let paths: Vec<&str> = path.split('|').collect();

  for file in paths.iter() {
    let filename = Path::new(file).file_name().unwrap().to_str().unwrap();
    window
      .emit("start_convert", &filename)
      .map_err(|e| e.to_string())?;
    let inner_time = Instant::now();
    match mode.as_str() {
      "index" => match crate::idx::create_index(file).await {
        Ok(_) => {
          let end_time = Instant::now();
          let elapsed_time = end_time.duration_since(inner_time).as_secs_f64();
          window
            .emit("count_msg", format!("{filename}|{elapsed_time:.2} s"))
            .map_err(|e| e.to_string())?;
        }
        Err(err) => {
          window
            .emit("count_err", format!("{filename}|{err}"))
            .map_err(|e| e.to_string())?;
        }
      },
      "count" => match count_rows(file).await {
        Ok(cnt) => {
          window
            .emit("count_msg", format!("{filename}|{cnt}"))
            .map_err(|e| e.to_string())?;
        }
        Err(err) => {
          window
            .emit("count_err", format!("{filename}|{err}"))
            .map_err(|e| e.to_string())?;
        }
      },
      _ => match count_check(file).await {
        Ok(cnt) => {
          window
            .emit("count_msg", format!("{filename}|{cnt}"))
            .map_err(|e| e.to_string())?;
        }
        Err(err) => {
          window
            .emit("count_err", format!("{filename}|{err}"))
            .map_err(|e| e.to_string())?;
        }
      },
    }
  }

  let end_time = Instant::now();
  let elapsed_time = end_time.duration_since(start_time).as_secs_f64();
  Ok(format!("{elapsed_time:.2}"))
}
