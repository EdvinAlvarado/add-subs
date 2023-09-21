use std::path::Path;
use std::error::Error;
use std::{fs, fmt, str};
use std::collections::HashMap;
use std::process::Command;
use main_error::MainError;
use clap::Parser;

#[derive(Debug)]
pub enum ProgramError {
    MismatchError,
    LangError(Box<str>),
    ExitError,
	InputError(std::io::Error),
}


impl Error for ProgramError {}
impl fmt::Display for ProgramError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ProgramError::MismatchError => write!(f, "Not the same amount of video and sub files."),
            ProgramError::LangError(lang) => write!(f, "{} language not supported.", lang),
            ProgramError::ExitError => write!(f, "User cancelled."),
			ProgramError::InputError(e) => write!(f, "Input error: {}", e),
        }
    }
}
impl From<std::io::Error> for ProgramError {
	fn from(err: std::io::Error) -> ProgramError {
		ProgramError::InputError(err)
	}
}

type Stdout = Vec<u8>;

fn addsubs<P: AsRef<Path>>(dir: P, videoformat: &str, subformat: &str, lang: &str) -> Result<Vec<Stdout>, ProgramError> {

    // ISO 639.2
    let langs = HashMap::from([
                              ("jpn", "Japanese"),
                              ("eng", "English"),
                              ("spa", "Spanish"),
                              ("und", "Undetermined"),
    ]);

    // Create list of files.
    let mut videofiles = Vec::new();
    let mut subfiles = Vec::new();
    for file in fs::read_dir(dir.as_ref())? {
        let f: Box<str> = file?.file_name().to_string_lossy().into();
        if f.contains(videoformat) {videofiles.push(f);}
        else if f.contains(subformat) {subfiles.push(f);}
    }
    videofiles.sort();
    subfiles.sort();
    if videofiles.len() == subfiles.len() {return Err(ProgramError::MismatchError);}

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


    // Run commands
    let mut res = Vec::with_capacity(videofiles.len());
    Command::new("mkdir").arg("output").output()?;
    for (s,v) in file_iter {
        println!("{}", v);
		
		// mkvmerge -o [dir]/output/[videofile] [videofile] --language 0:jpn --track-name 0:Japanese [subfile]
        let args = [
        "-o",
        &format!("{}/output/{}",dir.as_ref().display(), v),
        &format!("{}", v),
        "--language",
        &format!("0:{}", lang),
        "--track-name",
        &format!("0:{}", langs.get(lang).ok_or(ProgramError::LangError(lang.into()))?),
        &format!("{}", s)
        ];
        
        let output = Command::new("mkvmerge").args(args).output()?;
        res.push(output.stdout);
    }
    Ok(res)
}

/// mkvmerge wrapper to bulk add subtitles to videofiles.
/// An output folder will be created with the multiplexed video files.
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
	/// Directory with the video and sub files.
	#[arg(short, long, default_value=".")]
	dir: Box<str>,
	/// video file extension 
	#[arg(short, long, default_value="mkv")]
	videoformat: Box<str>,
	/// sub file extension
	#[arg(short, long, default_value="srt")]
	subformat: Box<str>,
	/// ISO 639-2 language abbreviation
	#[arg(short, long, default_value="jpn")]
	lang: Box<str>,
}


fn main() -> Result<(), MainError> {
    let args = Args::parse();

    for res in addsubs(args.dir.as_ref(), &args.videoformat, &args.subformat, &args.lang)? {
		let rs = String::from_utf8_lossy(&res);
		println!("{}", rs);
	}
	Ok(())
}
