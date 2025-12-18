use gtk::prelude::*;
// use adw::prelude::*;

mod window;
mod flasher;
mod utils;
mod widgets {
    pub mod device_list;
}

const APP_ID: &str = "com.github.bootc_media_creator";

fn main() -> gtk::glib::ExitCode {
    env_logger::init();
    gio::resources_register_include!("bootc_media_creator.gresource").expect("Failed to register resources.");

    // Initialize GTK/Adwaita
    let app = adw::Application::builder()
        .application_id(APP_ID)
        .build();

    app.connect_activate(build_ui);

    app.run()
}

fn build_ui(app: &adw::Application) {
    let window = window::Window::new(app);
    window.present();
}
