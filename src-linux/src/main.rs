//! The Linux stand-in "game". Discord sees this process (copied under the
//! game's own file name) and shows the game as being played. It only shows a
//! small window and stays open until closed.
//!
//! Arguments:  --title "Game name"   --icon /path/icon.bmp   --tray (no window)
//!             --check (exit 0 if GTK can start, used by the startup check)

use gtk::gdk_pixbuf::Pixbuf;
use gtk::gio::ApplicationFlags;
use gtk::glib::{self, clone};
use gtk::prelude::*;
use gtk::{Application, ApplicationWindow, Button, Image, Justification, Label, Orientation};
use std::env;

struct Options {
    title: String,
    icon: Option<String>,
    start_hidden: bool,
}

fn parse_args() -> Options {
    let args: Vec<String> = env::args().collect();
    let mut options = Options {
        title: "Discord Quest Completer".to_string(),
        icon: None,
        start_hidden: false,
    };

    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--title" if i + 1 < args.len() => {
                options.title = args[i + 1].clone();
                i += 2;
            }
            "--icon" if i + 1 < args.len() => {
                options.icon = Some(args[i + 1].clone());
                i += 2;
            }
            "--tray" => {
                options.start_hidden = true;
                i += 1;
            }
            _ => i += 1,
        }
    }
    options
}

fn main() {
    // `--check`: used by the app's startup check. Succeeds only if GTK can
    // start here (libraries installed and a desktop session to show windows on).
    if env::args().any(|a| a == "--check") {
        match gtk::init() {
            Ok(()) => std::process::exit(0),
            Err(e) => {
                eprintln!("GTK could not start: {}", e);
                std::process::exit(1);
            }
        }
    }

    let options = parse_args();

    // NON_UNIQUE: every fake game must be its own process. With a shared
    // application id GTK would hand a second launch over to the first one
    // and exit straight away, so Discord would only ever see one game.
    let app = Application::builder()
        .application_id("me.markterence.discordquestcompleter.runner")
        .flags(ApplicationFlags::NON_UNIQUE)
        .build();

    app.connect_activate(move |app| build_ui(app, &options));

    // Our own flags (--title, --icon) are parsed above. Passing them on to GTK
    // makes it fail with "Unknown option", so only the program name is given.
    let program = env::args().next().unwrap_or_default();
    app.run_with_args(&[program]);
}

fn build_ui(app: &Application, options: &Options) {
    let window = ApplicationWindow::builder()
        .application(app)
        .title(options.title.as_str())
        .default_width(380)
        .default_height(320)
        .build();

    let vbox = gtk::Box::new(Orientation::Vertical, 12);
    vbox.set_margin_top(24);
    vbox.set_margin_bottom(24);
    vbox.set_margin_start(24);
    vbox.set_margin_end(24);
    vbox.set_valign(gtk::Align::Center);

    let icon = options
        .icon
        .as_deref()
        .and_then(|path| Pixbuf::from_file_at_scale(path, 96, 96, true).ok());
    if let Some(pixbuf) = &icon {
        window.set_icon(Some(pixbuf));
        vbox.pack_start(&Image::from_pixbuf(Some(pixbuf)), false, false, 0);
    }

    let title_label = Label::new(None);
    title_label.set_markup(&format!(
        "<span size='x-large' weight='bold'>{}</span>",
        glib::markup_escape_text(&options.title)
    ));
    title_label.set_line_wrap(true);
    title_label.set_justify(Justification::Center);
    vbox.pack_start(&title_label, false, false, 0);

    let desc_label = Label::new(Some(
        "Part of the Discord Badge and Quest Completer.\nClose this window to stop playing.",
    ));
    desc_label.set_justify(Justification::Center);
    desc_label.set_margin_top(8);
    vbox.pack_start(&desc_label, false, false, 0);

    let quit_button = Button::with_label("Quit");
    quit_button.set_margin_top(12);
    quit_button.set_halign(gtk::Align::Center);
    quit_button.connect_clicked(clone!(@weak window => move |_| window.close()));
    vbox.pack_start(&quit_button, false, false, 0);

    window.add(&vbox);

    if options.start_hidden {
        // With no window open GTK quits at once; hold the app so it keeps running.
        std::mem::forget(app.hold());
    } else {
        window.show_all();
    }
}
