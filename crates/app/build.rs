use cxx_qt_build::{CxxQtBuilder, QmlModule};

fn main() {
    CxxQtBuilder::new_qml_module(QmlModule::new("cord.core"))
        .qt_module("Quick")
        .files(["src/br.rs"])
        .build();
}
