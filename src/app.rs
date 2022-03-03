use eframe::{
    egui::{self, FontData, FontTweak, Visuals},
    epaint::FontFamily,
    epi,
};
use native_dialog::{FileDialog, MessageDialog, MessageType};
use serde::{Deserialize, Serialize};
use std::{
    cell::RefCell,
    collections::HashMap,
    fs::{self, FileType},
    io::{self, BufRead},
    path::PathBuf,
};

/// We derive Deserialize/Serialize so we can persist app state on shutdown.
#[cfg_attr(feature = "persistence", derive(serde::Deserialize, serde::Serialize))]
#[cfg_attr(feature = "persistence", serde(default))] // if we add new fields, give them default values when deserializing old state
#[derive(Serialize, Deserialize, Clone, Copy)]
enum Theme {
    Dark,
    Light,
}

#[derive(PartialEq, Eq, Clone, Copy, Serialize, Deserialize)]
pub enum State {
    NonSelected,
    ToSelect,
    Selected,
    NoOneFound,
}

#[derive(PartialEq, Eq, Clone, Serialize, Deserialize)]
struct Messege {
    pub data: String,
    time: String,
}

#[derive(Serialize, Deserialize)]
pub struct ZoomApp {
    theme: Theme,
    student_map: HashMap<String, RefCell<Vec<Messege>>>,
    state: State,
    selected_student: Option<String>,
    filter_query: String,
    total_messeges: usize,
    scale: f32,
}

