use anyhow::Result;
use serde_json::Value;
use std::io::Write;

pub fn write_csv_row<W: Write>(writer: &mut W, values: &[String]) -> Result<()> {
    for (idx, value) in values.iter().enumerate() {
        if idx > 0 {
            writer.write_all(b",")?;
        }
        write_csv_cell(writer, value)?;
    }
    writer.write_all(b"\n")?;
    Ok(())
}

pub fn value_to_csv(value: Option<&Value>) -> String {
    match value {
        None | Some(Value::Null) => String::new(),
        Some(Value::String(s)) => s.clone(),
        Some(Value::Bool(b)) => b.to_string(),
        Some(Value::Number(n)) => n.to_string(),
        Some(v) => v.to_string(),
    }
}

fn write_csv_cell<W: Write>(writer: &mut W, raw: &str) -> Result<()> {
    let must_quote =
        raw.contains(',') || raw.contains('"') || raw.contains('\n') || raw.contains('\r');
    if must_quote {
        writer.write_all(b"\"")?;
        for b in raw.bytes() {
            if b == b'"' {
                writer.write_all(b"\"\"")?;
            } else {
                writer.write_all(&[b])?;
            }
        }
        writer.write_all(b"\"")?;
    } else {
        writer.write_all(raw.as_bytes())?;
    }
    Ok(())
}
