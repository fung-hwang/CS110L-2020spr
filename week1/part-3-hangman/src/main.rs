// Simple Hangman Program
// User gets five incorrect guesses
// Word chosen randomly from words.txt
// Inspiration from: https://doc.rust-lang.org/book/ch02-00-guessing-game-tutorial.html
// This assignment will introduce you to some fundamental syntax in Rust:
// - variable declaration
// - string manipulation
// - conditional statements
// - loops
// - vectors
// - files
// - user input
// We've tried to limit/hide Rust's quirks since we'll discuss those details
// more in depth in the coming lectures.
extern crate rand;
use rand::Rng;
use std::collections::HashMap;
use std::fs;
use std::io;
use std::io::Write;
use std::convert::TryInto;

const NUM_INCORRECT_GUESSES: u32 = 5;
const WORDS_PATH: &str = "words.txt";

fn pick_a_random_word() -> String {
    let file_string = fs::read_to_string(WORDS_PATH).expect("Unable to read file.");
    let words: Vec<&str> = file_string.split('\n').collect();
    String::from(words[rand::thread_rng().gen_range(0..words.len())].trim())
}

// Vec<char> 转 String
fn chars_to_string(chars: &Vec<char>) -> String {
    chars.into_iter().collect()
}

// 字符信息
#[derive(Debug)]
struct CharInWord {
    num: u32,         // word中某个字符共有num个
    guessed_num: u32, // 用户在猜字符过程中猜中了该字符多少次
    idxs: Vec<u32>,   // word中该字符的下标集合vec
}

// 统计word中的字符信息，以HashMap<char, CharInWord)形式存储
fn word_statistics(secret_word_chars: &Vec<char>) -> HashMap<char, CharInWord> {
    let mut statistics = HashMap::new();
    for (idx, ch) in secret_word_chars.iter().enumerate() {
        let a = statistics.entry(*ch).or_insert(CharInWord {
            num: 0,
            guessed_num: 0,
            idxs: Vec::new(),
        });
        (*a).num += 1;
        (*a).idxs.push(idx.try_into().unwrap());
    }
    statistics
}

enum Index {
    Yes(u32),
    No,
}

// 判断char是否可以匹配word中未被匹配的字符
// 若可以匹配，则返回 Index::Yes(ch_idx)，ch_idx指匹配到的字符在word中的下标
// 若不能匹配，则返回 Index::No
fn is_char_in_word(ch: char, statistics: &mut HashMap<char, CharInWord>) -> Index {
    match statistics.get_mut(&ch) {
        Some(ciw) => {
            // let ch_idx = ciw.idxs[ciw.guessed_num as usize];
            let ch_idx;
            match ciw.idxs.get(ciw.guessed_num as usize) {
                Some(x) => ch_idx = *x,
                None => {
                    return Index::No;
                }
            }
            ciw.guessed_num += 1;
            Index::Yes(ch_idx)
        }
        None => Index::No,
    }
}

// 读取用户输入并转换为char
fn read_guess_char() -> char {
    print!("Please guess a letter: ");
    // Make sure the prompt from the previous line gets displayed:
    io::stdout().flush().expect("Error flushing stdout.");
    let mut guess = String::new();
    io::stdin()
        .read_line(&mut guess)
        .expect("Error reading line.");
    guess.as_bytes()[0] as char
}

// 游戏主体
fn hangman(secret_word_chars: &Vec<char>) {
    let mut pass = false;
    let mut num_incorrect_guesses: u32 = 0; // 共猜错了几次
    let mut cur_word_chars = vec!['-'; secret_word_chars.len()]; // 用户目前猜中的字符，形式如 --h--d
    let mut statistics = word_statistics(secret_word_chars); // word中的字符统计信息
    let mut cur_guessed_chars = Vec::new(); // 顺序存储用户所有猜测的字符
    while num_incorrect_guesses < 5 {
        println!("The word so far is {}", chars_to_string(&cur_word_chars));
        println!(
            "You have guessed the following letters: {}",
            chars_to_string(&cur_guessed_chars)
        );
        println!(
            "You have {} guesses left",
            NUM_INCORRECT_GUESSES - num_incorrect_guesses
        );

        let ch = read_guess_char();

        if ch.is_alphabetic() {
            match is_char_in_word(ch, &mut statistics) {
                Index::Yes(idx) => {
                    cur_word_chars[idx as usize] = ch;
                    if cur_word_chars == *secret_word_chars {
                        pass = true;
                        break;
                    }
                }
                Index::No => {
                    num_incorrect_guesses += 1;
                    println!("Sorry, that letter is not in the word");
                }
            }
            cur_guessed_chars.push(ch);
        } else {
            panic!("char is not alphabetic"); // 如果用户输入的不是字母直接panic（无脑处理~）
        }
        println!("------------ Round ends ---------------\n")
    }
    if pass {
        println!(
            "Congratulations you guessed the secret word: {}!",
            chars_to_string(&cur_word_chars)
        )
    } else {
        println!("Sorry, you ran out of guesses!")
    }
}

fn main() {
    println!("Welcome to CS110L Hangman!");
    let secret_word = pick_a_random_word();
    // Note: given what you know about Rust so far, it's easier to pull characters out of a
    // vector than it is to pull them out of a string. You can get the ith character of
    // secret_word by doing secret_word_chars[i].
    let secret_word_chars: Vec<char> = secret_word.chars().collect();
    // Uncomment for debugging:
    // println!("random word: {}", secret_word);

    // Your code here! :)
    hangman(&secret_word_chars);
}

