// #![allow(unused)]

use std::ffi::OsStr;
use std::fs;
use std::io::{self, Write};
use std::iter::repeat;
use std::path::PathBuf;
use std::sync::Once;
use std::time::Duration;

use clap::Parser;
use colored::Colorize;
use crossterm::event::{poll, read, Event};
use crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, size, Clear, ClearType, EnterAlternateScreen, LeaveAlternateScreen,
};
use crossterm::ExecutableCommand;
use image::codecs::gif::GifDecoder;
use image::imageops::{resize, FilterType};
use image::{AnimationDecoder, Rgba};

#[derive(Parser)]
struct Cli {
    /// Name of gif file to play
    target_file: PathBuf,
    /// Character representing one cell of the image
    #[arg(short, long, default_value_t = '0')]
    char: char,
    /// Toggle debug information logging
    #[arg(short, long)]
    debug: bool,
}

static ONCE: Once = Once::new();

fn main() -> anyhow::Result<()> {
    pretty_env_logger::formatted_builder()
        .filter(None, log::LevelFilter::Debug)
        .init();
    let cli = Cli::parse();

    let target_path = cli.target_file;
    if !target_path.exists() {
        log::error!("File doesn't exist");
        std::process::exit(1);
    }
    let extension = target_path.extension().and_then(OsStr::to_str).unwrap_or("");
    if extension != "gif" {
        log::error!("File has an invalid extension. Provide a gif file");
        std::process::exit(1);
    }

    let target_file = fs::File::open(target_path)?;

    let frames = load_frames(target_file)?;
    let delay = frames[0].delay();

    let fitted_frames = fit_frames(cli.char, &frames, cli.debug)?;

    let mut stdout = io::stdout();

    enable_raw_mode()?;
    stdout.execute(EnterAlternateScreen)?;
    play_animation(&mut stdout, &fitted_frames, delay.into())?;
    disable_raw_mode()?;
    stdout.execute(LeaveAlternateScreen)?;

    Ok(())
}

fn load_frames<T: std::io::Read>(input: T) -> anyhow::Result<Vec<image::Frame>> {
    let decoder = GifDecoder::new(input)?;
    let frames = decoder.into_frames().collect_frames()?;
    Ok(frames)
}

fn fit_frames(char: char, frames: &[image::Frame], debug: bool) -> anyhow::Result<Vec<String>> {
    let (term_width, term_height) = size()?;
    Ok(frames
        .iter()
        .map(|frame| {
            let buffer = frame.buffer();

            let multiplier: f32 = (term_width as f32 / buffer.width() as f32).min(term_height as f32 / buffer.height() as f32);
            let new_height = (buffer.height() as f32 * multiplier).ceil() as u32;
            let new_width = (buffer.width() as f32 * multiplier).ceil() as u32;
            let padding = (term_width as u32 - new_width) / 2;

            if debug {
                ONCE.call_once(|| {
                    log::info!("---SIZES---");
                    log::info!("term: {}x{}", term_width, term_height);
                    log::info!("img: {}x{}", buffer.width(), buffer.height());
                    log::info!("output: {}x{}", new_width, new_height);
                });
            }

            resize(buffer, new_width, new_height, FilterType::Lanczos3)
                .rows()
                .map(|row| {
                    " ".repeat(padding as usize)
                        + &row
                            .into_iter()
                            .map(|Rgba([r, g, b, _])| char.to_string().truecolor(*r, *g, *b).to_string())
                            .collect::<String>()
                        + "\n\r"
                })
                .collect::<String>()
        })
        .collect())
}

fn play_animation(stdout: &mut io::Stdout, frames: &[String], delay: Duration) -> io::Result<()> {
    for frame in repeat(frames).flat_map(|x| x.iter()) {
        stdout.execute(Clear(ClearType::All))?;
        write!(stdout, "{}", frame)?;
        if poll(delay)? {
            if let Event::Key(k) = read()? {
                if let crossterm::event::KeyCode::Char('q') = k.code {
                    break;
                }
            }
        }
    }
    Ok(())
}
