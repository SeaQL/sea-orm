#[link(wasm_import_module = "sea-orm")]
extern "C" {
    pub fn query(str: i32, len: i32) -> i32;
}

pub fn main() {
    let str = "SELECT * FROM users";

    unsafe {
        let result = query(str.as_ptr() as i32, str.len() as i32);
        println!("result: {}", result);
    };

    println!("Done at api");
}
