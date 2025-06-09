use std::borrow::Cow;
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use godot::builtin::{Array, GString};
use godot::classes::file_access::ModeFlags;
use godot::classes::{Control, FileAccess};
use godot::meta::ToGodot;
use godot::obj::Gd;
use http::{Request, Response};
use http::header::{ACCEPT_RANGES, CONTENT_RANGE, CONTENT_TYPE, RANGE};
use lazy_static::lazy_static;
use uuid::Uuid;
use wry::RequestAsyncResponder;

pub fn get_res_response(request: Request<Vec<u8>>) -> Response<Cow<'static, [u8]>> {
    let root = PathBuf::from("res://");
    let path = format!(
        "{}{}",
        request.uri().host().unwrap_or_default(),
        request.uri().path()
    );
    let full_path = root.join(path);
    let full_path_str = GString::from(full_path.to_str().unwrap_or_default());

    if !FileAccess::file_exists(&full_path_str) {
        return http::Response::builder()
            .header(CONTENT_TYPE, "text/plain")
            .status(404)
            .body(Cow::from(
                format!("Could not find file at {:?}", full_path)
                    .as_bytes()
                    .to_vec(),
            ))
            .expect("Failed to build 404 response");
    }

    let extension = full_path
            .extension()
            .unwrap_or_default()
            .to_str()
            .unwrap_or_default();

    let content_type = MIME_TYPES
            .get(extension)
            .unwrap_or(&"application/octet-stream");
    
    let mut content_range: Option<(u64, u64)> = None;

    // The client might request a file with Range,
    // even if we set Accept-Ranges to none, Safari does this while loading media types.
    // So, we MUST implement the Content-Range logic to serve the file correctly.
    if let Some(range) = request.headers().get(RANGE) {
        let range_str = range.to_str().expect("failed to parse Range header");

        // assuming the range header is in the format "bytes=start-end"
        let parts: Vec<&str> = range_str[6..].split('-').collect();
        let (start, end) = (
            parts[0].parse::<u64>().expect("failed to parse range start"),
            parts[1].parse::<u64>().expect("failed to parse range end")
        );

        content_range = Some((start, end));
    }

    return FileAccess::open(&full_path_str, ModeFlags::READ)
        .map(|mut file| {
            let file_size: u64 = file.get_length().try_into().expect("failed to get file size");

            return if let Some((start, end)) = content_range {
                if start >= file_size {
                    return http::Response::builder()
                        .header(CONTENT_TYPE, *content_type)
                        .header(ACCEPT_RANGES, "bytes")
                        .header(CONTENT_RANGE, format!("bytes */{}", file_size))
                        .status(416) // Range Not Satisfiable
                        .body(Cow::from(Vec::new()))
                        .expect("Failed to build 416 response");
                }

                let end = if end == 0 || end >= file_size { file_size - 1 } else { end };

                let content_size = (end - start + 1) as i64;
                file.seek(start);
                let content = file.get_buffer(content_size).as_slice().to_vec();

                http::Response::builder()
                    .header(CONTENT_TYPE, *content_type)
                    .header(ACCEPT_RANGES, "bytes")
                    .header(CONTENT_RANGE, format!("bytes {}-{}/{}", start, end, file_size))
                    .status(206)
                    .body(Cow::from(content))
                    .expect("Failed to build 206 response")
            } else {
                let content = file.get_buffer(file_size as i64).as_slice().to_vec();
                http::Response::builder()
                    .header(CONTENT_TYPE, *content_type)
                    .status(200)
                    .body(Cow::from(content))
                    .expect("Failed to build 200 response")
            }
        }).unwrap_or_else(|| {
            http::Response::builder()
                .header(CONTENT_TYPE, "text/plain")
                .status(500)
                .body(Cow::from(
                    format!("Failed to open at {:?}", full_path)
                        .as_bytes()
                        .to_vec(),
                ))
                .expect("Failed to build 404 response")
        });
}

