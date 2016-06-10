use backend::{self, Arguments, logging, tokenizer};
use backend::traits::ToFilename;

use gdk::enums::key;
use gtk::prelude::*;
use gtk::{
    self, Builder, Button, CheckButton, Entry, FileChooserDialog, ListStore,
    SpinButton, TreeView, TreeViewColumn, Type, Window, WindowType
};
use std::fs;
use std::path::PathBuf;

/// Allow drag-and-drop support in the directory entry text field by fixing the URI generated by dropped files.
#[inline]
fn parse_directory(directory: &str) -> String {
    let mut output = String::from(directory);
    if output.starts_with("file://") {
        output = output.replace("file://", "");
        output = output.replace("%20", " ");
        output = output.replace("\n", "");
        output = output.replace("\r", "");
    }
    output
}

pub fn launch() {
    gtk::init().unwrap_or_else(|_| panic!("tv-renamer: failed to initialize GTK."));

    // Open the Glade GTK UI and import key GTK objects from the UI.
    let builder = Builder::new_from_string(include_str!("gtk_interface.glade"));
    let window: Window                  = builder.get_object("main_window").unwrap();
    let preview_button: Button          = builder.get_object("preview_button").unwrap();
    let rename_button: Button           = builder.get_object("rename_button").unwrap();
    let series_name_entry: Entry        = builder.get_object("series_name_entry").unwrap();
    let series_directory_entry: Entry   = builder.get_object("series_directory_entry").unwrap();
    let template_entry: Entry           = builder.get_object("template_entry").unwrap();
    let series_directory_button: Button = builder.get_object("series_directory_button").unwrap();
    let episode_spin_button: SpinButton = builder.get_object("episode_spin_button").unwrap();
    let season_spin_button: SpinButton  = builder.get_object("season_spin_button").unwrap();
    let preview_tree: TreeView          = builder.get_object("preview_tree").unwrap();
    let info_bar: gtk::InfoBar          = builder.get_object("info_bar").unwrap();
    let info_button: Button             = builder.get_object("info_close").unwrap();
    let notification_label: gtk::Label  = builder.get_object("notification_label").unwrap();
    let automatic_button: CheckButton   = builder.get_object("automatic_button").unwrap();
    let logging_button: CheckButton     = builder.get_object("logging_button").unwrap();

    // TreeView's List Store
    // Link these up to the preview_tree and then start renaming
    let preview_list = ListStore::new(&[Type::String, Type::String]);

    // A simple macro for adding a column to the preview tree.
    macro_rules! add_column {
        ($preview_tree:ident, $title:expr, $id:expr) => {{
            let column   = TreeViewColumn::new();
            let renderer = gtk::CellRendererText::new();
            column.set_title($title);
            column.set_resizable(true);
            column.pack_start(&renderer, true);
            column.add_attribute(&renderer, "text", $id);
            preview_tree.append_column(&column);
        }}
    }

    // Create and append the Before column to the preview tree
    add_column!(preview_tree, "Before", 0);
    add_column!(preview_tree, "After", 1);

    // Connect the preview_list to the preview tree
    preview_tree.set_model(Some(&preview_list));
    preview_tree.set_headers_visible(true);

    // A simple macro that is shared among all widgets that trigger the action to update the preview.
    macro_rules! gtk_preview {
        ($widget:ident) => {{
            let $widget             = $widget.clone();
            let auto                = automatic_button.clone();
            let log_changes         = logging_button.clone();
            let season_spin_button  = season_spin_button.clone();
            let episode_spin_button = episode_spin_button.clone();
            let series_entry        = series_name_entry.clone();
            let directory_entry     = series_directory_entry.clone();
            let preview_list        = preview_list.clone();
            let info_bar            = info_bar.clone();
            let notification_label  = notification_label.clone();
            let template_entry      = template_entry.clone();
            $widget.connect_clicked(move |_| {
                if let Some(directory) = directory_entry.get_text() {
                    let mut program = &mut Arguments {
                        automatic:     auto.get_active(),
                        dry_run:       true,
                        log_changes:   log_changes.get_active(),
                        verbose:       false,
                        directory:     parse_directory(&directory),
                        series_name:   series_entry.get_text().unwrap_or_default(),
                        season_number: season_spin_button.get_value_as_int() as usize,
                        episode_count: episode_spin_button.get_value_as_int() as usize,
                        pad_length:    2,
                        template:      tokenizer::tokenize_template(template_entry.get_text().unwrap().as_str())
                    };

                    if !program.directory.is_empty() {
                        program.update_preview(&preview_list, &info_bar, &notification_label);
                    }
                }
            });
        }}
    }

    // All of the widgets that implement the update preview action
    gtk_preview!(automatic_button);
    gtk_preview!(preview_button);

    { // Hide the Info Bar when the Info Bar is closed
        let info_bar = info_bar.clone();
        info_button.connect_clicked(move |_| {
            info_bar.hide();
        });
    }


    { // NOTE: Programs the Choose Directory button with a File Chooser Dialog.
        let auto                = automatic_button.clone();
        let log_changes         = logging_button.clone();
        let season_spin_button  = season_spin_button.clone();
        let episode_spin_button = episode_spin_button.clone();
        let series_entry        = series_name_entry.clone();
        let directory_entry     = series_directory_entry.clone();
        let preview_list        = preview_list.clone();
        let info_bar            = info_bar.clone();
        let notification_label  = notification_label.clone();
        let template_entry      = template_entry.clone();
        series_directory_button.connect_clicked(move |_| {
            // Open file chooser dialog to modify series_directory_entry.
            let dialog = FileChooserDialog::new (
                Some("Choose Directory"),
                Some(&Window::new(WindowType::Popup)),
                gtk::FileChooserAction::SelectFolder,
            );
            dialog.add_button("Cancel", gtk::ResponseType::Cancel as i32);
            dialog.add_button("Select", gtk::ResponseType::Ok as i32);

            if dialog.run() == gtk::ResponseType::Ok as i32 {
                dialog.get_filename().map(|path| path.to_str().map(|text| directory_entry.set_text(text)));
            }
            dialog.destroy();

            if let Some(directory) = directory_entry.get_text() {
                let mut program = &mut Arguments {
                    automatic:     auto.get_active(),
                    dry_run:       true,
                    log_changes:   log_changes.get_active(),
                    verbose:       false,
                    directory:     parse_directory(&directory),
                    series_name:   series_entry.get_text().unwrap_or_default(),
                    season_number: season_spin_button.get_value_as_int() as usize,
                    episode_count: episode_spin_button.get_value_as_int() as usize,
                    pad_length:    2,
                    template:      tokenizer::tokenize_template(template_entry.get_text().unwrap().as_str())
                };

                if !program.directory.is_empty() {
                    program.update_preview(&preview_list, &info_bar, &notification_label);
                }
            }
        });
    }

    { // NOTE: Controls what happens when rename button is pressed
        let button              = rename_button.clone();
        let auto                = automatic_button.clone();
        let log_changes         = logging_button.clone();
        let season_spin_button  = season_spin_button.clone();
        let episode_spin_button = episode_spin_button.clone();
        let series_entry        = series_name_entry.clone();
        let directory_entry     = series_directory_entry.clone();
        let preview_list        = preview_list.clone();
        let info_bar            = info_bar.clone();
        let notification_label  = notification_label.clone();
        let template_entry      = template_entry.clone();
        button.connect_clicked(move |_| {
            if let Some(directory) = directory_entry.get_text() {
                let mut program = &mut Arguments {
                    automatic:     auto.get_active(),
                    dry_run:       false,
                    log_changes:   log_changes.get_active(),
                    verbose:       false,
                    directory:     parse_directory(&directory),
                    series_name:   series_entry.get_text().unwrap_or_default(),
                    season_number: season_spin_button.get_value_as_int() as usize,
                    episode_count: episode_spin_button.get_value_as_int() as usize,
                    pad_length:    2,
                    template:      tokenizer::tokenize_template(template_entry.get_text().unwrap().as_str())
                };

                if !program.directory.is_empty() {
                    program.rename_series(&preview_list, &info_bar, &notification_label);
                }
            }
        });
    }

    window.show_all();
    info_bar.hide();


    // Quit the program when the program has been exited
    window.connect_delete_event(|_, _| {
        gtk::main_quit();
        Inhibit(false)
    });

    // Define custom actions on keypress
    window.connect_key_press_event(move |_, key| {
        if let key::Escape = key.get_keyval() { gtk::main_quit() }
        gtk::Inhibit(false)
    });

    gtk::main();

}

