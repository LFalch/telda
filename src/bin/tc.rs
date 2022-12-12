use std::{fs::File, env::args, path::Path, io::Write, process::ExitCode};

use telda2::source::{SourceLines, process, DataLine, write_data_operand};

fn main() -> ExitCode {
    let mut ret = ExitCode::SUCCESS;
    for arg in args().skip(1) {
        let p = Path::new(&arg);
        let (labels, data_lines)
        = match SourceLines::new(p).and_then(|l| process(l)) {
            Ok(s) => s,
            Err(e) => {
                eprintln!("{}", e);
                ret = ExitCode::FAILURE;
                continue;
            }
        };
        let mut mem = Vec::with_capacity(256);
        for data_line in data_lines {
            match data_line {
                DataLine::Raw(mut bytes) => {
                    mem.append(&mut bytes);
                }
                DataLine::Ins(opcode, dat_op) => {
                    mem.push(opcode);

                    write_data_operand(&mut mem, |id| labels[id].2, dat_op).unwrap();
                }
            }
        }

        let bin_path = p.with_extension("tbin");
        let mut f = File::create(&bin_path).unwrap();
        f.write_all(&mem).unwrap();
        println!("Wrote binary to {}", bin_path.display());

        let sym_path = p.with_extension("tsym");
        let mut f = File::create(p.with_extension("tsym")).unwrap();
        for (lbl, global, loc) in labels.iter() {
            if !global {
                write!(f, "private $").unwrap();
            }
            writeln!(f, "{lbl}: 0x{loc:02X}").unwrap();
        }
        println!("Wrote symbols to {}", sym_path.display());

        if labels.iter().all(|(s, _, _)| &**s != "_start") {
            eprintln!("Warning: no _start symbol");
        }
    }
    ret
}
