// Copyright (c) 2026 DigiText. All rights reserved.
// Kết xuất "Tài liệu hướng dẫn gói tin" theo mẫu gói dự án (HMT / DOffice builder): tiêu đề xanh
// đậm, thanh mục nền xanh nhạt, bảng hai cột nhãn / giá trị có viền, bảng danh sách văn bản, chân
// trang nhỏ. Một font TTF; chữ đậm giả bằng chế độ tô + viền (Tr 2).
use super::{f_2bf, f_2eo, f_2k2, f_2rm, f_qn, f_zc, pdfa, PdfError, C_0, C_1};
use lopdf::content::{Content, Operation};
use lopdf::{dictionary, Document, Object, Stream, StringFormat};

/// JSON (desktop gửi lên lệnh Tauri `pdf_manual`, 3C 27/11/2026): `{"t": "title", "v": "…"}`, `{"t": "table", "v": {"headers": …}}`.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(tag = "t", content = "v", rename_all = "snake_case")]
pub enum ManualBlock {
    /// Tiêu đề lớn, giữa trang, đậm, màu xanh.
    Title(String),
    /// Dòng phụ dưới tiêu đề, giữa trang, đậm, xanh.
    Subtitle(String),
    /// Thanh mục nền xanh nhạt, chữ đậm.
    SectionBar(String),
    /// Bảng hai cột: nhãn (nền nhạt, đậm) / giá trị, viền đủ.
    KeyValue(Vec<(String, String)>),
    /// Bảng cấu trúc: đường dẫn (đậm nếu là thư mục, thụt theo độ sâu) — mô tả.
    PathTable(Vec<(usize, String, String)>),
    /// Bảng nhiều cột: tiêu đề cột nền nhạt, viền đủ; `widths` tính theo tỷ lệ.
    Table { headers: Vec<String>, widths: Vec<f32>, rows: Vec<Vec<String>> },
    /// Khoảng trống (điểm).
    Gap(f32),
    /// Chân trang nhỏ, giữa, xám.
    Footer(String),
    /// Đầu văn bản hành chính hai cột: trái (tên cơ quan, số), phải (quốc hiệu, địa danh ngày tháng).
    /// Dòng bắt đầu bằng `*` in đậm (dấu `*` bị bỏ).
    /// `dated`: dòng địa danh, ngày tháng (nghiêng) dưới gạch của cột phải.
    OfficialHeader { left: Vec<String>, right: Vec<String>, dated: String },
    /// Đoạn văn căn trái, xuống dòng tự động.
    Paragraph(String),
    /// Khối ký hai cột: (chức danh, ghi chú nhỏ, họ tên) — để trống tên thì chừa chỗ ký.
    Signatures { left: (String, String, String), right: (String, String, String) },
    /// Sang trang mới (bìa hồ sơ, mục lục văn bản từng hồ sơ — 27/11/2026).
    PageBreak,
    /// Dòng chữ căn giữa cỡ tuỳ ý (bìa: tên phông, "HỒ SƠ" cỡ lớn, năm).
    Centered { text: String, size: f32, bold: bool },
    /// Nhãn hộp hồ sơ, 4 nhãn một trang A4 (2 × 2), mỗi nhãn có viền để cắt (đợt 5, 27/11/2026). Luôn bắt đầu trang mới.
    Labels2x2(Vec<Label>),
}

/// Một nhãn hộp: dòng đầu (cơ quan, phòng ban — dòng đầu in đậm), số hộp cỡ lớn, các dòng cuối dạng nhãn: giá trị.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct Label {
    pub top: Vec<String>,
    pub big: String,
    pub bottom: Vec<(String, String)>,
}

const MARGIN: f32 = 50.0;
const BOTTOM: f32 = 46.0;
const BODY: f32 = 9.8;
const LEAD: f32 = 13.0;
const PAD: f32 = 4.0;
const BLUE: (f32, f32, f32) = (0.12, 0.29, 0.60);
const BAR: (f32, f32, f32) = (0.86, 0.91, 0.98);
const CELL: (f32, f32, f32) = (0.94, 0.96, 0.99);
const BORDER: (f32, f32, f32) = (0.62, 0.72, 0.87);
const GREY: (f32, f32, f32) = (0.40, 0.40, 0.40);
const BLACK: (f32, f32, f32) = (0.0, 0.0, 0.0);

