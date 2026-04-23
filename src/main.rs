#![allow(non_snake_case)]
#![allow(non_camel_case_types)]

use crossterm::{
    cursor::{
        Hide,
        MoveTo,
        RestorePosition,
        SavePosition,
        MoveToColumn,
    }, event::{
        self,
        Event,
        KeyCode,
        KeyEvent,
    },
    execute,
    style::Print,
    terminal::{
        self,
        Clear,
        ClearType,
    }
};
use std::{
    time::Duration,
    io::stdout,
    process::exit,
};

struct ExitEvent;

impl Drop for ExitEvent {
    fn drop(&mut self) {
        let _ = terminal::disable_raw_mode();
    }
}

fn readKeyInput() -> KeyCode {
    loop {
        if event::poll(Duration::from_millis(500)).unwrap() {
            return event::read().unwrap().as_key_event().unwrap().code;
        }
    }
}

fn writeToLine(text: &str, position: u16) {
    execute!(
        stdout(),
        MoveToColumn(0),
        Clear(ClearType::CurrentLine),
        Print(text),
        MoveToColumn(position)
    ).unwrap();
}

fn textInput(prompt: &str) -> String {
    println!("{prompt}\r");

    let mut position: u16 = 0;
    let mut response = String::new();

    loop {
        match readKeyInput() {
            KeyCode::Enter => break,
            KeyCode::Backspace => {
                response.pop();
                writeToLine(&response, position);
            },
            KeyCode::Up => {
                position = 0;
                writeToLine(&response, position);
            },
            KeyCode::Down => {
                position = response.len() as u16;
                writeToLine(&response, position);
            },
            KeyCode::Left => {
                if position > 0 {
                    position -= 1;
                    writeToLine(&response, position);
                }
            },
            KeyCode::Right => {
                if position < response.len() as u16 {
                    position += 1;
                    writeToLine(&response, position);
                }
            },
            KeyCode::Char(key) => {
                if !key.is_control() {
                    response.push(key);
                    position += 1;
                    writeToLine(&response, position);
                }
            },
            _ => {}
        }
    }
    return response;
}

fn main () {
    let _exit_event = ExitEvent;

    terminal::enable_raw_mode().unwrap();
    println!("{}", textInput("Input your name:"));
    /*
    loop {
        if event::poll(Duration::from_millis(500)).unwrap() {
            match event::read().unwrap() {
                Event::Key(KeyEvent { code, ..}) => match code {
                    KeyCode::Esc | KeyCode::Char('q') => break,
                    KeyCode::Char(c)   => println!("Key: {c}\r"),
                    _ => {}
                }
                _ => {}
            }
        }
    }
    */


}