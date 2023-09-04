use std::path::Path;
use std::error::Error;
use std::{fs, fmt, str};
use std::collections::HashMap;
use std::process::Command;


#[derive(Debug)]
pub enum ProgramError {
    MismatchError,
    LangError,
    ExitError,
	NoArgumentError
}

impl Error for ProgramError {}
impl fmt::Display for ProgramError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ProgramError::MismatchError => write!(f, "Not the same amount of video and sub files."),
            ProgramError::LangError => write!(f, "Language not supported."),
            ProgramError::ExitError => write!(f, "User cancelled."),
			ProgramError::NoArgumentError => write!(f, "Not enough arguments provided.")
        }
    }
}


fn addsubs(dir: &Path, videoformat: &str, subformat: &str, lang: &str) -> Result<Vec<Vec<u8>>, Box<dyn Error>> {

    // ISO 639.2
    let langs = HashMap::from([
                              ("jpn", "Japanese"),
                              ("eng", "English"),
                              ("spa", "Spanish"),
                              ("und", "Undetermined"),
    ]);
    if !langs.contains_key(lang) {return Err(Box::new(ProgramError::LangError));}

    // Create list of files.
    let mut videofiles = Vec::new();
    let mut subfiles = Vec::new();
    for file in fs::read_dir(dir)? {
        let f = file.unwrap().file_name().into_string().unwrap();
        if f.contains(videoformat) {videofiles.push(f);}
        else if f.contains(subformat) {subfiles.push(f);}
    }
    videofiles.sort();
    subfiles.sort();
    //if videofiles.len() == subfiles.len() {return Err(Box::new(ProgramError::MismatchError));}

    // Check
    println!("Joining sub files to these video files.");
    let file_iter = subfiles.iter().zip(videofiles.iter()); 
    for (sub, vid) in file_iter.clone() {
        println!("{}\t{}", sub, vid);
    } 

    println!("Are these pairs correct? (Y/n): ");
    let mut answer = String::new();
    std::io::stdin().read_line(&mut answer)?;
    if answer.contains("n") {return Err(Box::new(ProgramError::ExitError));}


    // Run commands
    let mut res = Vec::new();
    Command::new("mkdir").arg("output").output().expect("Could not create output folder.");
    for (s,v) in file_iter {
        println!("{}", v);

        let args = [
        "-o",
        &format!("{}/output/{}",dir.display(), v),
        &format!("{}", v),
        "--language",
        &format!("0:{}", lang),
        "--track-name",
        &format!("0:{}", langs.get(lang).unwrap()),
        &format!("{}", s)
        ];
        
        let output = Command::new("mkvmerge").args(args).output().expect("mkvmerge command failed to run. Is mkvmerge installed?");
        res.push(output.stdout);
    }
 
    Ok(res)
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
	if args.len() < 5 {
		println!("The program requires 4 arguments to be provided:");
		println!("\t1. directory");
		println!("\t2. video file format (e.g. mkv)");
		println!("\t3. sub file format (e.g. srt)");
		println!("\t4. supported language code (e.g. jpn)");
		panic!("{}", ProgramError::NoArgumentError.to_string());
	}

    let res = addsubs(Path::new(&args[1]), &args[2], &args[3], &args[4]).unwrap();
    for buf in res {
        let rs = match str::from_utf8(&buf) {
            Ok(v) => v,
            Err(e) => panic!("Invalid UTF-8 sequence: {}", e),
        };
        println!("{}", rs);
    }
}
