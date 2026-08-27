// Copyright (c) 2026 DigiText. All rights reserved.
pub const C_FW: &str = "application/pdf/a";
pub const PDF: &str = "application/pdf";
pub const XML: &str = "text/xml";
pub const C_EJ: &str = "application/octet-stream";
pub const C_ML: &str = "text/plain";
pub const ZIP: &str = "application/zip";
pub fn f_1ds(r_40t: &str) -> &'static str {
    let r_40s = r_40t.to_ascii_lowercase();
    if r_40s.ends_with(".fetch.txt") {
        return C_ML;
    }
    let r_40r = r_40s.rsplit('.').next().unwrap_or("");
    match r_40r {
        "pdf" => PDF,
        "txt" => C_ML,
        "rtf" => "application/rtf",
        "docx" => "application/vnd.openxmlformats-officedocument.wordprocessingml.document",
        "doc" => "application/msword",
        "odt" => "application/vnd.oasis.opendocument.text",
        "csv" => "text/csv",
        "xlsx" => "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet",
        "xls" => "application/vnd.ms-excel",
        "ods" => "application/vnd.oasis.opendocument.spreadsheet",
        "htm" | "html" => "text/html",
        "pptx" => "application/vnd.openxmlformats-officedocument.presentationml.presentation",
        "ppt" => "application/vnd.ms-powerpoint",
        "odp" => "application/vnd.oasis.opendocument.presentation",
        "jpeg" | "jpg" => "image/jpeg",
        "gif" => "image/gif",
        "tif" | "tiff" => "image/tiff",
        "png" => "image/png",
        "mpg" | "mpeg" | "mp4" => "video/mpeg",
        "avi" => "video/x-msvideo",
        "wmv" => "video/x-ms-wmv",
        "mov" | "qt" => "video/quicktime",
        "mp3" => "audio/mpeg",
        "wma" => "audio/x-ms-wma",
        "aac" => "audio/aac",
        "xml" => XML,
        "xsd" => C_EJ,
        "zip" => ZIP,
        _ => C_EJ,
    }
}
