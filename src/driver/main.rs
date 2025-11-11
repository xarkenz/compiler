use std::time::Instant;

fn main() {
    let start_time = Instant::now();

    let args = cupric::cli::parse_command_line_args();
    match cupric::cli::invoke(&args) {
        Err(error) => {
            let (error, source_paths) = *error;
            println!("\x1b[31m{}\x1b[0m", error.to_string_with_context(&source_paths));
        }
        Ok(..) => {
            println!("\x1b[32mFinished\x1b[0m");
        }
    }

    let time_taken_ms = start_time.elapsed().as_millis();
    println!("\x1b[2mTime: {time_taken_ms} ms\x1b[22m");
}
