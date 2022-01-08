use clap::Parser;

use std::fs::File;
use std::io::{self, BufRead, BufReader};

#[derive(Parser, Debug)]
struct Args {
    /// Turns on verbosity
    #[clap(short, long)]
    verbose: bool,

    /// Turns on hard mode
    #[clap(short('H'), long)]
    hard: bool,

    /// Specifies wordlist file
    #[clap(short, long)]
    wordlist: String,
}

impl Args {
  fn load_wordlist(&self) -> io::Result<Vec<String>> {
      let file = File::open(&self.wordlist)?;
      let reader = BufReader::new(file);
      let mut lines = reader.lines();
      let mut wordlist = Vec::new();

      while let Some(Ok(line)) = lines.next() {
          if line.len() != 5 {
              panic!("Invalid word length");
          }

          if line.chars().any(|c| !c.is_ascii_lowercase()) {
              panic!("Invalid word");
          }

          wordlist.push(line);
      }

      Ok(wordlist)
    }
}

fn main() {
    let args = Args::parse();

    if args.verbose {
      eprintln!("Loading wordlist...");
    }

    let wordlist = args.load_wordlist().expect("Could not load wordlist");

    for word in wordlist {
      println!("{}", word);
      io::stdin().read_line(&mut String::new()).unwrap();
    }
}
