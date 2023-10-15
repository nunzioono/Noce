use std::{fmt::{self}, ops::Add, path::PathBuf, fs::File, io::Read};



#[derive(Clone, Default, Debug, PartialEq, Eq)]
pub struct Line {
    number: usize,
    line: String,
}

impl Line {
    pub fn new(number: usize, line: String) -> Line {
        Line { number, line }
    }

    pub fn set_number(&mut self, number: usize) {
        self.number = number;
    }

    pub fn get_number(&self) -> usize {
        self.number
    }

    pub fn set_string(&mut self, line: String) {
        self.line = line;
    }

    pub fn get_string(&self) -> String {
        self.line.clone()
    }
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Code {
    content: Vec<Line>,
    x: usize,
    y: usize
}

impl fmt::Display for Code {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for line in &self.content {
            writeln!(f, "{}: {}", line.number, line.line)?;
        }
        Ok(())
    }
}

impl Add<Line> for Code {
    type Output = Code;

    fn add(mut self, line: Line) -> Code {
        self.content.push(line);
        self
    }
}

impl Code {
    pub fn new(active_file: Option<PathBuf>) -> Code {
        let mut lines = Vec::new();
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
                    lines.push(line);
                })
            }
        }
        Code { content: lines, x: 0, y: 0 }
    }

    pub fn get_x(&self) -> usize {
        self.x
    } 

    pub fn get_y(&self) -> usize {
        self.y
    } 

    pub fn set_x(&mut self, x: usize) -> &mut Self {
        self.x = x;
        self
    } 

    pub fn set_y(&mut self, y: usize) -> &mut Self {
        self.y = y;
        self
    } 
    
    pub fn remove_line(&mut self, number: usize) -> &mut Code {
        self.content.retain(|line| line.number != number);
        self
    }

    pub fn remove_line_at_cursor(&mut self) -> &mut Code {
        self.content.retain(|line| line.number != self.x);
        self
    }

    pub fn change_line(&mut self, number: usize, new_value: String) -> &mut Code {
        for line in &mut self.content {
            if line.number == number {
                line.line = new_value;
                break;
            }
        }
        self
    }

    pub fn change_line_at_cursor(&mut self, new_value: String) -> &mut Code {
        for line in &mut self.content {
            if line.number == self.x {
                line.line = new_value;
                break;
            }
        }
        self
    }

    pub fn add_line(&mut self, line: Line) -> &mut Code {
        self.content.push(line);
        self
    }

    pub fn get_line(&self, number: usize) -> Option<&Line> {
        self.content.iter().find(|line| line.number == number)
    }

    pub fn get_content(&self) -> &Vec<Line> {
        &self.content
    }

}