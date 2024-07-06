use core::str::from_utf8;

// Calculates length by checking for a blank character
fn strlen(mut string: *const u8) -> usize {
    let mut count = 0;
    unsafe {
        while *string.add(count) != 0 {
            count += 1;
        }
    }
    count
}

// Converts a *const u8 pointer into a string which can be displayed
pub fn get_string_from_ptr(string_ptr: *const u8) -> &'static str {
    let len = strlen(string_ptr);
    let string_array = unsafe { core::slice::from_raw_parts(string_ptr, len) };
    from_utf8(string_array).unwrap().trim()
}

pub fn convert_utf8_to_trimmed_string(filename: &[u8]) -> &str {
    core::str::from_utf8(filename).unwrap().trim_end()
}