trait GTK3 {
    /// Update the GTK3 preview
    fn update_preview(&mut self, preview_list: &ListStore, info_bar: &gtk::InfoBar, notification_label: &gtk::Label);

    /// Grabs a list of seasons from a given directory and attempts to rename all of the episodes in each season.
    /// The GTK3 InfoBar will be updated to reflect the changes that did or did not happen.
    fn rename_series(&mut self, preview_list: &ListStore, info_bar: &gtk::InfoBar, notification_label: &gtk::Label);

    /// Attempts to rename all of the episodes in a given directory, and then updates the GTK3 preview.
    fn rename_episodes(&self, directory: &str, preview_list: &ListStore) -> Option<String>;
}

impl GTK3 for Arguments {
    fn update_preview(&mut self, preview_list: &ListStore, info_bar: &gtk::InfoBar, notification_label: &gtk::Label) {
        preview_list.clear();
        if self.automatic {
            let series = PathBuf::from(&self.directory);
            self.series_name = series.to_filename();
            match backend::get_seasons(&self.directory) {
                Ok(seasons) => {
                    for season in seasons {
                        match backend::derive_season_number(&season) {
                            Some(number) => self.season_number = number,
                            None         => continue
                        }
                        if let Some(error) = self.rename_episodes(season.as_os_str().to_str().unwrap(), preview_list) {
                            info_bar.set_message_type(gtk::MessageType::Error);
                            notification_label.set_text(&error);
                            info_bar.show();
                        }
                    }
                },
                Err(err) => {
                    info_bar.set_message_type(gtk::MessageType::Error);
                    notification_label.set_text(err);
                    info_bar.show();
                }
            }
        } else if let Some(error) = self.rename_episodes(&self.directory, preview_list) {
            info_bar.set_message_type(gtk::MessageType::Error);
            notification_label.set_text(&error);
            info_bar.show();
        }
    }

