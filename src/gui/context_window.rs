use core::fmt;
use std::borrow::Cow;
use std::sync::{Arc, Mutex};

use crate::auth::AuthState;
use crate::errors::MatrixError;
use crate::{events, gui};
use crate::gui::login::{OutputWidget, TextBoxWidget};
use crate::gui::messages::HeaderWidget;
use crossterm::event::{KeyCode, KeyEvent};
use ratatui::buffer::Buffer;
use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::Widget;

pub trait ContextWindow: fmt::Debug {
    fn new(auth: AuthState) -> Self
    where
        Self: Sized;

    fn get_fields(&self) -> &Vec<(String, String)>;
    fn get_pointer_index(&self) -> usize;
    fn get_selection(&self) -> usize;
    fn gen_output(&self) -> std::borrow::Cow<'_, str>; // Allows borrowed strings
    fn clone_box(&self) -> Box<dyn ContextWindow>;
    fn add_character(&mut self, c: char);
    fn backspace(&mut self);
    fn increment_selection(&mut self, i: i32);
    fn increment_pointer(&mut self, i: i32);
    fn enter(&self) -> bool;
    fn handle_key_events(&mut self, keyevent: KeyEvent) -> bool {
        match keyevent.code {
            KeyCode::Esc => return true,
            KeyCode::Enter => return self.enter(),
            KeyCode::Char(c) => self.add_character(c),
            KeyCode::Backspace => self.backspace(),
            KeyCode::BackTab => self.increment_selection(-1),
            KeyCode::Tab => self.increment_selection(1),
            KeyCode::Right => self.increment_pointer(1),
            KeyCode::Left => self.increment_pointer(-1),
            _ => {}
        }
        return false
    }
    fn render(&self, area: Rect, buf: &mut Buffer) {
        let layout_header = vec![Constraint::Length(1), Constraint::Fill(1)];

        // Populate layout with the proper spaces for the input fields
        let layout_inputs: Vec<Constraint> = (0..self.get_fields().len())
            .flat_map(|_| vec![Constraint::Fill(1), Constraint::Length(3)])
            .collect();

        let layout_bottom = vec![Constraint::Length(1), Constraint::Fill(10)];

        let layout = [layout_header, layout_inputs, layout_bottom].concat();
        // Example layout with three input boxes
        // * meaning not a blank spaceelf.output.clone().lock().unwrap()
        // [
        //     Constraint::Length(1), // header *
        //     Constraint::Fill(1),   // header

        //     Constraint::Fill(1),   // inputs 1
        //     Constraint::Length(3), // inputs 1 *
        //     Constraint::Fill(1),   // inputs 2
        //     Constraint::Length(3), // inputs 2 *
        //     Constraint::Fill(1),   // inputs 3
        //     Constraint::Length(3), // inputs 3 *

        //     Constraint::Length(1), // bottom  *
        //     Constraint::Fill(10),  // bottom
        // ]

        let sections: Vec<Rect> = Layout::vertical(layout).split(area).to_vec();

        // add the "header" bar with the hints
        let header = sections[0];
        HeaderWidget::new(vec![
            ("<tab>".to_string(), "switch".to_string()),
            ("<enter>".to_string(), "submit".to_string()),
            ("<esc>".to_string(), "exit".to_string()),
        ])
        .render(header, buf);

        // Render the text boxes
        
        for i in 0..self.get_fields().len() {
            let text_area = sections[3 + (i * 2)];
            let value: &(String, String) = &self.get_fields()[i]; // indexes the current textbox: see example
            let with_cursor: Line<'static> = {
                if self.get_selection() != i {
                    Line::from(value.1.to_string())
                } else {
                    let mut chars: Vec<char> = value.1.chars().collect();
                    chars.push(' ');

                    let n = self.get_pointer_index();
                    let before: String = chars[..n].iter().collect();
                    let target: String = chars[n..n + 1].iter().collect();
                    let after: String = chars[n + 1..].iter().collect();

                    Line::from(vec![
                        Span::raw(before),
                        Span::styled(target, Style::default().add_modifier(Modifier::UNDERLINED)),
                        Span::raw(after),
                    ])
                }
            };
            TextBoxWidget::new(
                value.0.clone(),
                with_cursor,
                self.get_selection() == i,
                false,
            )
            .render(text_area, buf);
        }

        // Render the bottom message 0
        let output_area = sections[sections.len() - 2]; // gets the second to last element
        OutputWidget::new(self.gen_output().to_string()).render(output_area, buf);
    }
}

impl Clone for Box<dyn ContextWindow> {
    fn clone(&self) -> Box<dyn ContextWindow> {
        self.clone_box()
    }
}

#[derive(Debug, Clone)]
pub struct RoomCreation {
    fields: Vec<(String, String)>,
    selection: usize,
    output: Arc<Mutex<String>>,
    pointer_index: usize,
    auth: AuthState
}

impl ContextWindow for RoomCreation {
    fn new(auth: AuthState) -> Self {
        RoomCreation {
            fields: vec![
                ("Room Name".to_string(), String::new()),
            ],
            selection: 0,
            output: Arc::new(Mutex::new(String::new())),
            pointer_index: 0,
            auth
        }
    }

    fn get_fields(&self) -> &Vec<(String, String)> {
        &self.fields
    }

    fn get_selection(&self) -> usize {
        self.selection
    }

    fn get_pointer_index(&self) -> usize {
        self.pointer_index
    }

    fn gen_output(&self) -> std::borrow::Cow<'_, str> {
        Cow::Owned(self.output.clone().lock().unwrap().clone())
    }

    // AI function
    fn increment_selection(&mut self, i: i32) {
        let len = self.fields.len() as i32;
        self.selection = (self.selection as i32 + i).rem_euclid(len) as usize;
        self.pointer_index = self.fields[self.selection].1.len();
    }
    // AI function
    fn increment_pointer(&mut self, i: i32) {
        let max = self.fields[self.selection].1.chars().count() as i32;
        self.pointer_index = (self.pointer_index as i32 + i).clamp(0, max) as usize;
    }

    fn clone_box(&self) -> Box<dyn ContextWindow> {
        Box::new(self.clone())
    }

    fn add_character(&mut self, c: char) {
        self.fields[self.selection].1.insert(self.pointer_index, c);
        self.pointer_index += 1;
    }

    fn backspace(&mut self) {
        if self.pointer_index == 0 { return; }
        self.fields[self.selection].1.remove(self.pointer_index -1);
        self.pointer_index -= 1;
    }

    fn enter(&self) -> bool {
        let room_name = self.fields.get(0).unwrap().1.clone();

        let auth = self.auth.clone();
        let response = std::thread::spawn(move || {
            let rt = tokio::runtime::Runtime::new().unwrap();
            rt.block_on(events::create_room(auth, &room_name, vec![], "private_chat"))
        })
        .join()
        .unwrap();

        let mut output_lock = self.output.lock().unwrap();
        match response {
            Ok(_) => return true, // Room created successfully 
            Err(e) => {
                match e.downcast_ref::<gui::context_window::MatrixError>() {
                    Some(MatrixError::MatrixError(errcode, errmsg)) => {
                        // errcode: &String, errmsg: &Option<String>
                        // handle it, e.g.:
                        *output_lock = errmsg.clone().unwrap_or_else(|| errcode.clone());
                    }
                    None => {
                        // it's some other error type boxed in there — fall back to Display
                        *output_lock = e.to_string();
                    }
                }
                return false
            }
        }
    }
}
