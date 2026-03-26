// stdlib/module.ol — Module system
// use "file.ol" → reads file, compiles, executes in current scope

let __loaded_modules = [];

pub fn _use_module(_um_path) {
    // Check if already loaded
    let _um_i = 0;
    while _um_i < len(__loaded_modules) {
        if __loaded_modules[_um_i] == _um_path { return 0; };
        let _um_i = _um_i + 1;
    };
    push(__loaded_modules, _um_path);
    // Read + compile + eval
    let _um_src = __file_read(_um_path);
    if len(_um_src) == 0 { return 0; };
    repl_eval(_um_src);
    return 1;
}