pub fn get_ipc_response(
    mut control: Gd<Control>,
    request: Request<Vec<u8>>,
    responder: RequestAsyncResponder,
    invoke_responders: Arc<Mutex<HashMap<String, RequestAsyncResponder>>>,
) {
    // DO NOT register responder if "X-No-Responder" header is present
    let register_responder = !request.headers().contains_key("X-No-Responder");

    let mut headers_array = Array::new();
    for (k, v) in request.headers().iter() {
        let mut tuple = Array::new();
        tuple.push(&GString::from(k.as_str()).to_variant());
        tuple.push(&GString::from(v.to_str().unwrap_or_default()).to_variant());
        headers_array.push(&tuple.to_variant());
    }

    let id = if register_responder { Uuid::new_v4().to_string() } else { String::new() };

    if !id.is_empty() {
        invoke_responders
            .lock()
            .expect("Failed to lock responders")
            .insert(id.clone(), responder);
    }

    control.emit_signal("invoke_message", &[
        id.to_variant(),
        request.method().to_string().to_variant(),
        request.uri().to_string().to_variant(),
        headers_array.to_variant(),
        request.body().to_vec().to_variant(),
    ]);
}

lazy_static! {
    static ref MIME_TYPES: HashMap<&'static str, &'static str> = HashMap::from([
        // https://developer.mozilla.org/en-US/docs/Web/HTTP/Guides/MIME_types/Common_types
        ("aac", "audio/aac"),
        ("abw", "application/x-abiword"),
        ("apng", "image/apng"),
        ("arc", "application/x-freearc"),
        ("avif", "image/avif"),
        ("avi", "video/x-msvideo"),
        ("azw", "application/vnd.amazon.ebook"),
        ("bin", "application/octet-stream"),
        ("bmp", "image/bmp"),
        ("bz", "application/x-bzip"),
        ("bz2", "application/x-bzip2"),
        ("cda", "application/x-cdf"),
        ("cjs", "text/javascript"),
        ("csh", "application/x-csh"),
        ("css", "text/css"),
        ("csv", "text/csv"),
        ("doc", "application/msword"),
        ("docx", "application/vnd.openxmlformats-officedocument.wordprocessingml.document"),
        ("eot", "application/vnd.ms-fontobject"),
        ("epub", "application/epub+zip"),
        ("gz", "application/gzip"),
        ("gif", "image/gif"),
        ("html", "text/html"),
        ("htm", "text/html"),
        ("ico", "image/vnd.microsoft.icon"),
        ("ics", "text/calendar"),
        ("jar", "application/java-archive"),
        ("jpeg", "image/jpeg"),
        ("jpg", "image/jpeg"),
        ("js", "text/javascript"),
        ("json", "application/json"),
        ("jsonld", "application/ld+json"),
        ("midi", "audio/midi"),
        ("mid", "audio/midi"),
        ("mjs", "text/javascript"),
        ("mp3", "audio/mpeg"),
        ("mp4", "video/mp4"),
        ("mpeg", "video/mpeg"),
        ("mpkg", "application/vnd.apple.installer+xml"),
        ("odp", "application/vnd.oasis.opendocument.presentation"),
        ("ods", "application/vnd.oasis.opendocument.spreadsheet"),
        ("odt", "application/vnd.oasis.opendocument.text"),
        ("oga", "audio/ogg"),
        ("ogv", "video/ogg"),
        ("ogx", "application/ogg"),
        ("opus", "audio/ogg"),
        ("otf", "font/otf"),
        ("png", "image/png"),
        ("pdf", "application/pdf"),
        ("php", "application/x-httpd-php"),
        ("ppt", "application/vnd.ms-powerpoint"),
        ("pptx", "application/vnd.openxmlformats-officedocument.presentationml.presentation"),
        ("rar", "application/vnd.rar"),
        ("rtf", "application/rtf"),
        ("sh", "application/x-sh"),
        ("svg", "image/svg+xml"),
        ("tar", "application/x-tar"),
        ("tif", "image/tiff"),
        ("tiff", "image/tiff"),
        ("ts", "video/mp2t"),
        ("ttf", "font/ttf"),
        ("txt", "text/plain"),
        ("vsd", "application/vnd.visio"),
        ("wav", "audio/wav"),
        ("weba", "audio/webm"),
        ("webm", "video/webm"),
        ("webp", "image/webp"),
        ("woff", "font/woff"),
        ("woff2", "font/woff2"),
        ("xhtml", "application/xhtml+xml"),
        ("xls", "application/vnd.ms-excel"),
        ("xlsx", "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet"),
        ("xml", "application/xml"),
        ("xul", "application/vnd.mozilla.xul+xml"),
        ("zip", "application/zip"),
        ("3gp", "video/3gpp"),
        ("3g2", "video/3gpp2"),
        ("7z", "application/x-7z-compressed"),
    ]);
}
