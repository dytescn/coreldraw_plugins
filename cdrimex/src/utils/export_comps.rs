use crate::utils::com::{export_doc, get_app_and_doc, ComGuard};
use crate::utils::filetypes;

pub fn export_select_comps(file_src: &str, file_type_name: &str, ver: &str) {
    let _guard = ComGuard::init();
    let Some((app, doc)) = get_app_and_doc(ver) else {
        eprintln!("export_select_comps: failed to get app or doc for version: {}", ver);
        return;
    };
    let Some(exp_opt) = app.create_export_options() else {
        eprintln!("export_select_comps: create_export_options failed");
        return;
    };
    let Some(palette) = app.create_palette_options() else {
        eprintln!("export_select_comps: create_palette_options failed");
        return;
    };

    let file_type = filetypes::cdr_file_type_to_u32(file_type_name);
    if export_doc(&doc, file_src, file_type, 2, &exp_opt, &palette).is_none() {
        eprintln!("export_select_comps: export failed");
    }
}