impl ZoomApp {
    fn swap_theme(&mut self, ctx: &egui::Context, theme: Theme) {
        self.theme = theme;
        match self.theme {
            Theme::Dark => {
                let mut visuals = Visuals::dark();
                visuals.override_text_color = Some(egui::color::Color32::WHITE);
                ctx.set_visuals(visuals);
            }
            Theme::Light => {
                ctx.set_visuals(Visuals::light());
            }
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
        // set back to 0 as none should have been interpered
        self.total_messeges = 0;

        // read each lines to filter out only the students messeges
        let mut name = String::new();
        let mut time = String::new();

        for line in io::BufReader::new(&file).lines() {
            // a line
            if line.is_ok() {
                let line = line.unwrap();
                let mut is_filter = false;

                if !line.starts_with("	") {
                    let mut name_buf = String::new();

                    for c in line.chars() {
                        if c == 'm' {
                            is_filter = true;
                        } else if name_buf.ends_with(teachers_name) {
                            name_buf = name_buf.replace(teachers_name, "").replace("to", "");
                            name = name_buf.trim().to_string();
                            break;
                        } else if is_filter {
                            name_buf.push(c);
                        }
                    }

                    let mut time_buf = String::new();
                    let mut word_i = 0;
                    for word in line.split(' ') {
                        if word_i == 1 {
                            time = time_buf.trim().to_string();
                            break;
                        }
                        time_buf += word;
                        word_i += 1;
                    }
                } else if !name.is_empty() {
                    // read a messege only if we know who said it
                    if name.len() > 0 {
                        if self.student_map.contains_key(&name) {
                            self.student_map
                                .get(&name)
                                .unwrap()
                                .borrow_mut()
                                .push(Messege {
                                    data: line.trim().to_string(),
                                    time: time.to_owned(),
                                });
                        } else {
                            self.student_map.insert(
                                name.clone(),
                                RefCell::new(vec![Messege {
                                    data: line.trim().to_string(),
                                    time: time.to_owned(),
                                }]),
                            );
                        }
                        self.total_messeges += 1;
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
            selected_student: None,
            filter_query: String::new(),
            total_messeges: 0,
            scale: 2.0,
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
        ctx: &egui::Context,
        _frame: &epi::Frame,
        storage: Option<&dyn epi::Storage>,
    ) {
        if let Some(storage) = storage {
            *self = epi::get_value(storage, epi::APP_KEY).unwrap_or_default()
        }

        self.swap_theme(ctx, self.theme.clone());

        let mut fonts = egui::FontDefinitions::default();
        let poppins = FontData {
            font: std::borrow::Cow::Borrowed(include_bytes!("../fonts/Poppins-Regular.ttf")),
            index: 0,
            tweak: FontTweak::default(),
        };
        let poppins_bold = FontData {
            font: std::borrow::Cow::Borrowed(include_bytes!("../fonts/Poppins-Bold.ttf")),
            index: 1,
            tweak: FontTweak::default(),
        };

        fonts.font_data.insert("Poppins".to_owned(), poppins);
        fonts
            .font_data
            .insert("Poppins-Bold".to_owned(), poppins_bold);
        fonts
            .families
            .get_mut(&FontFamily::Monospace)
            .unwrap()
            .insert(0, "Poppins-Bold".to_owned());
        fonts
            .families
            .get_mut(&FontFamily::Proportional)
            .unwrap()
            .insert(0, "Poppins".to_owned());

        ctx.set_fonts(fonts);
    }

    fn save(&mut self, storage: &mut dyn epi::Storage) {
        epi::set_value(storage, epi::APP_KEY, self);
    }

    fn update(&mut self, ctx: &egui::Context, frame: &epi::Frame) {
        ctx.set_pixels_per_point(self.scale);

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
                            }
                            Theme::Light => {
                                self.swap_theme(ctx, Theme::Dark);
                            }
                        }
                    }

                    if ui.button("Zoom in +").clicked() {
                        self.scale += 0.1;
                    }
                    if ui.button("Zoom out -").clicked() {
                        if !(self.scale < 0.5) {
                            self.scale -= 0.1;
                        }
                    }
                });
            });
        });

        if self.get_state() == State::ToSelect || self.get_state() == State::Selected {
            // found students
            egui::SidePanel::left("side_panel").show(ctx, |ui| {
                ui.spacing_mut().item_spacing.y = 30.0;
                ui.vertical_centered_justified(|ui| {
                    egui::ScrollArea::vertical().show(ui, |ui| {
                        ui.heading("Students");
                        ui.spacing_mut().item_spacing.y = 12.0;
                        for student in self.student_map.keys() {
                            if ui.button(student).clicked() {
                                self.selected_student = Some(student.to_owned());
                                self.state = State::Selected;
                            }
                        }
                    });
                });
            });
        }

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.spacing_mut().item_spacing.y = 15.0;
            ui.set_width(ui.available_width());
            match self.state {
                State::NonSelected => {
                    ui.heading("Zoom Chat Interperter");
                    ui.label("This program helps you analyse all messeges sent by students on zoom. Specify where the student data is. It is a export option on zoom.\nCreated by Shalev, a student of Discovery Academy");
                    if ui.button("Select Chat Folder").clicked() {
                        self.open_folder();
                    }
                    egui::warn_if_debug_build(ui);
                },
                State::ToSelect => {
                    ui.heading("Select A Student.");
                    ui.label("Wrong Folder?");
                    if ui.button("Select Chat Folder").clicked() {
                        self.open_folder();
                    }
                },
                State::Selected => {
                    if self.selected_student.is_none() {
                        self.selected_student = None;
                        self.state = State::NoOneFound;
                    } else {
                        let student_name = self.selected_student.clone().unwrap();
                        ui.heading(&student_name);
                        let student = &self.student_map.get(&student_name).unwrap();

                        ui.spacing_mut().item_spacing.y = 5.0;
                        ui.horizontal(|ui| {
                            ui.label("Filter: ");
                            ui.text_edit_singleline(&mut self.filter_query);
                        });

                        ui.spacing_mut().item_spacing.y = 12.0;
                        ui.horizontal(|ui| {
                            let freq_count = student.borrow().len()/self.total_messeges;
                            let mut freq = String::new();
                            freq.push_str("Frequency: ");
                            freq.push_str(&*freq_count.to_string());
                            freq.push_str("%");
                            ui.label(freq);
    
                            ui.label("-");

                            let av_freq_count = self.student_map.keys().len()/self.total_messeges;
                            let mut av_freq = String::new();
                            av_freq.push_str("Average Frequency: ");
                            av_freq.push_str(&*av_freq_count.to_string());
                            av_freq.push_str("%");
                            ui.label(av_freq);
                        });

                        ui.vertical(|ui| {
                            egui::ScrollArea::vertical().show(ui, |ui| {
                                student.borrow().iter().for_each(| messege | {
                                    let mut final_msg = String::new();
                                    if self.filter_query.len() > 0 {
                                        if *&messege.data.to_lowercase().contains(&self.filter_query.to_lowercase()) {
                                            final_msg.push_str(&student_name);
                                            final_msg.push_str(" at ");
                                            final_msg.push_str(&messege.time);
                                            final_msg.push_str(" : ");
                                            final_msg.push_str(&messege.data);
                                            ui.label(final_msg);
                                        }
                                    } else {
                                        final_msg.push_str(&student_name);
                                        final_msg.push_str(" at ");
                                        final_msg.push_str(&messege.time);
                                        final_msg.push_str(" : ");
                                        final_msg.push_str(&messege.data);
                                        ui.label(final_msg);
                                    }
                                });
                            });
                        });
                    }
                },
                State::NoOneFound => {
                    ui.heading("No Student Found...");
                    if ui.button("Select Chat Folder").clicked() {
                        self.open_folder();
                    }
                },
            }

            if self.get_state() != State::Selected {
                ui.with_layout(egui::Layout::bottom_up(egui::Align::Center), |ui| {
                    ui.label("© Shalev Haimovitz. All rights reserved.");
                    ui.hyperlink_to("github", "https://github.com/daslastic");
                });
            }
        });
    }
}
