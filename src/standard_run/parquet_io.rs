use super::{FORMAT_VERSION, RayRow, RecordedRun, Result};
use crate::{compute::RayStatus, simulation::rays::RayInitialCondition};
use parquet::{
    basic::{Compression, ConvertedType, LogicalType},
    data_type::{ByteArray, ByteArrayType, DoubleType, Int64Type},
    file::{
        metadata::KeyValue,
        properties::WriterProperties,
        reader::{FileReader, SerializedFileReader},
        writer::SerializedFileWriter,
    },
    record::Field,
    schema::{parser::parse_message_type, types::TypePtr},
};
use std::{
    fs::{File, OpenOptions},
    path::Path,
    sync::Arc,
};
const SCHEMA: &str = "message kerr_simulation {
 REQUIRED INT64 ray_id (UINT_64);
 REQUIRED DOUBLE position_x; REQUIRED DOUBLE position_y; REQUIRED DOUBLE position_z;
 REQUIRED DOUBLE direction_x; REQUIRED DOUBLE direction_y; REQUIRED DOUBLE direction_z;
 REQUIRED DOUBLE t_emit; REQUIRED BINARY status (UTF8);
 OPTIONAL DOUBLE u_hit; OPTIONAL DOUBLE v_hit; OPTIONAL DOUBLE t_hit;
}";
fn status(s: RayStatus) -> &'static str {
    match s {
        RayStatus::Active => "ACT",
        RayStatus::Detected => "DET",
        RayStatus::Captured => "CAP",
        RayStatus::Escaped => "ESC",
        RayStatus::NumericalFailure => "NUM",
    }
}
fn parse_status(s: &str) -> Result<RayStatus> {
    Ok(match s {
        // Historical archives keep their long enum names; new writes use codes.
        "ACT" | "Active" => RayStatus::Active,
        "DET" | "Detected" => RayStatus::Detected,
        "CAP" | "Captured" => RayStatus::Captured,
        "ESC" | "Escaped" => RayStatus::Escaped,
        "NUM" | "NumericalFailure" => RayStatus::NumericalFailure,
        _ => return Err(format!("unknown status {s}").into()),
    })
}
/// UINT_64/UTF8 converted annotations and INTEGER(64,false)/STRING logical
/// annotations have identical meanings. PyArrow writes the latter on a shuffle.
/// Still require exactly these flat columns, physical types and nullability.
fn matches_schema(actual: &[TypePtr], expected: &[TypePtr]) -> bool {
    actual.len() == expected.len()
        && actual.iter().zip(expected).all(|(a, e)| {
            if !a.is_primitive() || !e.is_primitive() {
                return false;
            }
            let ai = a.get_basic_info();
            let ei = e.get_basic_info();
            a.name() == e.name()
                && a.get_physical_type() == e.get_physical_type()
                && ai.repetition() == ei.repetition()
                && ai.converted_type() == ei.converted_type()
                && match ai.logical_type_ref() {
                    None => true,
                    Some(LogicalType::String) => ei.converted_type() == ConvertedType::UTF8,
                    Some(LogicalType::Integer(i)) => {
                        ei.converted_type() == ConvertedType::UINT_64
                            && i.bit_width == 64
                            && !i.is_signed
                    }
                    _ => false,
                }
        })
}
pub(super) fn write(path: &Path, run: &RecordedRun) -> Result<()> {
    let file = OpenOptions::new().write(true).create_new(true).open(path)?;
    // Parquet key/value metadata is UTF-8 strings, not typed scalar slots.
    // f64::to_string is a shortest round-trip representation, not reduced precision.
    let props = WriterProperties::builder()
        .set_compression(Compression::SNAPPY)
        .set_key_value_metadata(Some(vec![
            KeyValue::new("chi".into(), run.chi.to_string()),
            KeyValue::new("chi_type".into(), "float64".to_string()),
            KeyValue::new("data_format_version".into(), FORMAT_VERSION.to_string()),
        ]))
        .build();
    let mut writer =
        SerializedFileWriter::new(file, Arc::new(parse_message_type(SCHEMA)?), Arc::new(props))?;
    for rows in run.rows.chunks(8192) {
        let mut group = writer.next_row_group()?;
        for column in 0..12 {
            let mut col = group.next_column()?.ok_or("missing Parquet column")?;
            if column == 0 {
                // Parquet stores uint64 as INT64 bits with an unsigned annotation.
                let values: Vec<_> = rows
                    .iter()
                    .map(|r| r.ray_id.map(|id| id as i64).ok_or("missing ray_id"))
                    .collect::<std::result::Result<_, _>>()?;
                col.typed::<Int64Type>().write_batch(&values, None, None)?;
            } else if column == 8 {
                let values: Vec<_> = rows
                    .iter()
                    .map(|r| ByteArray::from(status(r.status)))
                    .collect();
                col.typed::<ByteArrayType>()
                    .write_batch(&values, None, None)?;
            } else if column < 8 {
                let values: Vec<_> = rows
                    .iter()
                    .map(|r| {
                        let c = r.input;
                        [
                            c.position_x,
                            c.position_y,
                            c.position_z,
                            c.direction_x,
                            c.direction_y,
                            c.direction_z,
                            c.t_emit,
                        ][column - 1]
                    })
                    .collect();
                col.typed::<DoubleType>().write_batch(&values, None, None)?;
            } else {
                let values: Vec<_> = rows
                    .iter()
                    .filter_map(|r| r.hit.map(|h| h[column - 9]))
                    .collect();
                let definitions: Vec<i16> =
                    rows.iter().map(|r| i16::from(r.hit.is_some())).collect();
                col.typed::<DoubleType>()
                    .write_batch(&values, Some(&definitions), None)?;
            }
            col.close()?;
        }
        group.close()?;
    }
    writer.finish()?;
    writer.inner().sync_all()?;
    Ok(())
}
pub(super) fn read(path: &Path) -> Result<(f64, Vec<RayRow>)> {
    let reader = SerializedFileReader::new(File::open(path)?)?;
    let meta = reader.metadata().file_metadata();
    let schema = parse_message_type(SCHEMA)?;
    let fields = meta.schema_descr().root_schema().get_fields();
    // Exact historical schemas remain readable; legacy energy must still be 1.
    let legacy = parse_message_type(&SCHEMA.replace(
        "REQUIRED DOUBLE t_emit",
        "REQUIRED DOUBLE energy; REQUIRED DOUBLE t_emit",
    ))?;
    let current = matches_schema(fields, schema.get_fields());
    let legacy_ids = matches_schema(fields, legacy.get_fields());
    let legacy_no_ids = matches_schema(fields, &legacy.get_fields()[1..]);
    let has_ids = current || legacy_ids;
    let has_energy = legacy_ids || legacy_no_ids;
    if !current && !legacy_ids && !legacy_no_ids {
        return Err(
            "unsupported Parquet schema: exact uint64/float64/UTF8/nullable columns required"
                .into(),
        );
    }
    let metadata = meta.key_value_metadata().ok_or("missing file metadata")?;
    let value = |name: &str| -> Result<&str> {
        let values: Vec<_> = metadata.iter().filter(|v| v.key == name).collect();
        if values.len() != 1 {
            return Err(format!("missing or duplicate metadata {name}").into());
        }
        values[0]
            .value
            .as_deref()
            .ok_or_else(|| format!("null metadata {name}").into())
    };
    if value("data_format_version")? != FORMAT_VERSION || value("chi_type")? != "float64" {
        return Err("unsupported Parquet metadata version/type".into());
    }
    let chi: f64 = value("chi")?.parse()?;
    crate::physics::kerr::Kerr::new(chi)?;
    let mut rows = Vec::new();
    for row in reader.get_row_iter(None)? {
        let row = row?;
        let fields: Vec<_> = row.get_column_iter().map(|(_, f)| f).collect();
        let ray_id = if has_ids {
            match fields[0] {
                Field::ULong(id) => Some(*id),
                _ => return Err("expected non-null uint64 ray_id".into()),
            }
        } else {
            None
        };
        let fields = &fields[usize::from(has_ids)..];
        let number = |i: usize| -> Result<f64> {
            match fields[i] {
                Field::Double(v) => Ok(*v),
                _ => Err("expected non-null float64".into()),
            }
        };
        let status_column = 7 + usize::from(has_energy);
        let mut hit = [0.0; 3];
        let mut present = 0;
        for i in 0..3 {
            match fields[status_column + 1 + i] {
                Field::Double(v) => {
                    hit[i] = *v;
                    present += 1;
                }
                Field::Null => {}
                _ => return Err("invalid nullable hit type".into()),
            }
        }
        if present != 0 && present != 3 {
            return Err("partial detector hit nulls".into());
        }
        let status = match fields[status_column] {
            Field::Str(s) => parse_status(s)?,
            _ => return Err("invalid status type".into()),
        };
        rows.push(RayRow {
            ray_id,
            input: RayInitialCondition {
                position_x: number(0)?,
                position_y: number(1)?,
                position_z: number(2)?,
                direction_x: number(3)?,
                direction_y: number(4)?,
                direction_z: number(5)?,
                t_emit: number(6 + usize::from(has_energy))?,
            },
            energy: if has_energy { number(6)? } else { 1.0 },
            status,
            hit: (present == 3).then_some(hit),
        });
    }
    if rows.len() as i64 != meta.num_rows() {
        return Err("Parquet row count mismatch".into());
    }
    Ok((chi, rows))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn short_and_historical_status_strings_restore_the_same_enum() {
        for (code, legacy, expected) in [
            ("ACT", "Active", RayStatus::Active),
            ("DET", "Detected", RayStatus::Detected),
            ("CAP", "Captured", RayStatus::Captured),
            ("ESC", "Escaped", RayStatus::Escaped),
            ("NUM", "NumericalFailure", RayStatus::NumericalFailure),
        ] {
            assert_eq!(parse_status(code).unwrap(), expected);
            assert_eq!(parse_status(legacy).unwrap(), expected);
        }
        for invalid in ["", "UNKNOWN", "det", "DET "] {
            assert!(parse_status(invalid).is_err());
        }
    }

    #[test]
    fn equivalent_arrow_annotations_are_accepted_but_types_and_nullability_stay_strict() {
        let expected = parse_message_type(SCHEMA).unwrap();
        let arrow = SCHEMA
            .replace("UINT_64", "INTEGER(64,false)")
            .replace("UTF8", "STRING");
        let actual = parse_message_type(&arrow).unwrap();
        assert!(matches_schema(actual.get_fields(), expected.get_fields()));
        for bad in [
            arrow.replace("INTEGER(64,false)", "INTEGER(64,true)"),
            arrow.replace("REQUIRED INT64 ray_id", "OPTIONAL INT64 ray_id"),
            arrow.replace("REQUIRED INT64 ray_id", "REQUIRED INT64 row_index"),
            arrow.replace("DOUBLE position_x", "FLOAT position_x"),
            arrow.replace("OPTIONAL DOUBLE u_hit", "REQUIRED DOUBLE u_hit"),
        ] {
            let actual = parse_message_type(&bad).unwrap();
            assert!(!matches_schema(actual.get_fields(), expected.get_fields()));
        }
    }
}
