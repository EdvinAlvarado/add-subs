use clap::Parser;
use main_error::MainError;
use std::collections::HashMap;
use std::process::Command;
use std::sync::Arc;
use std::{fs, str, thread};
use thiserror::Error;


#[derive(Debug, Error)]
pub enum ProgramError {
	#[error("Not the same amount of video and sub files.")]
    MismatchError,
	#[error("{0} language not supported.")]
    LangError(Box<str>),
	#[error("User cancelled.")]
    ExitError,
	#[error("IO error: {0}")]
	InputError(#[from] std::io::Error),
}


type Stdout = Vec<u8>;
type ProgramResult<T> = Result<T, ProgramError>;


fn addsubs(params: Args) -> ProgramResult<Vec<ProgramResult<Stdout>>> {

    // ISO 639.2
    let langs = HashMap::from([
                              ("jpn", "Japanese"),
                              ("eng", "English"),
                              ("spa", "Spanish"),
                              ("und", "Undetermined"),
    ]);

    // Create list of files.
    let mut videofiles = Vec::with_capacity(12);
    let mut subfiles = Vec::with_capacity(12);
    for file in fs::read_dir(params.dir.as_ref())? {
        let f: Arc<str> = file?.file_name().to_string_lossy().into();
        if f.contains(params.videoformat.as_ref()) {videofiles.push(f);}
        else if f.contains(params.subformat.as_ref()) {subfiles.push(f);}
    }
    videofiles.sort();
    subfiles.sort();
    if videofiles.len() != subfiles.len() {return Err(ProgramError::MismatchError);}

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
    if answer.contains("n") {return Err(ProgramError::ExitError);}
	

	// Create output folder
	let output_dir: Arc<str> = format!("{}/output", params.dir).into();
	fs::create_dir(output_dir.as_ref())?;
    
	// Run commands
    let mut threads = Vec::with_capacity(videofiles.len());
    let cmd_lang: Arc<str> = format!("0:{}", params.lang).into();
    let cmd_long_lang: Arc<str> = format!(
        "0:{}",
        langs
            .get(params.lang.as_ref())
            .ok_or(ProgramError::LangError(params.lang.into()))?
    )
    .into();
    for (s, v) in file_iter {
        // mkvmerge -o [dir]/output/[videofile] [videofile] --language 0:jpn --track-name 0:Japanese [subfile]
        let args: [Arc<str>; 8] = [
            "-o".into(),
            format!("{}/{}", output_dir, v).into(),
            v.clone(),
            "--language".into(),
            cmd_lang.clone(),
            "--track-name".into(),
            cmd_long_lang.clone(),
            s.clone(),
        ];

        let vc = v.clone();
        threads.push(thread::spawn(move || -> ProgramResult<Stdout> {
            let arg_iter = args.iter().map(|s| s.as_ref());
            let output = Command::new("mkvmerge").args(arg_iter).output()?;
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
    lang: Box<str>,
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