struct Page {
    ops: Vec<Operation>,
}
impl Page {
    fn new() -> Self {
        Self { ops: Vec::new() }
    }
    fn rect_fill(&mut self, x: f32, y: f32, w: f32, h: f32, c: (f32, f32, f32)) {
        self.ops.push(Operation::new("rg", vec![c.0.into(), c.1.into(), c.2.into()]));
        self.ops.push(Operation::new("re", vec![x.into(), y.into(), w.into(), h.into()]));
        self.ops.push(Operation::new("f", vec![]));
    }
    fn rect_stroke(&mut self, x: f32, y: f32, w: f32, h: f32, c: (f32, f32, f32)) {
        self.ops.push(Operation::new("RG", vec![c.0.into(), c.1.into(), c.2.into()]));
        self.ops.push(Operation::new("w", vec![0.6.into()]));
        self.ops.push(Operation::new("re", vec![x.into(), y.into(), w.into(), h.into()]));
        self.ops.push(Operation::new("S", vec![]));
    }
    fn hline(&mut self, x1: f32, x2: f32, y: f32, c: (f32, f32, f32), w: f32) {
        self.ops.push(Operation::new("RG", vec![c.0.into(), c.1.into(), c.2.into()]));
        self.ops.push(Operation::new("w", vec![w.into()]));
        self.ops.push(Operation::new("m", vec![x1.into(), y.into()]));
        self.ops.push(Operation::new("l", vec![x2.into(), y.into()]));
        self.ops.push(Operation::new("S", vec![]));
    }
    fn text(&mut self, face: &ttf_parser::Face<'_>, s: &str, size: f32, x: f32, y: f32, bold: bool, c: (f32, f32, f32)) -> Result<(), PdfError> {
        if s.is_empty() {
            return Ok(());
        }
        let run = f_2eo(face, s)?;
        self.ops.push(Operation::new("BT", vec![]));
        self.ops.push(Operation::new("rg", vec![c.0.into(), c.1.into(), c.2.into()]));
        if bold {
            self.ops.push(Operation::new("RG", vec![c.0.into(), c.1.into(), c.2.into()]));
            self.ops.push(Operation::new("w", vec![(size * 0.035).into()]));
            self.ops.push(Operation::new("Tr", vec![2.into()]));
        } else {
            self.ops.push(Operation::new("Tr", vec![0.into()]));
        }
        self.ops.push(Operation::new("Tf", vec![Object::Name(b"F0".to_vec()), size.into()]));
        self.ops.push(Operation::new("Tm", vec![1.into(), 0.into(), 0.into(), 1.into(), x.into(), y.into()]));
        self.ops.push(Operation::new("Tj", vec![Object::String(f_zc(&run), StringFormat::Hexadecimal)]));
        self.ops.push(Operation::new("ET", vec![]));
        Ok(())
    }
}

