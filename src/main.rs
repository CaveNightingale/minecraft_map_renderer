use std::{path::PathBuf, process::ExitCode};

use clap::Parser;

#[derive(Parser)]
#[command(
    name = "minecraft-map-renderer",
    version,
    about = "Render Minecraft 26.1 region files into map-color tiles"
)]
struct Args {
    #[arg(long, value_name = "REGION_DIRECTORY")]
    input: PathBuf,
    #[arg(long, value_name = "NEW_DIRECTORY")]
    output: PathBuf,
    #[arg(long, value_name = "JSON")]
    blockstate_registry: PathBuf,
    #[arg(long, default_value = "2048", value_parser = minecraft_map_renderer::parse_tile_size)]
    tile_size: usize,
}

fn main() -> ExitCode {
    let args = Args::parse();
    match minecraft_map_renderer::render(
        &args.input,
        &args.output,
        &args.blockstate_registry,
        args.tile_size,
    ) {
        Ok(stats) => {
            eprintln!(
                "Rendered {} full chunks into {} tiles; skipped {} partial chunks",
                stats.full_chunks, stats.tiles, stats.partial_chunks
            );
            ExitCode::SUCCESS
        }
        Err(error) => {
            eprintln!("error: {error:#}");
            ExitCode::FAILURE
        }
    }
}
