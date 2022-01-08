# Wordle Runner

The `wordle` binary runs a game of wordle against either a human player via the terminal, or a program written to the specification detailed below.

After each run, the binary will print the number of guesses to stdout, and then exit.

## Invocation

The subordinate program is invoked with the following flags:

- -w [WORDLIST]
Path to a file where the wordlist may be found. Words are written one per line, all lower-case.

- -v
If this flag is provided, the program's stderr will be copied to wordle's stderr, for debugging.

- -H
If this flag is provided, the program must use information gleaned in previous clues on each subsequent guess. (TODO: Implement this!)

The program must at this point begin writing its guesses to stdout, ending each guess with a newline. Each guess must be a word from the
wordlist, and must be lower-case; writing anything else to stdout will cause the program to end. 

After each guess, the program must read its score from stdin. Each score will consist of five characters, followed by a newline. Each
character gives a hint about the corresponding letter of the guess:

- 'O' indicates that the letter is in the correct place.
- 'X' indicates that the letter is in the word, but not in the correct place.
- ' ' (u+0020 SPACE) indicates that the letter is not in the word.

The program must continue running until it receives a score of "OOOOO", after which it must exit.
