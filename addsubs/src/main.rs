use std::path::Path;
use std::error::Error;
use std::fs;
use std::fmt;
use std::collections::HashMap;
use std::process::Command;


#[derive(Debug)]
pub enum ProgramError {
    MismatchError,
    LangError,
    ExitError,
}

impl Error for ProgramError {}
impl fmt::Display for ProgramError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ProgramError::MismatchError => write!(f, "Not the same amount of video and sub files."),
            ProgramError::LangError => write!(f, "Language not supported."),
            ProgramError::ExitError => write!(f, "User cancelled."),
        }
    }
}


fn addsubs(dir: &Path, videoformat: &str, subformat: &str, lang: &str) -> Result<(), Box<dyn Error>> {

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
    if videofiles.len() == subfiles.len() {return Err(Box::new(ProgramError::MismatchError));}

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
    Command::new("mkdir").arg("output").output().expect("Could not create output folder.");
    for (s,v) in file_iter {
        Command::new("mkvmerge").args([
            "-o",
            &format!("'output/{}'", v),
            &format!("'{}'", v),
            "--language",
            &format!("0:{}", lang),
            "--track-name",
            &format!("0:{}", langs.get(lang).unwrap()),
            &format!("'{}'", s)
        ]).output().expect("mkvmerge command failed to run. Is mkvmerge installed?");
    }
    
    Ok(())
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    addsubs(Path::new(&args[1]), &args[2], &args[3], &args[4]).unwrap()
}
