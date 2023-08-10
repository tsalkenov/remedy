#![allow(unused)]
use std::ffi::OsStr;
use std::io::{self, Write};
use std::iter::repeat;
use std::path::PathBuf;
use std::sync::Once;
use std::{env, fs};

use colored::Colorize;
use crossterm::event::{poll, read, Event};
use crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, size, Clear, ClearType, EnterAlternateScreen, LeaveAlternateScreen,
};
use crossterm::ExecutableCommand;
use image::codecs::farbfeld::FarbfeldDecoder;
use image::codecs::gif::GifDecoder;
use image::imageops::{resize, FilterType};
use image::{AnimationDecoder, Rgba};

static ONCE: Once = Once::new();

fn main() -> anyhow::Result<()> {
    pretty_env_logger::formatted_builder()
        .filter(None, log::LevelFilter::Debug)
        .init();

    let target_path = recieve_path()?;
    let target_file = fs::File::open(target_path)?;

    let decoder = GifDecoder::new(target_file)?;
    let frames = decoder.into_frames();
    let frames = frames.collect_frames()?;
    let delay = frames[0].delay();

    let (term_width, term_height) = size()?;

    let fitted_frames: Vec<String> = frames
        .into_iter()
        .map(|frame| {
            let buffer = frame.buffer();

            let multiplier: f32 = (term_width as f32 / buffer.width() as f32).min(term_height as f32 / buffer.height() as f32);
            let new_height = (buffer.height() as f32 * multiplier).ceil() as u32;
            let new_width = (buffer.width() as f32 * multiplier).ceil() as u32;

            ONCE.call_once(|| {
                log::info!("multiplier: {multiplier}");
                log::info!("term: {term_width} {term_height}");
                log::info!("news: {new_width} {new_height}");
            });

            resize(buffer, new_height, new_width, FilterType::Lanczos3)
                .rows()
                .map(|row| {
                    row.into_iter()
                        .map(|Rgba([r, g, b, _])| "0".truecolor(*r, *g, *b).to_string())
                        .collect::<String>()
                        + "\n\r"
                })
                .collect::<String>()
        })
        .collect();

    let mut stdout = io::stdout();

    enable_raw_mode()?;
    stdout.execute(EnterAlternateScreen)?;

    for frame in repeat(fitted_frames).flat_map(|x| x.into_iter()) {
        stdout.execute(Clear(ClearType::All))?;
        write!(stdout, "{}", frame)?;
        if poll(delay.into())? {
            if let Event::Key(k) = read()? {
                if let crossterm::event::KeyCode::Char('q') = k.code {
                    break;
                }
            }
        }
    }

    disable_raw_mode()?;
    stdout.execute(LeaveAlternateScreen)?;

    Ok(())
}

fn recieve_path() -> io::Result<PathBuf> {
    if env::args().len() <= 1 {
        log::error!("Bruh no path to file");
        std::process::exit(1);
    }
    let target = env::args().last().unwrap();

    let target_file = env::current_dir()?.join(target);

    if !target_file.exists() {
        log::error!("Bruh path file no be");
        std::process::exit(1);
    }
    let extension = target_file.extension().and_then(OsStr::to_str).unwrap_or("");

    if extension == "gif" {
        log::error!("bruh file no good way encode");
        std::process::exit(1);
    }

    Ok(target_file)
}
