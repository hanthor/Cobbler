use gtk::prelude::*;
use adw::prelude::*;
use gtk::glib;
use gtk::CompositeTemplate;
use gtk::subclass::prelude::*;
use adw::subclass::prelude::*;
use std::cell::RefCell;



mod imp {
    use super::*;

    #[derive(CompositeTemplate, Default)]
    #[template(resource = "/com/github/bootc_media_creator/ui/window.ui")]
    pub struct Window {
        #[template_child]
        pub content_stack: TemplateChild<gtk::Stack>,
        #[template_child]
        pub image_entry: TemplateChild<adw::EntryRow>,
        #[template_child]
        pub device_list_box: TemplateChild<gtk::ListBox>,
        #[template_child]
        pub flash_button: TemplateChild<gtk::Button>,
        #[template_child]
        pub progress_bar: TemplateChild<gtk::ProgressBar>,
        #[template_child]
        pub status_page: TemplateChild<adw::StatusPage>,
        #[template_child]
        pub result_box: TemplateChild<gtk::Box>,

        // Internal State
        pub selected_device: RefCell<Option<String>>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for Window {
        const NAME: &'static str = "BootcMediaCreatorWindow";
        type Type = super::Window;
        type ParentType = adw::ApplicationWindow;

        fn class_init(klass: &mut Self::Class) {
            klass.bind_template();
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for Window {
        fn constructed(&self) {
            self.parent_constructed();
            let obj = self.obj();
            obj.setup_callbacks();
        }
    }
    impl WidgetImpl for Window {}
    impl WindowImpl for Window {}
    impl ApplicationWindowImpl for Window {}
    impl AdwApplicationWindowImpl for Window {}
}

glib::wrapper! {
    pub struct Window(ObjectSubclass<imp::Window>)
        @extends gtk::Widget, gtk::Window, gtk::ApplicationWindow, adw::ApplicationWindow,
        @implements gtk::Accessible, gtk::Buildable, gtk::ConstraintTarget, gtk::Native, gtk::Root, gtk::ShortcutManager,
        gio::ActionGroup, gio::ActionMap;
}

impl Window {
    pub fn new(app: &adw::Application) -> Self {
        glib::Object::builder()
            .property("application", app)
            .build()
    }

    fn setup_callbacks(&self) {
        self.refresh_devices();

        let imp = self.imp();
        
        let window_weak = self.downgrade();
        imp.flash_button.connect_clicked(move |_| {
            if let Some(window) = window_weak.upgrade() {
                window.on_flash_clicked();
            }
        });
    }

    fn on_flash_clicked(&self) {
        let imp = self.imp();
        let image_ref = imp.image_entry.text().to_string();
        let device_opt = imp.selected_device.borrow().clone();
        
        if let Some(device_path) = device_opt {
            imp.content_stack.set_visible_child_name("progress");
            
            let window_weak = self.downgrade();
            glib::MainContext::default().spawn_local(async move {
                 if let Some(window) = window_weak.upgrade() {
                     match crate::flasher::flash(image_ref, device_path).await {
                        Ok(_) => window.show_result(true, "Image installed successfully."),
                        Err(e) => window.show_result(false, &format!("Error: {}", e)),
                     }
                 }
            });
        } else {
             println!("No device selected!");
        }
    }

    fn show_result(&self, success: bool, message: &str) {
        let imp = self.imp();
        if let Some(child) = imp.result_box.first_child() {
            if let Ok(page) = child.downcast::<adw::StatusPage>() {
                if success {
                    page.set_title("Success");
                    page.set_icon_name(Some("emblem-ok-symbolic"));
                } else {
                    page.set_title("Failed");
                    page.set_icon_name(Some("dialog-error-symbolic"));
                }
                page.set_description(Some(message));
            }
        }
        imp.content_stack.set_visible_child_name("result");
    }

    fn refresh_devices(&self) {
        let imp = self.imp();
        let list_box = imp.device_list_box.get();
        
        while let Some(child) = list_box.first_child() {
            list_box.remove(&child);
        }

        let window_weak = self.downgrade();
        glib::MainContext::default().spawn_local(async move {
             let devices_result = crate::widgets::device_list::fetch_devices_metadata().await;
             
             if let Some(window) = window_weak.upgrade() {
                 let imp = window.imp();
                 let list_box = &imp.device_list_box;

                 if let Ok(devices) = devices_result {
                    for device in devices {
                        let row = adw::ActionRow::new();
                        let display = device.display_string.clone().unwrap_or_else(|| "Unknown Device".to_string());
                        let path = display.clone();
                        let label_text = device.label.clone().unwrap_or(device.info.clone().unwrap_or("Unknown".to_string()));

                        row.set_title(&label_text);
                        row.set_subtitle(&path);
                        
                        let check = gtk::CheckButton::new();
                        check.set_valign(gtk::Align::Center);
                        
                        row.add_prefix(&check);
                        list_box.append(&row);

                        let device_path_str = path.clone();
                        
                        let window_weak_inner = window.downgrade();
                        check.connect_toggled(move |btn| {
                           if let Some(window_inner) = window_weak_inner.upgrade() {
                               if btn.is_active() {
                                   *window_inner.imp().selected_device.borrow_mut() = Some(device_path_str.clone());
                               }
                           }
                        });
                    }
                 } else {
                      println!("Failed to fetch devices");
                 }
             }
        });
    }
}
