use std::path::Path;
use std::error::Error;
use std::rc::Rc;
use std::{fs, fmt, str};
use std::collections::HashMap;
use std::process::Command;
use main_error::MainError;

#[derive(Debug)]
pub enum ProgramError {
    MismatchError,
    LangError(Box<str>),
    ExitError,
	NoArgumentError,
	InputError(std::io::Error),
}

static NOARGUMENT: &str = "The program requires 4 arguments to be provide:
\t 1. directory
\t 2. video file format (e.g. mkv)
\t 3. sub file format (e.g. srt)
\t 4. supported language code (e.g. jpn)";

impl Error for ProgramError {}
impl fmt::Display for ProgramError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ProgramError::MismatchError => write!(f, "Not the same amount of video and sub files."),
            ProgramError::LangError(lang) => write!(f, "{} language not supported.", *lang),
            ProgramError::ExitError => write!(f, "User cancelled."),
			ProgramError::InputError(e) => write!(f, "Input error: {}", e),
			ProgramError::NoArgumentError => write!(f,"{}", NOARGUMENT),
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
    //if videofiles.len() == subfiles.len() {return Err(ProgramError::MismatchError);}

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

fn main() -> Result<(), MainError> {
    let args: Rc<[String]> = std::env::args().collect();
	(args.len() == 5).then_some(0).ok_or(ProgramError::NoArgumentError)?;

    for res in addsubs(&args[1], &args[2], &args[3], &args[4])?.iter() {
		let rs = String::from_utf8_lossy(res);
		println!("{}", rs);
	}
	Ok(())
}
