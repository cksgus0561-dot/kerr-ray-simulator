"""Read actual archives with PyArrow; old reference files remain untouched."""
import collections
import hashlib
import json
from pathlib import Path
import shutil
import struct
import subprocess
import sys
import tempfile

root = Path(__file__).resolve().parents[1]
sys.path.insert(0, str(root / "work/python-parquet/site-packages"))
import pyarrow as pa
import pyarrow.parquet as pq

columns = ["ray_id", "position_x", "position_y", "position_z",
           "direction_x", "direction_y", "direction_z", "energy", "t_emit",
           "status", "u_hit", "v_hit", "t_hit"]
physical = columns[1:7] + ["t_emit"]
hit_names = columns[-3:]
reports = []
for n, count, expected in [(1, 256, {"Detected": 216, "Captured": 20, "Escaped": 20}),
                            (2, 4096, {"Detected": 3502, "Captured": 330, "Escaped": 264})]:
    folder = f"simulation_{n}"
    old_path = root / "results/standard_validation" / folder / "sim_1.parquet"
    new_path = root / "results/ray_id_validation" / folder / "sim_1.parquet"
    old = pq.read_table(old_path)
    table = pq.read_table(new_path)
    assert table.column_names == columns
    assert table.num_rows == count
    assert table.schema.field("ray_id").type == pa.uint64()
    assert table.column("ray_id").null_count == 0
    assert all(table.schema.field(c).nullable == (c in hit_names) for c in columns)
    assert all(table.schema.field(c).type == pa.float64() for c in columns
               if c not in ["ray_id", "status"])
    assert table.schema.field("status").type == pa.string()
    assert table.schema.metadata == old.schema.metadata
    rows = table.to_pylist()
    ids = [r["ray_id"] for r in rows]
    assert sorted(ids) == list(range(count))
    # Grid fixture direction = (-1,0,0), hence normalization is exact here.
    canonical = sorted(rows, key=lambda r: tuple(r[c] for c in physical))
    assert [r["ray_id"] for r in canonical] == list(range(count))
    assert ids != list(range(count))  # Storage row number is demonstrably not ID.
    assert dict(collections.Counter(r["status"] for r in rows)) == expected
    old_by_input = {tuple(r[c] for c in physical): r for r in old.to_pylist()}
    for row in rows:
        reference = old_by_input[tuple(row[c] for c in physical)]
        for col in columns[1:]:
            actual, original = row[col], reference[col]
            if isinstance(actual, float):
                assert struct.pack("<d", actual) == struct.pack("<d", original), (n, row["ray_id"], col)
            else:
                assert actual == original
        assert (row["status"] == "Detected") == all(row[c] is not None for c in hit_names)
        if row["status"] != "Detected":
            assert all(row[c] is None for c in hit_names)
    metadata = pq.ParquetFile(new_path).metadata
    assert all(metadata.row_group(g).column(c).compression == "SNAPPY"
               for g in range(metadata.num_row_groups) for c in range(metadata.num_columns))
    old_bytes, new_bytes = old_path.stat().st_size, new_path.stat().st_size
    reports.append({"rays": count, "schema": str(table.schema), "dtype": "uint64",
                    "unique_ids": len(set(ids)), "id_min": min(ids), "id_max": max(ids),
                    "id_nulls": table.column("ray_id").null_count,
                    "counts": expected, "old_column_bit_mismatches": 0,
                    "canonical_id_mismatches": 0, "non_detected_ids": count - expected["Detected"],
                    "hit_nulls_per_column": [table.column(c).null_count for c in hit_names],
                    "old_bytes": old_bytes, "new_bytes": new_bytes,
                    "increase_bytes": new_bytes - old_bytes,
                    "increase_percent": 100 * (new_bytes - old_bytes) / old_bytes,
                    "old_sha256": hashlib.sha256(old_path.read_bytes()).hexdigest(),
                    "new_sha256": hashlib.sha256(new_path.read_bytes()).hexdigest()})

# Independent PyArrow rewrite exercises the real Rust reader and CLI, not a mock.
base = root / "results/ray_id_validation/simulation_1"
table = pq.read_table(base / "sim_1.parquet")
cli_checks = []
for case in ["shuffled", "swapped_ids", "duplicate_ids"]:
    directory = Path(tempfile.mkdtemp(prefix=f"ray_id_pyarrow_{case}_", dir=root / "work"))
    shutil.copyfile(base / "common.json", directory / "common.json")
    altered = table.take(pa.array(list(reversed(range(table.num_rows)))))
    if case != "shuffled":
        ids = altered.column("ray_id").to_pylist()
        if case == "swapped_ids":
            ids[0], ids[1] = ids[1], ids[0]
        else:
            ids[0] = ids[1]
        altered = altered.set_column(0, table.schema.field("ray_id"), pa.array(ids, type=pa.uint64()))
    pq.write_table(altered, directory / "sim_1.parquet", compression="snappy")
    command = "inspect-simulation" if case == "duplicate_ids" else "reproduce"
    run = subprocess.run([str(root / "target/release/kerr-ray.exe"), command,
                          "--input", str(directory)], capture_output=True, text=True,
                         creationflags=subprocess.CREATE_NO_WINDOW)
    (root / "validation/ray_ids" / f"pyarrow_{case}.txt").write_text(
        run.stdout + run.stderr, encoding="utf-8")
    if case == "shuffled":
        assert run.returncode == 0, run.stdout + run.stderr
        assert "stored ray_id checked: 256 / 256" in run.stdout
        assert "stored vs regenerated ray_id mismatch: 0" in run.stdout
    elif case == "swapped_ids":
        assert run.returncode != 0
        assert "stored vs regenerated ray_id mismatch: 2" in run.stdout
        assert "hit bit mismatches [u,v,t]: [0, 0, 0]" in run.stdout
        assert "status mismatch: 0" in run.stdout
        assert "regenerated ray_id mismatch: 0" in run.stdout
    else:
        assert run.returncode != 0
        assert "duplicate ray_id" in run.stderr
    cli_checks.append({"case": case, "directory": str(directory.relative_to(root)),
                       "exit_code": run.returncode, "pass": True})

result = {"pyarrow": pa.__version__, "reports": reports, "cli_checks": cli_checks}
destination = root / "validation/ray_ids/python_compatibility.json"
destination.write_text(json.dumps(result, indent=2), encoding="utf-8")
print(json.dumps(result, indent=2))