    fn rename_series(&mut self, preview_list: &ListStore, info_bar: &gtk::InfoBar, notification_label: &gtk::Label) {
        preview_list.clear();
        if self.automatic {
            let series = PathBuf::from(&self.directory);
            self.series_name = series.to_filename();
            match backend::get_seasons(&self.directory) {
                Ok(seasons) => {
                    for season in seasons {
                        match backend::derive_season_number(&season) {
                            Some(number) => self.season_number = number,
                            None         => continue
                        }
                        if let Some(error) = self.rename_episodes(season.as_os_str().to_str().unwrap(), preview_list) {
                            info_bar.set_message_type(gtk::MessageType::Error);
                            notification_label.set_text(&error);
                        } else {
                            info_bar.set_message_type(gtk::MessageType::Info);
                            notification_label.set_text("Rename Success");
                        }
                    }
                },
                Err(err) => {
                    info_bar.set_message_type(gtk::MessageType::Error);
                    notification_label.set_text(err);
                }
            }
        } else if let Some(error) = self.rename_episodes(&self.directory, preview_list) {
            info_bar.set_message_type(gtk::MessageType::Error);
            notification_label.set_text(&error);
        } else {
            info_bar.set_message_type(gtk::MessageType::Info);
            notification_label.set_text("Rename Success");
        }
        info_bar.show();
    }

    fn rename_episodes(&self, directory: &str, preview_list: &ListStore) -> Option<String> {
        match backend::get_episodes(directory) {
            Ok(episodes) => {
                match self.get_targets(directory, &episodes, self.episode_count) {
                    Ok(targets) => {
                        if self.log_changes { logging::append_time(); }
                        let mut error_occurred = false;
                        for (source, target) in episodes.iter().zip(targets) {
                            if !self.dry_run {
                                if fs::rename(&source, &target).is_err() { error_occurred = true; };
                                if self.log_changes { logging::append_change(source.as_path(), target.as_path()); }
                            }

                            // Update the preview
                            let source = source.components().last().unwrap().as_os_str().to_str().unwrap().to_string();
                            let target = target.components().last().unwrap().as_os_str().to_str().unwrap().to_string();
                            preview_list.insert_with_values(None, &[0, 1], &[&source, &target]);
                        }
                        if error_occurred { Some(String::from("Rename Failed")) } else { None }
                    },
                    Err(err) => Some(err)
                }
            },
            Err(err) => Some(String::from(err))
        }
    }
}
