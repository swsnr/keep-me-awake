// Copyright Sebastian Wiesner <sebastian@swsnr.de>
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

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
    use adw::subclass::prelude::*;
    use glib::subclass::InitializingObject;
    use gtk::CompositeTemplate;

    #[derive(Default, CompositeTemplate)]
    #[template(resource = "/de/swsnr/keepmeawake/ui/application-window.ui")]
    pub struct KeepMeAwakeApplicationWindow {}

    #[glib::object_subclass]
    impl ObjectSubclass for KeepMeAwakeApplicationWindow {
        const NAME: &'static str = "KeepMeAwakeApplicationWindow";

        type Type = super::KeepMeAwakeApplicationWindow;

        type ParentType = adw::ApplicationWindow;

        fn class_init(klass: &mut Self::Class) {
            klass.bind_template();
        }

        fn instance_init(obj: &InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for KeepMeAwakeApplicationWindow {}

    impl WidgetImpl for KeepMeAwakeApplicationWindow {}

    impl WindowImpl for KeepMeAwakeApplicationWindow {}

    impl ApplicationWindowImpl for KeepMeAwakeApplicationWindow {}

    impl AdwApplicationWindowImpl for KeepMeAwakeApplicationWindow {}
}
