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

fn to_lowercase<'a>(src: &str, buffer: &'a mut [u8]) -> Result<&'a str, &'static str> {
    if src.len() > buffer.len() {
        return Err("Buffer too small");
    }

    for (i, c) in src.chars().enumerate() {
        buffer[i] = if c.is_ascii_uppercase() {
            c as u8 + 32
        } else {
            c as u8
        };
    }

    core::str::from_utf8(&buffer[..src.len()]).map_err(|_| "Invalid UTF-8")
}

pub fn concatenate_filename_ext<'a>(
    filename: &str,
    ext: &str,
    buffer: &'a mut [u8],
) -> Result<&'a str, &'static str> {
    // Calculate the total length needed (filename + '.' + extension)
    let total_length = filename.len() + 1 + ext.len();

    let mut lower_filename = [0u8; 32];
    let mut lower_ext = [0u8; 32];

    let lower_filename = to_lowercase(filename, &mut lower_filename)?;
    let lower_ext = to_lowercase(ext, &mut lower_ext)?;

    // Ensure the buffer is large enough
    if total_length > buffer.len() {
        return Err("Buffer size is too small");
    }

    // Copy the filename into the buffer
    buffer[..lower_filename.len()].copy_from_slice(lower_filename.as_bytes());

    // Add the period
    buffer[lower_filename.len()] = b'.';

    // Copy the extension into the buffer
    buffer[(lower_filename.len() + 1)..total_length].copy_from_slice(lower_ext.as_bytes());

    // Convert the buffer to &str
    core::str::from_utf8(&buffer[..total_length]).map_err(|_| "Invalid UTF-8")
}
