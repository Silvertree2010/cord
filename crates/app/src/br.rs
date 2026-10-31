#[cxx_qt::bridge]
pub mod q {
    unsafe extern "C++" {
        include!("cxx-qt-lib/qstring.h");
        type QString = cxx_qt_lib::QString;
    }
    unsafe extern "RustQt" {
        #[qobject]
        #[qml_element]
        #[qproperty(QString, st)]
        type Hub = super::HubR;
        #[qinvokable]
        fn ping(self: Pin<&mut Self>);
    }
}

use core::pin::Pin;
use cxx_qt_lib::QString;

#[derive(Default)]
pub struct HubR {
    st: QString,
}

impl q::Hub {
    pub fn ping(self: Pin<&mut Self>) {
        self.set_st(QString::from("pong"));
    }
}