struct Layout<'a> {
    face: ttf_parser::Face<'a>,
    pages: Vec<Page>,
    y: f32,
    /// Khổ trang (rộng, cao) — A4 dọc hoặc ngang (báo cáo nhiều cột, 26/11/2026).
    pw: f32,
    ph: f32,
}
impl<'a> Layout<'a> {
    fn new_page(&mut self) {
        self.pages.push(Page::new());
        self.y = self.ph - MARGIN;
    }
    fn ensure(&mut self, h: f32) {
        if self.y - h < BOTTOM {
            self.new_page();
        }
    }
    fn wrap(&self, s: &str, size: f32, width: f32) -> Result<Vec<String>, PdfError> {
        let mut out = Vec::new();
        for para in s.split('\n') {
            for line in f_2rm(&self.face, para, size, width, "")? {
                // Từ dài không có dấu cách (tên tệp, mã băm, OBJID) → cắt theo ký tự cho vừa ô.
                if self.width(&line, size)? <= width {
                    out.push(line);
                    continue;
                }
                let mut cur = String::new();
                for ch in line.chars() {
                    cur.push(ch);
                    if self.width(&cur, size)? > width && cur.chars().count() > 1 {
                        let last = cur.pop().expect("có ký tự");
                        out.push(std::mem::take(&mut cur));
                        cur.push(last);
                    }
                }
                if !cur.is_empty() {
                    out.push(cur);
                }
            }
        }
        Ok(out)
    }
    fn width(&self, s: &str, size: f32) -> Result<f32, PdfError> {
        Ok(f_2bf(&f_2eo(&self.face, s)?, size))
    }
    fn centered(&mut self, s: &str, size: f32, bold: bool, c: (f32, f32, f32), lead: f32) -> Result<(), PdfError> {
        for line in self.wrap(s, size, self.pw - MARGIN * 2.0)? {
            self.ensure(lead);
            self.y -= lead;
            let w = self.width(&line, size)?;
            let x = ((self.pw - w) / 2.0).max(MARGIN);
            let y = self.y;
            let (face, page) = (&self.face, self.pages.last_mut().expect("có trang"));
            page.text(face, &line, size, x, y, bold, c)?;
        }
        Ok(())
    }
    /// Chiều cao một hàng bảng sẽ chiếm (để biết trước có vừa trang không mà lặp lại tiêu đề cột).
    fn row_height(&self, cells: &[(f32, f32, String, bool, Option<(f32, f32, f32)>)], size: f32) -> Result<f32, PdfError> {
        let mut lines = 1usize;
        for c in cells {
            lines = lines.max(self.wrap(&c.2, size, c.1 - PAD * 2.0)?.len());
        }
        Ok(lines as f32 * LEAD + PAD * 2.0)
    }
    /// Một hàng bảng: các ô (x, w, text, bold, fill); trả chiều cao đã vẽ. Tự sang trang nếu thiếu chỗ.
    fn row(&mut self, cells: &[(f32, f32, String, bool, Option<(f32, f32, f32)>)], size: f32) -> Result<(), PdfError> {
        let wrapped: Vec<Vec<String>> = cells.iter().map(|c| self.wrap(&c.2, size, c.1 - PAD * 2.0)).collect::<Result<_, _>>()?;
        let lines = wrapped.iter().map(|w| w.len().max(1)).max().unwrap_or(1) as f32;
        let h = lines * LEAD + PAD * 2.0;
        self.ensure(h);
        let top = self.y;
        let (face, page) = (&self.face, self.pages.last_mut().expect("có trang"));
        for (i, c) in cells.iter().enumerate() {
            if let Some(f) = c.4 {
                page.rect_fill(c.0, top - h, c.1, h, f);
            }
            page.rect_stroke(c.0, top - h, c.1, h, BORDER);
            let mut yy = top - PAD - size;
            for line in &wrapped[i] {
                page.text(face, line, size, c.0 + PAD, yy, c.3, BLACK)?;
                yy -= LEAD;
            }
        }
        self.y = top - h;
        Ok(())
    }
}

/// Chuẩn hoá chữ trước khi dàn trang. Ô Excel xuống dòng (Alt+Enter) mang CR LF, có khi lẫn tab, ký tự điều
/// khiển, ký tự độ rộng 0 hoặc ký tự font không có; tờ hướng dẫn chỉ là tài liệu đọc kèm nên những thứ đó
/// không được làm hỏng cả gói (2/10/2026: 289 gói "dựng hỏng" vì CR trong trích yếu).
/// CR LF / CR → xuống dòng; tab → dấu cách; điều khiển / độ rộng 0 → bỏ; thiếu glyph → '?'.
fn clean(face: &ttf_parser::Face<'_>, s: &str, one_line: bool) -> String {
    let has = |c: char| face.glyph_index(c).is_some_and(|g| g.0 != 0);
    let mut out = String::with_capacity(s.len());
    let mut prev_cr = false;
    for c in s.chars() {
        let was_cr = std::mem::replace(&mut prev_cr, c == '\r');
        match c {
            '\r' => out.push(if one_line { ' ' } else { '\n' }),
            '\n' => {
                if !was_cr {
                    out.push(if one_line { ' ' } else { '\n' });
                }
            }
            '\t' | '\u{000B}' | '\u{000C}' => out.push(' '),
            '\u{200B}'..='\u{200F}' | '\u{2060}' | '\u{FEFF}' | '\u{00AD}' => {}
            c if c.is_control() => {}
            '\u{00A0}' => out.push(c),
            c if has(c) => out.push(c),
            c if c.is_whitespace() => out.push(' '),
            _ => out.push(if has('?') { '?' } else { ' ' }),
        }
    }
    out
}

