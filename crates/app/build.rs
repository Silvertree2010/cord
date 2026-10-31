use cxx_qt_build::{CxxQtBuilder, QmlModule};

fn main() {
    let b = CxxQtBuilder::new_qml_module(QmlModule::new("cord.core"))
        .qt_module("Quick")
        .files(["src/br.rs", "src/img.rs"])
        .cpp_file("src/img.cpp");
    let b = unsafe {
        b.cc_builder(|cc| {
            cc.include(concat!(env!("CARGO_MANIFEST_DIR"), "/src"));
        })
    };
    b.build();
}
