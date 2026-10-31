mod br;

use cxx_qt_lib::{QGuiApplication, QQmlApplicationEngine, QUrl};

fn main() {
    let mut app = QGuiApplication::new();
    let mut eng = QQmlApplicationEngine::new();
    let p = std::env::var("CORD_QML").unwrap_or_else(|_| concat!(env!("CARGO_MANIFEST_DIR"), "/qml/Main.qml").into());
    if let Some(e) = eng.as_mut() {
        e.load(&QUrl::from_local_file(&p.as_str().into()));
    }
    if let Some(a) = app.as_mut() {
        a.exec();
    }
}
