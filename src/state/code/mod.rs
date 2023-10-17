pub mod code_history;
pub mod code_selection;
pub mod code;

use std::{fs::{File, OpenOptions}, io::{Write, Read}, error::Error, path::PathBuf};
use self::{code::{Code, Line}, code_history::CodeHistory, code_selection::CodeSelection};
use clipboard::{ClipboardProvider, ClipboardContext};
use crossterm::event::{KeyEventKind, Event, KeyCode, KeyModifiers, ModifierKeyCode};

use super::{Component, ComponentType, AppContext};

#[derive(Debug, PartialEq, Eq)]
pub struct CodeComponent {
    current: Code,
    history: CodeHistory,
    selection: Option<CodeSelection>,
}

impl Component for CodeComponent {

    fn get_type(&self) -> ComponentType {
        ComponentType::Code
    }

    fn handle_event(&mut self, context: &mut AppContext, event: Event) {
        if let Event::Key(key) = event {
            if key.kind == KeyEventKind::Press {
                match key.code {
                    KeyCode::Char(char) => {

                        let mut char_normalized = char.clone().to_string();
                        char_normalized = char_normalized.to_lowercase().to_string();
                        if char_normalized == "x" && key.modifiers.contains(KeyModifiers::CONTROL) {
                            let mut cut = String::default();
                            if let Some(selection) = self.selection.as_mut() {
                                if selection.is_selecting() {
                                    let code = selection.get_selection();
                                    cut = code.to_string();
                                }
                            }
                            let clipboard: Result<ClipboardContext, Box<dyn Error>> = ClipboardProvider::new();
                            if let Ok(mut context) =  clipboard {
                                let _ = context.set_contents(cut);
                            } 
                            self.selection = None;
                        } else if char_normalized == "c" && key.modifiers.contains(KeyModifiers::CONTROL) {
                            let mut copy = String::default();
                            if let Some(selection) = self.selection.as_mut() {
                                if selection.is_selecting() {
                                    let code = selection.get_selection();
                                    copy = code.to_string()
                                }
                            }
                            let clipboard: Result<ClipboardContext, Box<dyn Error>> = ClipboardProvider::new();
                            if let Ok(mut context) =  clipboard {
                                let _ = context.set_contents(copy);
                            } 
                        } else if char_normalized == "v" && key.modifiers.contains(KeyModifiers::CONTROL) {
                            let clipboard: Result<ClipboardContext, Box<dyn Error>> = ClipboardProvider::new();
                            if let Ok(mut context) =  clipboard {
                                if let Ok(contents) = context.get_contents() {
                                    contents.split("\n").for_each(|line| {
                                        let number = self.current.get_content().into_iter().map(|line| line.get_number()).max().take().unwrap() + 1;
                                        let line = Line::new(number, line.to_string());
                                        let _ = self.current.add_line(line);
                                    });                                
                                }
                            }
                        } else if char_normalized == "s" && key.modifiers.contains(KeyModifiers::CONTROL) {
                                self.history.use_last();
                                let code = self.history.get_current_code();
                                let utf8_code = code.to_string().chars().map(|char| char as u8).fold(vec![], |mut vec, char| {
                                    vec.push(char);
                                    vec
                                });
                                if let Some(path) = context.active_file() {
                                    if path.is_file() {
                                        let f = OpenOptions::new().append(true).open(path);
                                        if let Ok(mut file) = f {
                                            let _ = file.write_all(&utf8_code);
                                        }    
                                    }
                                } else if let Some(path) = context.active_file() {
                                    let f = File::create(path);
                                    if let Ok(mut file) = f {
                                        let _ = file.write_all(&utf8_code);
                                    }
                                } 
    
                        } else if char_normalized == "z" && key.modifiers.contains(KeyModifiers::CONTROL) {
                            self.history.use_previous();
                            let code = self.history.get_current_code();
                            self.current = code.clone();                            
                        } else if char_normalized == "y" && key.modifiers.contains(KeyModifiers::CONTROL) {
                            self.history.use_next();
                            let code = self.history.get_current_code();
                            self.current = code.clone();
                        } else {
                            self.current.remove_cursor();
                            if let Some(current_line) = self.current.get_line(self.get_current().get_x()) {
                                self.current.change_line_at_cursor(current_line.get_string()[..self.get_current().get_y()].to_string() + &char.to_string() + &current_line.get_string()[self.current.get_y()..].to_string());    
                            }
                            self.current.set_y(self.current.get_y()+1);
                            self.current.set_cursor();
                        }
                    },
                    KeyCode::Delete => {
                        let last_number = self.current.get_content().into_iter().map(|x| x.get_number()).fold(0, |line1, line2| {
                            if line1 > line2 { line1 } else { line2 }
                        });
                        let last_line = self.current.get_line(last_number).unwrap();
                        self.current.change_line(last_line.get_number(), last_line.get_string()[..last_line.get_string().len()-1].to_string());
                    },
                    KeyCode::Enter => {
                        {
                            let mut_code = self.get_mut_current();
                            mut_code.remove_cursor();    
                        }
                        let code = self.get_current().clone();
                        let mut_code = self.get_mut_current();
                        if let Some(current_line) = code.get_content().get(code.get_x()) {
                            let line_number = current_line.get_number().clone();
                            let new_current_string = current_line.get_string()[..code.get_y()].to_string().clone();
                            let new_generated_string = current_line.get_string()[code.get_y()..].to_string().clone();
                            mut_code.flush();
                            for number in 0 .. line_number {
                                if let Some(line) = code.get_line(number) {
                                    mut_code.add_line(line.clone());                                    
                                }
                            }
                            mut_code.add_line(Line::new(current_line.get_number(), new_current_string));
                            mut_code.set_x(code.get_x());
                            mut_code.set_y(code.get_y());
                            mut_code.add_line(Line::new(current_line.get_number() + 1, new_generated_string));
                            for number in current_line.get_number() + 1.. code.get_content().len() {
                                if let Some(line) = code.get_line(number) {
                                    let mut new_line = line.clone();
                                    new_line.set_number(number + 1);
                                    mut_code.add_line(new_line.clone());                                    
                                }
                            }
                            mut_code.set_cursor();
                        }
                    },
                    KeyCode::Up => {
                        let mut current_line = self.current.get_x();
                        if current_line > 0 {
                            self.current.remove_cursor();
                            current_line -= 1;
                            self.current.set_x(current_line);
                            if let Some(line) = self.current.get_content().get(current_line) {
                                if line.get_string().len() < self.get_current().get_y() {
                                    self.current.set_y(line.get_string().len() - 1);
                                }
                            }
                            self.current.set_cursor();
                        }
                    },
                    KeyCode::Down => {
                        let mut current_line = self.current.get_x();
                        if current_line < self.current.get_content().len() - 1 {
                            self.current.remove_cursor();
                            current_line += 1;
                            self.current.set_x(current_line);
                            if let Some(line) = self.current.get_content().get(current_line) {
                                if line.get_string().len() < self.get_current().get_y() {
                                    if line.get_string().len() == 0 {
                                        self.current.set_y(0);                                        
                                    } else {
                                        self.current.set_y(line.get_string().len() - 1);
                                    }
                                }
                            }
                            self.current.set_cursor();

                        }
                    },
                    KeyCode::Left => {
                        let mut current_char = self.current.get_y();
                        if current_char > 0 {
                            self.current.remove_cursor();
                            current_char -= 1;
                            self.current.set_y(current_char);
                            self.current.set_cursor();
                        }
                    },
                    KeyCode::Right => {
                        let actual_code = self.get_current();
                        let mut current_char = self.current.get_y();
                        if let Some(line) = actual_code.get_content().get(actual_code.get_x()) {
                            if current_char < line.get_string().len() - 1{
                                self.current.remove_cursor();
                                current_char += 1;
                                self.current.set_y(current_char);
                                self.current.set_cursor();
                            }
                        }
                    },
                    KeyCode::Modifier(ModifierKeyCode::LeftShift) => {
                        if let Some(selection) = &mut self.selection {
                            if let Some(current_line) = selection.get_selection().get_line(self.current.get_x()) {
                                if self.current.get_y() > 0 {
                                    let new_value = current_line.get_string().chars().enumerate()
                                    .filter(|tuple| tuple.0 < self.current.get_y() - 1)
                                    .map(|tuple| tuple.1)
                                    .fold(String::default(), |mut char1, char2| {
                                        char1.push(char2);
                                        char1
                                    });
                                    selection.get_selection().change_line_at_cursor(new_value);    
                                    self.current.set_y(self.current.get_y() -1);    
                                }
                            }
                        }
                    },
                    KeyCode::Modifier(ModifierKeyCode::RightShift) => {
                        if let Some(selection) = &mut self.selection {
                            if let Some(current_line) = selection.get_selection().get_line(self.current.get_x()) {
                                if self.current.get_y() < current_line.get_string().len()-1 {
                                    let new_value = current_line.get_string().chars().enumerate()
                                    .filter(|tuple| tuple.0 < self.current.get_y() + 1)
                                    .map(|tuple| tuple.1).fold(String::default(), |mut char1, char2| {
                                        char1.push(char2);
                                        char1
                                    });
                                    selection.get_selection().change_line_at_cursor(new_value);
                                    self.current.set_y(self.current.get_y() +1);    
                                }
    
                            }
                        }
                    },
                    KeyCode::Esc => {
                        context.set_focus(None);
                        context.set_hover(self.get_type());             
                    },
                    _ => {}
                }
            } if key.kind == KeyEventKind::Repeat {
                match key.code {
                    KeyCode::Char(char) => {
                        self.current.remove_cursor();
                        if let Some(current_line) = self.current.get_line(self.get_current().get_x()) {
                            self.current.change_line_at_cursor(current_line.get_string()[..self.get_current().get_y()].to_string() + &char.to_string() + &current_line.get_string()[self.current.get_y()..].to_string());    
                        }
                        self.current.set_y(self.current.get_y()+1);
                        self.current.set_cursor();
                    },
                    KeyCode::Delete => {
                        let last_number = self.current.get_content().into_iter().map(|x| x.get_number()).fold(0, |line1, line2| {
                            if line1 > line2 { line1 } else { line2 }
                        });
                        let last_line = self.current.get_line(last_number).unwrap();
                        self.current.change_line(last_line.get_number(), last_line.get_string()[..last_line.get_string().len()-1].to_string());
                    },
                    KeyCode::Enter => {
                        {
                            let mut_code = self.get_mut_current();
                            mut_code.remove_cursor();    
                        }
                        let code = self.get_current().clone();
                        let mut_code = self.get_mut_current();
                        if let Some(current_line) = code.get_content().get(code.get_x()) {
                            let line_number = current_line.get_number().clone();
                            let new_current_string = current_line.get_string()[..code.get_y()].to_string().clone();
                            let new_generated_string = current_line.get_string()[code.get_y()..].to_string().clone();
                            mut_code.flush();
                            for number in 0 .. line_number {
                                if let Some(line) = code.get_line(number) {
                                    mut_code.add_line(line.clone());                                    
                                }
                            }
                            mut_code.add_line(Line::new(current_line.get_number(), new_current_string));
                            mut_code.set_x(code.get_x());
                            mut_code.set_y(code.get_y());
                            mut_code.add_line(Line::new(current_line.get_number() + 1, new_generated_string));
                            for number in current_line.get_number() + 1.. code.get_content().len() {
                                if let Some(line) = code.get_line(number) {
                                    let mut new_line = line.clone();
                                    new_line.set_number(number + 1);
                                    mut_code.add_line(new_line.clone());                                    
                                }
                            }
                            mut_code.set_cursor();
                        }
                    },
                    KeyCode::Up => {
                        let mut current_line = self.current.get_x();
                        if current_line > 0 {
                            self.current.remove_cursor();
                            current_line -= 1;
                            self.current.set_x(current_line);
                            if let Some(line) = self.current.get_content().get(current_line) {
                                if line.get_string().len() < self.get_current().get_y() {
                                    self.current.set_y(line.get_string().len() - 1);
                                }
                            }
                            self.current.set_cursor();
                        }
                    },
                    KeyCode::Down => {
                        let mut current_line = self.current.get_x();
                        if current_line < self.current.get_content().len() - 1 {
                            self.current.remove_cursor();
                            current_line += 1;
                            self.current.set_x(current_line);
                            if let Some(line) = self.current.get_content().get(current_line) {
                                if line.get_string().len() < self.get_current().get_y() {
                                    self.current.set_y(line.get_string().len() - 1);
                                }
                            }
                            self.current.set_cursor();

                        }
                    },
                    KeyCode::Left => {
                        let mut current_char = self.current.get_y();
                        if current_char > 0 {
                            self.current.remove_cursor();
                            current_char -= 1;
                            self.current.set_y(current_char);
                            self.current.set_cursor();
                        }
                    },
                    KeyCode::Right => {
                        let actual_code = self.get_current();
                        let mut current_char = self.current.get_y();
                        if let Some(line) = actual_code.get_content().get(actual_code.get_x()) {
                            if current_char < line.get_string().len() - 1{
                                self.current.remove_cursor();
                                current_char += 1;
                                self.current.set_y(current_char);
                                self.current.set_cursor();
                            }
                        }
                    },
                    _ => {}
                }
            }
        }
    }
}

impl CodeComponent {

    pub fn new() -> Self {
        let code = Code::new();
        CodeComponent {
            current: code.clone(),
            history: CodeHistory::new(code.clone()),
            selection: None,
        }
    }

    pub fn set_current(&mut self, active_file: Option<PathBuf>) {
        if let Some(path) = active_file {
            let file = File::open(path);
            if let Ok(mut file) = file {
                let mut contents = String::new();
                let _ = file.read_to_string(&mut contents);
                contents
                .split("\n")
                .enumerate()
                .for_each(|tuple| {
                    let line = Line::new(tuple.0, tuple.1.to_string());
                    self.current.add_line(line);
                })
            }
            self.current.set_cursor();
        }
    }

    pub fn get_current(&self) -> &Code {
        &self.current
    }

    pub fn get_mut_current(&mut self) -> &mut Code {
        &mut self.current
    }

    pub fn get_history(&self) -> &CodeHistory {
        &self.history
    }

    pub fn get_selection(&self) -> &Option<CodeSelection> {
        &self.selection
    }
}