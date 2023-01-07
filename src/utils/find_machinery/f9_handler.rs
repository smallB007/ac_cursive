use cursive::Cursive;

use super::create_find_dlg::open_find_dlg;

pub fn f9_handler(s: &mut Cursive) {
    open_find_dlg(s);
}
