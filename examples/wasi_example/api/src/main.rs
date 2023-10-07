use std::sync::Mutex;

#[link(wasm_import_module = "sea-orm")]
extern "C" {
    pub fn query(str: usize, len: usize) -> usize;
}

lazy_static::lazy_static! {
    static ref RET: Mutex<Option<Vec<u8>>> = Mutex::new(None);
}

pub fn main() {
    // Static string test

    let str = "SELECT * FROM users";

    unsafe {
        let result = query(str.as_ptr() as usize, str.len() as usize);
        println!("Count: {}", result);
    };

    // Dynamic string test
    RET.lock()
        .unwrap()
        .replace(format!("SELECT * FROM users WHERE id = {}", 42).into());

    unsafe {
        let ptr = RET.lock().unwrap().as_ref().unwrap().as_ptr() as usize;
        let len = RET.lock().unwrap().as_ref().unwrap().len();

        let result = query(ptr, len);
        println!("Count: {}", result);
    };

    println!("Done at WASI runtime");
}