fn clean_block(face: &ttf_parser::Face<'_>, b: &ManualBlock) -> ManualBlock {
    let c = |s: &String| clean(face, s, false);
    match b {
        ManualBlock::Title(t) => ManualBlock::Title(c(t)),
        ManualBlock::Subtitle(t) => ManualBlock::Subtitle(c(t)),
        ManualBlock::SectionBar(t) => ManualBlock::SectionBar(clean(face, t, true)),
        ManualBlock::KeyValue(rows) => ManualBlock::KeyValue(rows.iter().map(|(k, v)| (c(k), c(v))).collect()),
        ManualBlock::PathTable(rows) => ManualBlock::PathTable(rows.iter().map(|(d, p, desc)| (*d, clean(face, p, true), c(desc))).collect()),
        ManualBlock::Table { headers, widths, rows } => ManualBlock::Table {
            headers: headers.iter().map(c).collect(),
            widths: widths.clone(),
            rows: rows.iter().map(|r| r.iter().map(c).collect()).collect(),
        },
        ManualBlock::Gap(g) => ManualBlock::Gap(*g),
        ManualBlock::Footer(t) => ManualBlock::Footer(c(t)),
        ManualBlock::OfficialHeader { left, right, dated } => ManualBlock::OfficialHeader {
            left: left.iter().map(|s| clean(face, s, true)).collect(),
            right: right.iter().map(|s| clean(face, s, true)).collect(),
            dated: clean(face, dated, true),
        },
        ManualBlock::Paragraph(t) => ManualBlock::Paragraph(c(t)),
        ManualBlock::Signatures { left, right } => {
            let f = |t: &(String, String, String)| (clean(face, &t.0, true), clean(face, &t.1, true), clean(face, &t.2, true));
            ManualBlock::Signatures { left: f(left), right: f(right) }
        }
        ManualBlock::PageBreak => ManualBlock::PageBreak,
        ManualBlock::Centered { text, size, bold } => ManualBlock::Centered { text: clean(face, text, *bold), size: *size, bold: *bold },
        ManualBlock::Labels2x2(nhan) => ManualBlock::Labels2x2(
            nhan.iter()
                .map(|n| Label { top: n.top.iter().map(|t| clean(face, t, true)).collect(), big: clean(face, &n.big, true), bottom: n.bottom.iter().map(|(k, v)| (clean(face, k, true), clean(face, v, true))).collect() })
                .collect(),
        ),
    }
}

