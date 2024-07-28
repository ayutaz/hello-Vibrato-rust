use std::ffi::{CStr, CString};
use std::os::raw::c_char;
use vibrato::{Dictionary, Tokenizer};

#[no_mangle]
pub extern "C" fn tokenize(input: *const c_char, dict_path: *const c_char) -> *mut c_char {
    let input_str = unsafe { CStr::from_ptr(input).to_str().unwrap() };
    let dict_path_str = unsafe { CStr::from_ptr(dict_path).to_str().unwrap() };

    // 辞書ファイルのロード
    let reader = zstd::Decoder::new(std::fs::File::open(dict_path_str).unwrap()).unwrap();
    let dict = Dictionary::read(reader).unwrap();

    // トークナイザーの生成
    let tokenizer = Tokenizer::new(dict)
        .ignore_space(true).unwrap()
        .max_grouping_len(24);

    let mut worker = tokenizer.new_worker();
    worker.reset_sentence(input_str);
    worker.tokenize();

    let result: String = worker.token_iter()
        .filter(|t| {
            let words: Vec<&str> = t.feature().split(',').collect();
            let subwords: Vec<&str> = words[0].split('-').collect();
            subwords[0] == "名詞" || subwords[0] == "カスタム名詞"
        })
        .map(|t| format!("{}: {}", t.surface(), t.feature()))
        .collect::<Vec<String>>()
        .join("\n");

    CString::new(result).unwrap().into_raw()
}

#[no_mangle]
pub extern "C" fn free_string(s: *mut c_char) {
    unsafe {
        if s.is_null() { return }
        drop(CString::from_raw(s));
    };
}