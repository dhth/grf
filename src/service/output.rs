use crate::domain::OutputFormat;
use anyhow::Context;
use chrono::{DateTime, Utc};
use serde_json::Value;
use std::fs::File;
use std::io::Write;
use std::path::{Path, PathBuf};

pub fn write_results<P>(
    results: &[Value],
    results_directory: P,
    format: &OutputFormat,
    reference_time: DateTime<Utc>,
) -> anyhow::Result<PathBuf>
where
    P: AsRef<Path>,
{
    std::fs::create_dir_all(&results_directory).with_context(|| {
        format!(
            "failed to create results directory: {}",
            results_directory.as_ref().to_string_lossy()
        )
    })?;

    let file_name = reference_time.format("%m-%d-%H-%M-%S");
    let output_file_path =
        results_directory
            .as_ref()
            .join(format!("{}.{}", file_name, format.extension()));

    let file = File::create(&output_file_path).with_context(|| {
        format!(
            "couldn't open output file: {}",
            output_file_path.to_string_lossy()
        )
    })?;

    match format {
        OutputFormat::Csv => write_csv(results, file)?,
        OutputFormat::Json => write_json(results, file)?,
    }

    Ok(output_file_path)
}

fn write_csv<W>(results: &[Value], writer: W) -> anyhow::Result<()>
where
    W: Write,
{
    if results.is_empty() {
        return Ok(());
    }

    let mut csv_writer = csv::Writer::from_writer(writer);

    let Some(first) = results.first().and_then(|v| v.as_object()) else {
        anyhow::bail!("expected results to be an array of objects");
    };

    let headers: Vec<&str> = first.keys().map(|s| s.as_str()).collect();
    csv_writer.write_record(&headers)?;

    for result in results {
        let Some(obj) = result.as_object() else {
            anyhow::bail!("expected each result to be an object");
        };

        let row: Vec<String> = headers
            .iter()
            .map(|&header| {
                obj.get(header)
                    .map(value_to_csv_field)
                    .unwrap_or_default()
            })
            .collect();

        csv_writer.write_record(&row)?;
    }

    csv_writer.flush()?;
    Ok(())
}

fn value_to_csv_field(value: &Value) -> String {
    match value {
        Value::Null => String::new(),
        Value::Bool(b) => b.to_string(),
        Value::Number(n) => n.to_string(),
        Value::String(s) => s.clone(),
        _ => serde_json::to_string(value).unwrap_or_default(),
    }
}

fn write_json<W>(results: &[Value], mut writer: W) -> anyhow::Result<()>
where
    W: Write,
{
    let json_string =
        serde_json::to_string_pretty(results).context("couldn't serialize results to JSON")?;
    writer
        .write_all(json_string.as_bytes())
        .context("couldn't write bytes to file")?;

    Ok(())
}
