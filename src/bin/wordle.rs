use clap::{Subcommand, Parser};
use rand::seq::SliceRandom;

use std::io::{self, BufRead, BufReader, Write};
use std::fs::File;
use std::path::Path;
use std::process::{self, Command, Stdio, Child, ChildStdin, ChildStdout};

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
enum Score {
    Here,
    Somewhere,
    Nowhere,
}

// Precondition: solution and guess each contain exactly five ASCII letters.
fn score(solution: &str, guess: &str) -> [Score; 5] {
    use Score::*;

    assert_eq!(solution.bytes().len(), 5);
    assert_eq!(guess.bytes().len(), 5);

    let mut scores = [Nowhere; 5];

    for (i, wl) in solution.bytes().enumerate() {
        if wl == guess.bytes().nth(i).unwrap() {
            scores[i] = Here;
        } else {
            for (j, gl) in guess.bytes().enumerate() {
                if scores[j] != Nowhere {
                    continue;
                }

                if i == j {
                    continue;
                }

                if wl == gl {
                    if i == j { 
                        scores[j] = Here;
                    } else { 
                        scores[j] = Somewhere;
                    }
                    break;
                }
            }
        }
    }

    scores
}

#[cfg(test)]
mod tests {
    use super::score;
    use super::Score;

    #[test]
    fn test_score() {
        use Score::*;

        assert_eq!(score("quata", "quass"), [Here, Here, Here, Nowhere, Nowhere]);
        assert_eq!(score("moroc", "mormo"), [Here, Here, Here, Nowhere, Somewhere]);
    }
}

trait GameIO {
    fn read_guess(&mut self) -> io::Result<String>;
    fn write_scores(&mut self, scores: &[Score; 5]) -> io::Result<()>;
}

struct TerminalIO;

impl GameIO for TerminalIO {
    fn read_guess(&mut self) -> io::Result<String> {
        let mut guess = String::new();
        let stdin = io::stdin();
        let mut handle = stdin.lock();
        handle.read_line(&mut guess)?;
        let len = guess.trim_end().len();
        guess.truncate(len);
        Ok(guess)
    }

    fn write_scores(&mut self, scores: &[Score; 5]) -> io::Result<()> {
        use Score::*;

        let stdout = io::stdout();
        let mut handle = stdout.lock();

        for score in scores {
            match score {
                Nowhere => write!(handle, " ")?,
                Somewhere => write!(handle, "{}", console::style("X").yellow())?,
                Here => write!(handle, "{}", console::style("O").green())?,
            }
        }

        write!(handle, "\n")?;
        handle.flush()?;

        Ok(())
    }
}

struct ProgramIO {
    child: Child,
    stdin: ChildStdin,
    stdout: BufReader<ChildStdout>,
}

impl GameIO for ProgramIO {
    fn read_guess(&mut self) -> io::Result<String> {
        let mut guess = String::new();
        self.stdout.read_line(&mut guess)?;
        let len = guess.trim_end().len();
        guess.truncate(len);
        Ok(guess)
    }

    fn write_scores(&mut self, scores: &[Score; 5]) -> io::Result<()> {
        use Score::*;

        for score in scores {
            match score {
                Nowhere => write!(self.stdin, " ")?,
                Somewhere => write!(self.stdin, "{}", console::style("X").yellow())?,
                Here => write!(self.stdin, "{}", console::style("O").green())?,
            }
        }

        write!(self.stdin, "\n")?;
        self.stdin.flush()?;

        Ok(())
    }
}

impl Drop for ProgramIO {
    fn drop(&mut self) {
        self.child.kill().unwrap();
        self.child.wait().unwrap();
    }
}

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
    wordlist: Option<String>,

    /// Runs game with given solution
    #[clap(short, long)]
    solution: Option<String>,

    /// Player program; default is to play on the terminal.
    #[clap(subcommand)]
    program_args: Option<Program>
}

#[derive(Subcommand, Debug)]
enum Program {
    #[clap(external_subcommand)]
    Program(Vec<String>)
}

const DEFAULT_WORDLIST_PATH: &str = "word-list-5.txt";

fn load_wordlist<P: AsRef<Path>>(wordlist_path: P) -> io::Result<Vec<String>> {
    let file = File::open(wordlist_path)?;
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

fn main() {
    let args = Args::parse();

    // load wordlist
    let wordlist_path = args.wordlist.as_deref().unwrap_or(DEFAULT_WORDLIST_PATH);

    if args.verbose {
        eprintln!("Loading wordlist from {}", wordlist_path);
    }

    let wordlist = load_wordlist(wordlist_path).unwrap();

    // Select solution
    let solution = if let Some(solution) = args.solution {
        if wordlist.iter().any(|w| *w == solution) {
            solution
        } else {
            panic!("Invalid solution");
        }
    } else {
        let mut rng = &mut rand::thread_rng();
        wordlist.choose(&mut rng).cloned().unwrap()
    };

    if args.verbose {
        eprintln!("Solution: {}", solution);
    }

    let mut io: Box<dyn GameIO> = if let Some(Program::Program(program_args)) = args.program_args {
        let (p_name, p_args) = program_args.split_first().unwrap();

        let mut child = Command::new(p_name);

        if args.hard {
            child.arg("-H");
        }

        if args.verbose {
            child.arg("-v");
        }

        let mut child = child
            .args(["-w", wordlist_path])
            .args(p_args)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(if args.verbose { Stdio::inherit() } else { Stdio::null() })
            .spawn()
            .expect("Could not initialize program");

        let stdin = child.stdin.take().unwrap();
        let stdout = child.stdout.take().unwrap();

        Box::new(ProgramIO {
            child,
            stdin,
            stdout: BufReader::new(stdout)
        })
    } else {
        Box::new(TerminalIO)
    };

    let mut turn = 1;

    while let Ok(guess) = io.read_guess() {
        if args.verbose {
            eprintln!("Guess: \"{}\"", guess);
        }

        if wordlist.iter().find(|w| **w == guess).is_none() {
            eprintln!("Invalid guess: {}", guess);
            process::exit(1);
        }

        let scores = score(&solution, &guess);

        io.write_scores(&scores).unwrap();

        if scores.iter().all(|s| *s == Score::Here) {
            println!("{}", turn);
            return;
        }

        turn += 1;
    }
}
