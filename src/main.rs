use std::error::Error;
use std::{fs, fmt, str, thread};
use std::collections::HashMap;
use std::process::Command;
use main_error::MainError;
use clap::Parser;
use std::sync::Arc;

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
	

    // Run commands
    Command::new("mkdir").arg("output").output()?;
	let mut threads = Vec::with_capacity(videofiles.len());
	let cmd_lang: Arc<str> = format!("0:{}", params.lang).into();
	let cmd_long_lang: Arc<str> = format!("0:{}", langs.get(params.lang.as_ref()).ok_or(ProgramError::LangError(params.lang.into()))?).into();
    for (s,v) in file_iter {

		// mkvmerge -o [dir]/output/[videofile] [videofile] --language 0:jpn --track-name 0:Japanese [subfile]
		let args: [Arc<str>; 8] = [
		"-o".into(),
		format!("{}/output/{}",params.dir, v).as_str().into(),
		v.clone(),
		"--language".into(),
		cmd_lang.clone(),
		"--track-name".into(),
		cmd_long_lang.clone(),
		s.clone()
		];
	
		let vc = v.clone();
		threads.push(thread::spawn(move || -> ProgramResult<Stdout> {
			let arg_iter =  args.iter().map(|s| s.as_ref());
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
	#[arg(short, long, default_value=".")]
	dir: Arc<str>,
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

    for res in addsubs(args)? {
		let stdout = res?;
		let rs = String::from_utf8_lossy(&stdout);
		println!("{}", rs);
	}
	Ok(())
}
