use eframe::{egui::{self, Visuals}, epi};
use native_dialog::{FileDialog, MessageDialog, MessageType};
use std::{fs::{self, FileType}, path::PathBuf, io::{self, BufRead}, collections::HashMap, cell::RefCell};

/// We derive Deserialize/Serialize so we can persist app state on shutdown.
#[cfg_attr(feature = "persistence", derive(serde::Deserialize, serde::Serialize))]
#[cfg_attr(feature = "persistence", serde(default))] // if we add new fields, give them default values when deserializing old state

enum Theme {
    Dark,
    Light
}

#[derive(PartialEq, Eq, Clone, Copy)]
pub enum State {
    NonSelected,
    ToSelect,
    Selected,
    NoOneFound,
}

struct Messege {
    data: Vec<String>,
    //date: String,
}

pub struct ZoomApp {
    theme: Theme,
    student_map: HashMap<String, RefCell<Messege>>,
    state: State,
}

impl ZoomApp {

    fn swap_theme(&mut self, ctx: &egui::Context, theme: Theme) {
        self.theme = theme;
        match self.theme {
            Theme::Dark => {
                let mut visuals = Visuals::dark();
                visuals.override_text_color = Some(egui::color::Color32::WHITE);
                ctx.set_visuals(visuals);
            },
            Theme::Light => {
                ctx.set_visuals(Visuals::light());
            },
        }
    }

    fn open_folder(&mut self) {
        let path = FileDialog::new()
        .set_location("~/")
        .show_open_single_dir()
        .unwrap();

        let path = match path {
            Some(path) => path,
            None => return,
        };

        let yes = MessageDialog::new()
            .set_type(MessageType::Info)
            .set_title("Do you want to open the folder?")
            .set_text(&format!("{} {:#?}", "Are you sure?", path))
            .show_confirm()
            .unwrap();
        
        if yes {
            // read the dir
            self.set_students(path);

        } 
    }

    fn set_students(&mut self, path: PathBuf) -> bool {
        //                                       we can saftely unwrap it as long as it was selected with FileDialog
        for zoom_meeting in fs::read_dir(path).unwrap() {
            if zoom_meeting.is_ok() {
                self.set_state(State::ToSelect);

                let zoom_meeting = zoom_meeting.unwrap();
                // check if it is a folder
                if FileType::is_dir(&zoom_meeting.file_type().unwrap()) {
                    let teacher = &zoom_meeting.file_name();
                    let teacher = teacher.to_str();

                    if teacher.is_none() {
                        return false;
                    }

                    // idk rust well enough to use its iterators to do this
                    let teacher = teacher.unwrap();
                    let mut teacher_buf = String::new();

                    let mut spaces = 0;
                    for c in teacher.chars() {
                        // if spaces is more then two we know it's the teachers name
                        if spaces >= 2 {
                            teacher_buf.push(c);
                        } else if c == ' ' {
                            spaces += 1;
                        }
                    }

                    // remove the end
                    let teacher = &teacher_buf.replace("'s Personal Meeting Room", "");

                    // loop through all the text documents

                    for meeting in fs::read_dir(zoom_meeting.path()).unwrap() {
                        let meeting = meeting.unwrap();
                        if meeting.file_name().into_string().unwrap() == "meeting_saved_chat.txt" {
                            
                            self.interpret_file(meeting.path(), &teacher);

                        }
                    }
                }
            }
        }
        
        true
    }

    fn interpret_file(&mut self, path: PathBuf, teachers_name: &str) {

        let file = fs::File::open(&path).unwrap();

        // let metadata = &file.metadata().unwrap();

        // read each lines to filter out only the students messeges
        for line in io::BufReader::new(&file).lines() {
            // a line
            if line.is_ok() {
                
                let line = line.unwrap();

                // read name
                let mut name = String::new();
                let mut msg = vec![];
                let mut is_filter = false;
                
                if !line.starts_with("    ") {
                    let mut name_buf = String::new();
                   
                    for c in line.chars() {
                        if c == 'm' {
                            is_filter = true;
                        } else if name_buf.ends_with(teachers_name) {
                            name_buf = name_buf.replace(teachers_name, "")
                                               .replace("to", "");
                            name = name_buf.trim().to_string();
                            break;
                        } else if is_filter {
                            name_buf.push(c);
                        }
                    }
                } else if !name.is_empty() { // read a messege only if we know who said it  
                    msg.push(line);
                }

                if name.len() > 0 {
                    if self.student_map.contains_key(&name) {
                        self.student_map.get(&name).unwrap().borrow_mut().data.append(&mut msg);
                    } else {
                        self.student_map.insert(name, RefCell::new(Messege { data: msg } ));
                    }
                }

            }
        }
    }

