use anyhow::Result;
use std::io::{stdout, BufRead as _, BufReader, BufWriter, Read, Write as _};

static LINE_RANGE: usize = 5;

pub fn print_source_code(r: impl Read, current_line: usize) -> Result<()> {
    let mut start_line = 1;
    if current_line < LINE_RANGE {
        start_line = current_line - LINE_RANGE;
    }
    let end_line = current_line + LINE_RANGE;

    let mut bw = BufWriter::new(stdout());

    let reader = BufReader::new(r);
    for (mut read_line, line) in reader.lines().enumerate() {
        read_line += 1;
        if read_line < start_line {
            continue;
        }
        if read_line > end_line {
            break;
        }

        let line = line?;
        let bytes = if read_line == current_line {
            format!("-> {:1}: {}\n", read_line, line)
        } else {
            format!("   {:1}: {}\n", read_line, line)
        };

        bw.write(bytes.as_bytes())?;
    }

    bw.flush()?;

    Ok(())
}
