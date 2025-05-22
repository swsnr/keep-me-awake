// Copyright Sebastian Wiesner <sebastian@swsnr.de>
//
// Licensed under the EUPL
//
// See https://interoperable-europe.ec.europa.eu/collection/eupl/eupl-text-eupl-12

use glib::object::IsA;
use gtk::gio;

glib::wrapper! {
    pub struct KeepMeAwakeApplicationWindow(ObjectSubclass<imp::KeepMeAwakeApplicationWindow>)
        @extends adw::ApplicationWindow, gtk::ApplicationWindow, gtk::Window, gtk::Widget,
        @implements gio::ActionGroup, gio::ActionMap,
            gtk::Accessible, gtk::Buildable, gtk::ConstraintTarget,
            gtk::Native, gtk::Root, gtk::ShortcutManager;
}

impl KeepMeAwakeApplicationWindow {
    pub fn new(application: &impl IsA<gtk::Application>) -> Self {
        glib::Object::builder()
            .property("application", application)
            .build()
    }
}

mod imp {
    use std::cell::Cell;

    use adw::prelude::*;
    use adw::subclass::prelude::*;
    use glib::subclass::InitializingObject;
    use gtk::CompositeTemplate;

    use crate::app::Inhibit;

    #[derive(Default, CompositeTemplate, glib::Properties)]
    #[properties(wrapper_type = super::KeepMeAwakeApplicationWindow)]
    #[template(resource = "/de/swsnr/keepmeawake/ui/application-window.ui")]
    pub struct KeepMeAwakeApplicationWindow {
        #[property(get, set, builder(Inhibit::default()))]
        inhibitors: Cell<Inhibit>,
    }

    #[gtk::template_callbacks]
    impl KeepMeAwakeApplicationWindow {
        #[template_callback(function)]
        fn icon_name(inhibit: Inhibit) -> &'static str {
            match inhibit {
                Inhibit::Nothing => "cup-empty",
                Inhibit::Suspend => "cup-full",
                Inhibit::SuspendAndIdle => "cup-full-steaming",
            }
        }

        #[template_callback(function)]
        fn toggle_name(inhibit: Inhibit) -> &'static str {
            match inhibit {
                Inhibit::Nothing => "inhibit-nothing",
                Inhibit::Suspend => "inhibit-suspend",
                Inhibit::SuspendAndIdle => "inhibit-suspend-and-idle",
            }
        }

        #[template_callback]
        fn active_toggle_changed(&self, _property: &glib::Value, toggles: &adw::ToggleGroup) {
            let inhibit = match toggles.active_name().as_ref().map(glib::GString::as_str) {
                Some("inhibit-nothing") => Inhibit::Nothing,
                Some("inhibit-suspend") => Inhibit::Suspend,
                Some("inhibit-suspend-and-idle") => Inhibit::SuspendAndIdle,
                other => unreachable!("We should not reach this: {other:?}"),
            };
            if inhibit != self.inhibitors.get() {
                self.inhibitors.set(inhibit);
                self.obj().notify_inhibitors();
            }
        }
    }

    #[glib::object_subclass]
    impl ObjectSubclass for KeepMeAwakeApplicationWindow {
        const NAME: &'static str = "KeepMeAwakeApplicationWindow";

        type Type = super::KeepMeAwakeApplicationWindow;

        type ParentType = adw::ApplicationWindow;

        fn class_init(klass: &mut Self::Class) {
            klass.bind_template();
            klass.bind_template_callbacks();
        }

        fn instance_init(obj: &InitializingObject<Self>) {
            obj.init_template();
        }
    }

    #[glib::derived_properties]
    impl ObjectImpl for KeepMeAwakeApplicationWindow {}

    impl WidgetImpl for KeepMeAwakeApplicationWindow {}

    impl WindowImpl for KeepMeAwakeApplicationWindow {}

    impl ApplicationWindowImpl for KeepMeAwakeApplicationWindow {}

    impl AdwApplicationWindowImpl for KeepMeAwakeApplicationWindow {}
}