    fn set_state(&mut self, state: State) {
        self.state = state;
    }

    fn get_state(&self) -> State {
        self.state
    }
}

impl Default for ZoomApp {
    fn default() -> Self {
        Self {
            theme: Theme::Light,
            student_map: HashMap::new(),
            state: State::NonSelected,
        }
    }
}

impl epi::App for ZoomApp {
    fn name(&self) -> &str {
        "Zoom-Chat Interperter"
    }

    /// Called once before the first frame.
    fn setup(
        &mut self,
        _ctx: &egui::Context,
        _frame: &epi::Frame,
        storage: Option<&dyn epi::Storage>,
    ) {
        if let Some(storage) = storage {
            //*self = epi::get_value(storage, epi::APP_KEY).unwrap_or_default()
        }
    }

    fn save(&mut self, storage: &mut dyn epi::Storage) {
        //epi::set_value(storage, epi::APP_KEY, self);
    }

    fn update(&mut self, ctx: &egui::Context, frame: &epi::Frame) {
        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            // The top panel is often a good place for a menu bar:
            egui::menu::bar(ui, |ui| {
                ui.menu_button("File", |ui| {
                    if ui.button("Select Chat Folder").clicked() {
                        self.open_folder();
                    }
                    if ui.button("Quit").clicked() {
                        frame.quit();
                    }
                });
                ui.menu_button("Preferences", |ui| {
                    if ui.button("Change Theme").clicked() {
                        match self.theme {
                            Theme::Dark => {
                                self.swap_theme(ctx, Theme::Light);
                            },
                            Theme::Light => {
                                self.swap_theme(ctx, Theme::Dark);
                            },
                        }
                    }
                });
            });
        });

        if self.get_state() == State::ToSelect || self.get_state() == State::Selected { // found students
            egui::SidePanel::left("side_panel").show(ctx, |ui| {
                ui.spacing_mut().item_spacing.y = 18.0;
                ui.heading("Students");
                ui.spacing_mut().item_spacing.y = 12.0;
                ui.vertical_centered_justified(|ui| {
                    for student in self.student_map.keys() {
                        ui.label(student);
                    }
                });
            });
        }

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.spacing_mut().item_spacing.y = 15.0;
            match self.state {
                State::NonSelected => {
                    ui.heading("Zoom Chat Interperter");
                    ui.label("This program helps you analyse all messeges sent by students on zoom. Specify where this document is located. To gain the chat data there is a export option on zoom.");
                    if ui.button("Select Chat Folder").clicked() {
                        self.open_folder();
                    }
                    egui::warn_if_debug_build(ui);
                },
                State::ToSelect => {
                    ui.heading("Select A Student...");
                },
                State::Selected => {
                    
                },
                State::NoOneFound => {
                    ui.heading("No Student Found...");
                    if ui.button("Select Chat Folder").clicked() {
                        self.open_folder();
                    }
                },
            }

            ui.with_layout(egui::Layout::bottom_up(egui::Align::LEFT), |ui| {
                ui.horizontal(|ui| {
                    ui.centered_and_justified(|ui| {
                        ui.label("© Shalev Haimovitz. All rights reserved.");
                        ui.hyperlink_to("github", "https://github.com/daslastic");
                    });
                });
            });
        });

        //egui::Window::new("User Manual").show(ctx, |ui| {
        //    ui.label("This program filters and allows you to");
        //    ui.label("easily analyse all messeges students have sent");
        //    
        //});
    }
}
