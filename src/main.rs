use clap::Parser;
use main_error::MainError;
use std::collections::HashMap;
use std::fmt::Display;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::Arc;
use std::{fs, str, thread};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ProgramError {
    #[error("Not the same amount of video and sub files.")]
    MismatchError,
    #[error("{0} language not supported.")]
    LangError(Arc<str>),
    #[error("User cancelled.")]
    ExitError,
    #[error("IO error: {0}")]
    InputError(#[from] std::io::Error),
}

type Stdout = Vec<u8>;
type ProgramResult<T> = Result<T, ProgramError>;

// mkvmerge -o [dir]/output/[videofile] [videofile] --language 0:jpn --track-name 0:Japanese [subfile]
fn mkvmerge<P: AsRef<Path>, S: AsRef<str> + Display>(
    output_dir: P,
    videofile_src: S,
    language: S,
    track_name: S,
    subfile_src: S,
) -> Command {
    let lang: Box<str> = format!("0:{}", language).into();
    let track: Box<str> = format!("0:{}", track_name).into();
    let mut videofile_out = output_dir.as_ref().to_path_buf();
    videofile_out.push(videofile_src.as_ref());
    let args = [
        "-o",
        &videofile_out.to_string_lossy(),
        videofile_src.as_ref(),
        "--language",
        lang.as_ref(),
        "--track-name",
        track.as_ref(),
        subfile_src.as_ref(),
    ];

    let mut cmd = Command::new("mkvmerge");
    cmd.args(args);
    cmd
}

fn ffs<S: AsRef<str> + Display>(video: S, sub_in: S, sub_out: S) -> Command {
    let args = [
        video.as_ref(),
        "-i",
        sub_in.as_ref(),
        "-o",
        sub_out.as_ref(),
    ];
    let mut cmd = Command::new("ffs");
    cmd.args(args);
    cmd
}

fn addsubs(params: Args) -> ProgramResult<Vec<ProgramResult<Stdout>>> {
    // ISO 639.2
    let langs = HashMap::from([
        ("jpn", "Japanese"),
        ("eng", "English"),
        ("spa", "Spanish"),
        ("und", "Undetermined"),
    ]);
    let language: Arc<str> = (*langs
        .get(params.lang.as_ref())
        .ok_or(ProgramError::LangError(params.lang.clone()))?)
    .into();

    // Create list of files.
    let mut videofiles = Vec::with_capacity(12);
    let mut subfiles = Vec::with_capacity(12);
    for file in fs::read_dir(params.dir.as_ref())? {
        let f: Arc<str> = file?.file_name().to_string_lossy().into();
        if f.contains(params.videoformat.as_ref()) {
            videofiles.push(f);
        } else if f.contains(params.subformat.as_ref()) {
            subfiles.push(f);
        }
    }
    videofiles.sort();
    subfiles.sort();
    if videofiles.len() != subfiles.len() {
        return Err(ProgramError::MismatchError);
    }

    // Check
    println!("Joining sub files to these video files.");
    let file_iter = subfiles.iter().zip(videofiles.iter());

    for (sub, vid) in file_iter.clone() {
        println!("{}\t{}", sub, vid);
    }

    // User confirmation
    println!("Are these pairs correct? (Y/n): ");
    let mut answer = String::new();
    std::io::stdin().read_line(&mut answer)?;
    if answer.contains("n") {
        return Err(ProgramError::ExitError);
    }

    // Create output folder
    let odf: PathBuf = format!("{}/output", params.dir).into();
    let output_dir: Arc<Path> = odf.into();
    fs::create_dir(output_dir.as_ref())?;

    // Run commands
    let mut threads = Vec::with_capacity(videofiles.len());

    for (s, v) in file_iter {
        let vc = v.clone();
        let mut cmd = mkvmerge(
            output_dir.clone(),
            v.clone(),
            params.lang.clone(),
            language.clone(),
            s.clone(),
        );

        let mut subsync_cmd = ffs(v.clone(), s.clone(), s.clone());

        threads.push(thread::spawn(move || -> ProgramResult<Stdout> {
            if params.sync {
                subsync_cmd.output()?;
            }
            let output = cmd.output()?;
            println!("Multiplexing {vc}");
            Ok(output.stdout)
        }));
    }
    Ok(threads.into_iter().map(|t| t.join().unwrap()).collect())
}

/// mkvmerge wrapper to bulk add subtitles to videofiles.
/// An output folder will be created with the multiplexed video files.
#[derive(Parser, Debug)]
#[command(author, version, about)]
struct Args {
    /// Directory with the video and sub files.
    #[arg(short, long, default_value = ".")]
    dir: Arc<str>,
    /// video file extension
    #[arg(short, long, default_value = "mkv")]
    videoformat: Box<str>,
    /// sub file extension
    #[arg(short, long, default_value = "srt")]
    subformat: Box<str>,
    /// ISO 639-2 language abbreviation
    #[arg(short, long, default_value = "jpn")]
    lang: Arc<str>,
    #[arg(short,long, action=clap::ArgAction::SetTrue)]
    sync: bool,
}

fn main() -> Result<(), MainError> {
    let args = Args::parse();

    for res in addsubs(args)? {
        let stdout = res?;
        let rs = String::from_utf8_lossy(&stdout);
        println!("{}", rs);
    }
    Ok(())
}