/// A4 dọc.
pub fn render_manual(blocks: &[ManualBlock], font: &[u8]) -> Result<Vec<u8>, PdfError> {
    render_manual_kho(blocks, font, false)
}
/// A4 ngang — báo cáo thống kê nhiều cột, sổ.
pub fn render_manual_ngang(blocks: &[ManualBlock], font: &[u8]) -> Result<Vec<u8>, PdfError> {
    render_manual_kho(blocks, font, true)
}
fn render_manual_kho(blocks: &[ManualBlock], font: &[u8], ngang: bool) -> Result<Vec<u8>, PdfError> {
    let face = ttf_parser::Face::parse(font, 0).map_err(|e| PdfError::BadFont(e.to_string()))?;
    let cleaned: Vec<ManualBlock> = blocks.iter().map(|b| clean_block(&face, b)).collect();
    let blocks = &cleaned;
    let (pw, ph) = if ngang { (C_0, C_1) } else { (C_1, C_0) };
    let cw = pw - MARGIN * 2.0;
    let mut lay = Layout { face, pages: Vec::new(), y: 0.0, pw, ph };
    lay.new_page();
    let mut all_text: Vec<String> = Vec::new();
    for b in blocks {
        match b {
            ManualBlock::Title(t) => {
                all_text.push(t.clone());
                lay.y -= 6.0;
                lay.centered(t, 15.0, true, BLUE, 20.0)?;
                // Khoảng thở dưới tiêu đề: bảng/khối ngay sau không đè lên chữ (báo cáo không có Subtitle/SectionBar).
                lay.y -= 6.0;
            }
            ManualBlock::Subtitle(t) => {
                all_text.push(t.clone());
                lay.centered(t, 10.5, true, BLUE, 15.0)?;
            }
            ManualBlock::Gap(g) => lay.y -= g,
            ManualBlock::SectionBar(t) => {
                all_text.push(t.clone());
                lay.ensure(34.0);
                lay.y -= 10.0;
                let top = lay.y;
                {
                    let (face, page) = (&lay.face, lay.pages.last_mut().expect("có trang"));
                    page.rect_fill(MARGIN, top - 20.0, cw, 20.0, BAR);
                    page.hline(MARGIN, MARGIN + cw, top - 20.0, BLUE, 1.2);
                    page.text(face, t, 11.0, MARGIN + 6.0, top - 14.5, true, BLUE)?;
                }
                lay.y = top - 24.0;
            }
            ManualBlock::KeyValue(rows) => {
                let lw = 160.0;
                for (k, v) in rows {
                    all_text.push(k.clone());
                    all_text.push(v.clone());
                    lay.row(&[(MARGIN, lw, k.clone(), true, Some(CELL)), (MARGIN + lw, cw - lw, v.clone(), false, None)], BODY)?;
                }
            }
            ManualBlock::PathTable(rows) => {
                let lw = 190.0;
                for (depth, path, desc) in rows {
                    all_text.push(path.clone());
                    all_text.push(desc.clone());
                    let indent = "\u{a0}\u{a0}\u{a0}\u{a0}".repeat(*depth);
                    let is_dir = path.ends_with('/');
                    lay.row(&[(MARGIN, lw, format!("{indent}{path}"), is_dir, None), (MARGIN + lw, cw - lw, format!("— {desc}"), false, None)], BODY)?;
                }
            }
            ManualBlock::Table { headers, widths, rows } => {
                let total: f32 = widths.iter().sum();
                let xs: Vec<(f32, f32)> = {
                    let mut x = MARGIN;
                    widths.iter().map(|w| { let ww = cw * w / total; let r = (x, ww); x += ww; r }).collect()
                };
                for h in headers {
                    all_text.push(h.clone());
                }
                let hdr: Vec<_> = headers.iter().enumerate().take(xs.len()).map(|(i, h)| (xs[i].0, xs[i].1, h.clone(), true, Some(CELL))).collect();
                lay.row(&hdr, BODY)?;
                for r in rows {
                    for c in r {
                        all_text.push(c.clone());
                    }
                    let cells: Vec<_> = r.iter().enumerate().take(xs.len()).map(|(i, c)| (xs[i].0, xs[i].1, c.clone(), false, None)).collect();
                    // Hàng không vừa trang → sang trang và in lại tiêu đề cột, kẻo trang sau là bảng không đầu.
                    if lay.y - lay.row_height(&cells, BODY)? < BOTTOM && !hdr.is_empty() {
                        lay.new_page();
                        lay.row(&hdr, BODY)?;
                    }
                    lay.row(&cells, BODY)?;
                }
            }
            ManualBlock::OfficialHeader { left, right, dated } => {
                all_text.extend(left.iter().chain(right).cloned());
                all_text.push(dated.clone());
                let col = cw / 2.0;
                let top = lay.y;
                let mut lowest = top;
                for (lines, x0) in [(left, MARGIN), (right, MARGIN + col)] {
                    let mut y = top;
                    for l in lines {
                        let (bold, s) = match l.strip_prefix('*') {
                            Some(s) => (true, s),
                            None => (false, l.as_str()),
                        };
                        let size = if bold { 10.5 } else { 10.0 };
                        for w in lay.wrap(s, size, col - 6.0)? {
                            y -= 14.0;
                            let tw = lay.width(&w, size)?;
                            let x = x0 + ((col - tw) / 2.0).max(0.0);
                            let (face, page) = (&lay.face, lay.pages.last_mut().expect("có trang"));
                            page.text(face, &w, size, x, y, bold, BLACK)?;
                        }
                    }
                    // Gạch ngắn dưới dòng cuối mỗi cột, kiểu văn bản hành chính.
                    if !lines.is_empty() {
                        let (face_w, page) = (60.0, lay.pages.last_mut().expect("có trang"));
                        page.hline(x0 + (col - face_w) / 2.0, x0 + (col + face_w) / 2.0, y - 6.0, BLACK, 0.5);
                    }
                    lowest = lowest.min(y - 6.0);
                    if x0 > MARGIN && !dated.is_empty() {
                        let y2 = y - 22.0;
                        let tw = lay.width(dated, 10.0)?;
                        let (face, page) = (&lay.face, lay.pages.last_mut().expect("có trang"));
                        page.text(face, dated, 10.0, x0 + ((col - tw) / 2.0).max(0.0), y2, false, BLACK)?;
                        lowest = lowest.min(y2 - 4.0);
                    }
                }
                lay.y = lowest - 10.0;
            }
            ManualBlock::Paragraph(t) => {
                all_text.push(t.clone());
                for line in lay.wrap(t, BODY + 0.5, cw)? {
                    lay.ensure(LEAD + 1.0);
                    lay.y -= LEAD + 1.0;
                    let y = lay.y;
                    let (face, page) = (&lay.face, lay.pages.last_mut().expect("có trang"));
                    page.text(face, &line, BODY + 0.5, MARGIN, y, false, BLACK)?;
                }
                lay.y -= 3.0;
            }
            ManualBlock::Signatures { left, right } => {
                // Chừa cả chỗ cho chân trang để khối ký không bị tách riêng chân trang sang trang mới.
                lay.ensure(150.0);
                let col = cw / 2.0;
                let top = lay.y - 10.0;
                for (t, x0) in [(left, MARGIN), (right, MARGIN + col)] {
                    all_text.extend([t.0.clone(), t.1.clone(), t.2.clone()]);
                    let mut y = top;
                    for (s, size, bold, gap) in [(&t.0, 10.5, true, 14.0), (&t.1, 9.0, false, 13.0)] {
                        if s.is_empty() {
                            continue;
                        }
                        y -= gap;
                        let tw = lay.width(s, size)?;
                        let (face, page) = (&lay.face, lay.pages.last_mut().expect("có trang"));
                        page.text(face, s, size, x0 + ((col - tw) / 2.0).max(0.0), y, bold, BLACK)?;
                    }
                    if !t.2.is_empty() {
                        let y2 = top - 100.0;
                        let tw = lay.width(&t.2, 10.5)?;
                        let (face, page) = (&lay.face, lay.pages.last_mut().expect("có trang"));
                        page.text(face, &t.2, 10.5, x0 + ((col - tw) / 2.0).max(0.0), y2, true, BLACK)?;
                    }
                }
                lay.y = top - 110.0;
            }
            ManualBlock::PageBreak => lay.new_page(),
            ManualBlock::Centered { text, size, bold } => {
                all_text.push(text.clone());
                let size = size.clamp(6.0, 72.0);
                lay.centered(text, size, *bold, BLACK, size * 1.35)?;
            }
            ManualBlock::Labels2x2(nhan) => {
                // Trang mới nếu trang hiện tại đã có gì; mỗi trang 4 ô (2 cột × 2 hàng), ô cách lề 6 pt để cắt.
                if lay.y < lay.ph - MARGIN - 0.5 {
                    lay.new_page();
                }
                let o_w = (cw - 12.0) / 2.0;
                let o_h = (lay.ph - MARGIN * 2.0 - 12.0) / 2.0;
                for (i, n) in nhan.iter().enumerate() {
                    if i > 0 && i % 4 == 0 {
                        lay.new_page();
                    }
                    let (col, row) = ((i % 4) % 2, (i % 4) / 2);
                    let x0 = MARGIN + col as f32 * (o_w + 12.0);
                    let top = lay.ph - MARGIN - row as f32 * (o_h + 12.0);
                    {
                        let page = lay.pages.last_mut().expect("có trang");
                        page.rect_stroke(x0, top - o_h, o_w, o_h, BLACK);
                    }
                    let mut y = top - 22.0;
                    let giua = |lay: &Layout<'_>, s: &str, size: f32| -> Result<f32, PdfError> { Ok(x0 + ((o_w - lay.width(s, size)?) / 2.0).max(4.0)) };
                    for (j, t) in n.top.iter().enumerate() {
                        let size = if j == 0 { 11.0 } else { 10.0 };
                        for line in lay.wrap(t, size, o_w - 16.0)? {
                            all_text.push(line.clone());
                            let x = giua(&lay, &line, size)?;
                            let (face, page) = (&lay.face, lay.pages.last_mut().expect("có trang"));
                            page.text(face, &line, size, x, y, j == 0, BLACK)?;
                            y -= size * 1.35;
                        }
                    }
                    // "HỘP SỐ" + số hộp lớn ở giữa ô.
                    let giua_o = top - o_h / 2.0;
                    let nhan_hop = "HỘP SỐ";
                    all_text.push(nhan_hop.into());
                    all_text.push(n.big.clone());
                    {
                        let x = giua(&lay, nhan_hop, 12.0)?;
                        let (face, page) = (&lay.face, lay.pages.last_mut().expect("có trang"));
                        page.text(face, nhan_hop, 12.0, x, giua_o + 22.0, true, BLACK)?;
                    }
                    let big_size = if n.big.chars().count() > 4 { 34.0 } else { 44.0 };
                    {
                        let x = giua(&lay, &n.big, big_size)?;
                        let (face, page) = (&lay.face, lay.pages.last_mut().expect("có trang"));
                        page.text(face, &n.big, big_size, x, giua_o - 22.0, true, BLACK)?;
                    }
                    // Dòng cuối từ đáy ô lên.
                    let mut yb = top - o_h + 14.0 + (n.bottom.len().saturating_sub(1)) as f32 * 14.0;
                    for (k, v) in &n.bottom {
                        let s = format!("{k}: {v}");
                        all_text.push(s.clone());
                        let (face, page) = (&lay.face, lay.pages.last_mut().expect("có trang"));
                        page.text(face, &s, 10.0, x0 + 12.0, yb, false, BLACK)?;
                        yb -= 14.0;
                    }
                }
                lay.y = BOTTOM; // trang đã đầy: khối sau tự sang trang mới
            }
            ManualBlock::Footer(t) => {
                all_text.push(t.clone());
                lay.ensure(26.0);
                lay.y -= 8.0;
                { let y = lay.y; lay.pages.last_mut().expect("có trang").hline(MARGIN, MARGIN + cw, y, BORDER, 0.6); }
                lay.y -= 4.0;
                lay.centered(t, 8.5, false, GREY, 12.0)?;
            }
        }
    }
    // Font con: mọi ký tự đã dùng.
    let mut glyphs: Vec<(char, u16, u16)> = Vec::new();
    let mut seen = std::collections::BTreeSet::new();
    for t in &all_text {
        for c in t.chars() {
            if c == '\n' {
                continue; // xuống dòng do `wrap` xử lý, không phải glyph
            }
            if seen.insert(c) {
                let run = f_2eo(&lay.face, &c.to_string())?;
                let (gid, adv) = run.glyphs[0];
                glyphs.push((c, gid, adv));
            }
        }
    }
    let mut doc = Document::with_version("1.7");
    let pages_id = doc.new_object_id();
    let font_id = f_qn(&mut doc, font, &glyphs);
    let mut kids: Vec<Object> = Vec::new();
    for p in lay.pages {
        let content = Content { operations: p.ops };
        let bytes = content.encode().map_err(|e| PdfError::Encode(e.to_string()))?;
        let cs = doc.add_object(Stream::new(dictionary! {}, bytes));
        let pid = doc.add_object(dictionary! {
            "Type" => "Page", "Parent" => pages_id,
            "MediaBox" => vec![0.into(), 0.into(), pw.into(), ph.into()],
            "Contents" => cs,
            "Resources" => dictionary! { "Font" => dictionary! { "F0" => font_id } },
        });
        kids.push(Object::Reference(pid));
    }
    let n = kids.len() as i64;
    doc.objects.insert(pages_id, Object::Dictionary(dictionary! { "Type" => "Pages", "Kids" => kids, "Count" => n }));
    let title = blocks.iter().find_map(|b| if let ManualBlock::Title(t) = b { Some(t.as_str()) } else { None });
    let meta = pdfa::f_qs(&mut doc, title);
    let cat = doc.add_object(dictionary! {
        "Type" => "Catalog", "Pages" => pages_id, "Metadata" => meta.metadata, "OutputIntents" => meta.output_intents,
    });
    doc.trailer.set("Root", cat);
    pdfa::f_2e3(&mut doc, &f_2k2(all_text.iter().map(|s| s.as_str())));
    let mut out = Vec::new();
    doc.save_to(&mut out).map_err(|e| PdfError::Save(e.to_string()))?;
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Chữ từ Excel có xuống dòng CR LF, tab, ký tự độ rộng 0 và ký tự font không có vẫn phải ra PDF.
    /// Nhãn hộp 2×2: 5 nhãn → 2 trang, chữ "HỘP SỐ" và số hộp có trong PDF.
    #[test]
    fn nhan_hop_2x2_hai_trang() {
        let Some(font) = crate::sans_font().and_then(|p| std::fs::read(p).ok()) else {
            return;
        };
        let nhan: Vec<Label> = (1..=5)
            .map(|i| Label {
                top: vec!["UBND XÃ DIỄN XÁ".into(), "Văn phòng".into(), "*****".into()],
                big: format!("{i:02}"),
                bottom: vec![("Năm".into(), "2024".into()), ("Từ hồ sơ số".into(), "01".into()), ("Đến hồ sơ số".into(), "12".into()), ("Thời hạn bảo quản".into(), "Vĩnh viễn".into())],
            })
            .collect();
        let pdf = render_manual(&[ManualBlock::Title("Nhãn hộp".into()), ManualBlock::Labels2x2(nhan)], &font).unwrap();
        let doc = lopdf::Document::load_mem(&pdf).unwrap();
        assert_eq!(doc.get_pages().len(), 3, "trang tiêu đề + 2 trang nhãn (4 + 1)");
        // Xem bằng mắt: DIGITEXT_PDF_OUT=<thư mục> cargo test -p digitext-pdf nhan_hop -- --ignored
        if let Some(dir) = std::env::var_os("DIGITEXT_PDF_OUT") {
            std::fs::write(std::path::Path::new(&dir).join("nhan-hop-2x2.pdf"), &pdf).unwrap();
        }
    }

    #[test]
    fn manual_survives_dirty_excel_text() {
        let Some(font) = crate::sans_font().and_then(|p| std::fs::read(p).ok()) else {
            return; // máy không có font hệ thống: bỏ qua
        };
        let dirty = format!("Báo cáo tổng kết{}năm 2014{}(bản chính){} {} cuối{}dòng", "\r\n", '\t', '\u{200B}', '\u{1F600}', '\r');
        let blocks = vec![
            ManualBlock::Title(dirty.clone()),
            ManualBlock::SectionBar(dirty.clone()),
            ManualBlock::KeyValue(vec![(format!("Tiêu đề{}hồ sơ", "\r\n"), dirty.clone())]),
            ManualBlock::PathTable(vec![(1, format!("data/{}", '\r'), dirty.clone())]),
            ManualBlock::Table { headers: vec!["#".into(), "Trích yếu".into()], widths: vec![1.0, 9.0], rows: vec![vec!["1".into(), dirty.clone()]] },
            ManualBlock::Footer(dirty),
        ];
        let pdf = render_manual(&blocks, &font).expect("tờ hướng dẫn phải dựng được với chữ bẩn");
        assert!(pdf.starts_with(b"%PDF-"));
    }
    /// Xuất bảng A4 NGANG dài nhiều trang để NHÌN: tiêu đề cột phải lặp lại ở đầu mỗi trang.
    ///   $env:NGANG_RA="D:\tmp\ngang.pdf"; cargo test -p digitext-pdf xuat_ngang_de_xem -- --ignored --nocapture
    #[test]
    #[ignore]
    fn xuat_ngang_de_xem() {
        let font = crate::sans_font().and_then(|p| std::fs::read(p).ok()).expect("cần font hệ thống");
        let rows: Vec<Vec<String>> = (1..=70)
            .map(|i| {
                vec![
                    i.to_string(),
                    format!("H49.132.{:02}", i % 9 + 1),
                    format!("Phông số {i} — UBND huyện"),
                    (i * 37).to_string(),
                    (i * 412).to_string(),
                    (i * 9).to_string(),
                    if i % 3 == 0 { "Vĩnh viễn".into() } else { "20 năm".into() },
                    "2015 – 2024".into(),
                ]
            })
            .collect();
        let blocks = vec![
            ManualBlock::OfficialHeader {
                left: vec!["*LƯU TRỮ LỊCH SỬ TỈNH QUẢNG NINH".into()],
                right: vec!["*CỘNG HOÀ XÃ HỘI CHỦ NGHĨA VIỆT NAM".into(), "Độc lập - Tự do - Hạnh phúc".into()],
                dated: "Quảng Ninh, ngày 26 tháng 11 năm 2026".into(),
            },
            ManualBlock::Title("BÁO CÁO TỔNG HỢP THEO PHÔNG".into()),
            ManualBlock::KeyValue(vec![("Kỳ báo cáo".into(), "Năm 2026".into())]),
            ManualBlock::Table {
                headers: ["STT", "Mã phông", "Tên phông", "Hồ sơ", "Trang", "Hộp", "THBQ", "Thời gian"].iter().map(|s| s.to_string()).collect(),
                widths: vec![1.0, 2.0, 6.0, 1.5, 1.5, 1.2, 2.0, 2.5],
                rows,
            },
            ManualBlock::Signatures { left: ("NGƯỜI LẬP".into(), "".into(), "".into()), right: ("LÃNH ĐẠO".into(), "(Ký, ghi rõ họ tên)".into(), "".into()) },
            ManualBlock::Footer("DigiText — in 26/11/2026".into()),
        ];
        let pdf = render_manual_ngang(&blocks, &font).unwrap();
        let ra = std::env::var("NGANG_RA").unwrap_or_else(|_| "ngang-xem.pdf".into());
        std::fs::write(&ra, &pdf).unwrap();
        println!("đã ghi {ra} ({} byte)", pdf.len());
    }
}

