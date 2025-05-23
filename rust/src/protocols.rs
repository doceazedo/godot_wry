use std::borrow::Cow;
use std::collections::HashMap;
use std::path::PathBuf;
use godot::builtin::GString;
use godot::classes::file_access::ModeFlags;
use godot::classes::FileAccess;
use http::{Request, Response};
use http::header::CONTENT_TYPE;
use lazy_static::lazy_static;

pub fn get_res_response(
    request: Request<Vec<u8>>,
) -> Response<Cow<'static, [u8]>> {
    let root = PathBuf::from("res://");
    let path = format!("{}{}", request.uri().host().unwrap_or_default(), request.uri().path());
    let full_path = root.join(path);
    let full_path_str = GString::from(full_path.to_str().unwrap_or_default());

    if !FileAccess::file_exists(&full_path_str) {
        return http::Response::builder()
        .header(CONTENT_TYPE, "text/plain")
        .status(404)
        .body(Cow::from(format!("Could not find file at {:?}", full_path).as_bytes().to_vec()))
        .expect("Failed to build 404 response");
    }
    
    return FileAccess::open(&full_path_str, ModeFlags::READ)
        .map(|file| {
            let extension = full_path.extension().unwrap_or_default().to_str().unwrap_or_default();
            let content_type = MIME_TYPES.get(extension).unwrap_or(&"application/octet-stream").clone();

            let content_size: i64 = file.get_length().try_into().unwrap_or(0);

            let content = file.get_buffer(content_size).as_slice().to_vec();
            http::Response::builder()
                .header(CONTENT_TYPE, content_type)
                .status(200)
                .body(Cow::from(content))
                .expect("Failed to build 200 response")
        })
        .unwrap_or_else( || {
            http::Response::builder()
                .header(CONTENT_TYPE, "text/plain")
                .status(404)
                .body(Cow::from(format!("Could not find file at {:?}", full_path).as_bytes().to_vec()))
                .expect("Failed to build 404 response")
        });
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
