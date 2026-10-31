mod acc;
mod accounts;
mod files;
mod media;
mod out;
mod snap;
mod bks;
mod calls;
mod feat_ctl;
mod frames;
mod img;
mod br;
mod cfg;
mod rt;

use cxx_qt_lib::{QGuiApplication, QQmlApplicationEngine, QString, QUrl};

fn main() {
    let _ = rustls::crypto::ring::default_provider().install_default();
    rt::init();
    let mut app = QGuiApplication::new();
    if let Some(mut a) = app.as_mut() {
        a.as_mut().set_application_name(&QString::from("cord"));
        a.as_mut().set_organization_name(&QString::from("cord"));
    }
    let mut eng = QQmlApplicationEngine::new();
    let p = cfg::qml_dir().join("Main.qml");
    if let Some(mut e) = eng.as_mut() {
        e.as_mut().add_import_path(&QString::from(cfg::qml_dir().to_string_lossy().as_ref()));
        e.as_mut().add_import_path(&QString::from(cfg::dir().join("mods").to_string_lossy().as_ref()));
        img::reg_img(e.as_mut());
        e.load(&QUrl::from_local_file(&QString::from(p.to_string_lossy().as_ref())));
    }
    if let Some(a) = app.as_mut() {
        a.exec();
    }
    rt::flush_store();
}
