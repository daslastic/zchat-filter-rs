use eframe::{egui::{self, Visuals}, epi};
use native_dialog::{FileDialog, MessageDialog, MessageType};
use std::{fs::{self, FileType, DirEntry}, path::PathBuf};

/// We derive Deserialize/Serialize so we can persist app state on shutdown.
#[cfg_attr(feature = "persistence", derive(serde::Deserialize, serde::Serialize))]
#[cfg_attr(feature = "persistence", serde(default))] // if we add new fields, give them default values when deserializing old state

enum Theme {
    Dark,
    Light
}

pub struct ZoomApp {
    theme: Theme,
}

impl ZoomApp {
    fn swap_theme(&mut self, _ctx: &egui::Context, theme: Theme) {
        self.theme = theme;
        match self.theme {
            Theme::Dark => {
                let mut visuals = Visuals::dark();
                visuals.override_text_color = Some(egui::color::Color32::WHITE);
                _ctx.set_visuals(visuals);
            },
            Theme::Light => {
                _ctx.set_visuals(Visuals::light());
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

    fn set_students(&mut self, path: PathBuf) {
        //                                       we can saftely unwrap it as long as it was selected with FileDialog
        for zoom_meeting in fs::read_dir(path).unwrap() {
            if zoom_meeting.is_ok() {
                let zoom_meeting = zoom_meeting.unwrap();
                // check if it is a folder
                if FileType::is_dir(&zoom_meeting.file_type().unwrap()) {
                    // loop through all the text documents
                    for meeting in fs::read_dir(zoom_meeting.path()).unwrap() {
                        let meeting = meeting.unwrap();
                        if meeting.file_name().into_string().unwrap() == "meeting_saved_chat.txt" {
                            
                            self.interpret_file(meeting.path());

                        }
                    }

                }
            }
        }
    }

    fn interpret_file(&mut self, path: PathBuf) {

        let file = fs::File::open(path).unwrap();

        println!("{:?}", file);

    }
}

impl Default for ZoomApp {
    fn default() -> Self {
        Self {
            theme: Theme::Light
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
        _storage: Option<&dyn epi::Storage>,
    ) {

        // Load previous app state (if any).
        // Note that you must enable the `persistence` feature for this to work.
        #[cfg(feature = "persistence")]
        if let Some(storage) = _storage {
            *self = epi::get_value(storage, epi::APP_KEY).unwrap_or_default()
        }
    }

    /// Called by the frame work to save state before shutdown.
    /// Note that you must enable the `persistence` feature for this to work.
    #[cfg(feature = "persistence")]
    fn save(&mut self, storage: &mut dyn epi::Storage) {
        epi::set_value(storage, epi::APP_KEY, self);
    }

    fn update(&mut self, ctx: &egui::Context, frame: &epi::Frame) {
        //let Self { label, value } = self;

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

        if false { // found students
            egui::SidePanel::left("side_panel").show(ctx, |ui| {
                ui.horizontal(|ui| {
                    ui.heading("Students");
                    ui.spacing_mut().item_spacing.y = 15.0;
                });
            });
        }

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.spacing_mut().item_spacing.y = 15.0;
            if true { // no zoom file
                ui.heading("Zoom Chat Interperter");
                ui.label("This program helps you analyse all messeges sent by students on zoom. Specify where this document is located. To gain the chat data there is a export option on zoom.");
                if ui.button("Select Chat Folder").clicked() {
                    self.open_folder();
                }
            }

            ui.with_layout(egui::Layout::bottom_up(egui::Align::LEFT), |ui| {
                ui.horizontal(|ui| {
                    ui.centered_and_justified(|ui| {
                        ui.label("© Shalev Haimovitz. All rights reserved.");
                    });
                    //ui.hyperlink_to("github", "https://github.com/daslastic");
                });
            });
            egui::warn_if_debug_build(ui);
        });

        //egui::Window::new("User Manual").show(ctx, |ui| {
        //    ui.label("This program filters and allows you to");
        //    ui.label("easily analyse all messeges students have sent");
        //    
        //});
    }
}
