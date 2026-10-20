#[path = "../src/render_lab/experimental.rs"]
mod experimental;

fn main() {
    match experimental::shader_present_probe() {
        Ok(output) => {
            println!("CPU_READBACK_PRESENTATION_READY");
            println!("bytes_copied={}", output.bytes_copied);
            println!("encode_submit_micros={}", output.encode_submit_micros);
            println!(
                "completion_readback_micros={}",
                output.completion_readback_micros
            );
            println!("total_micros={}", output.total_micros);
            for sample in output.samples {
                println!("sample=({}, {}) rgba={:?}", sample.x, sample.y, sample.rgba);
            }
            println!(
                "image_bytes={}",
                output.image.as_bytes(0).map_or(0, <[u8]>::len)
            );
        }
        Err(error) => {
            eprintln!("READBACK_BLOCKED: {error}");
            std::process::exit(1);
        }
    }
}
