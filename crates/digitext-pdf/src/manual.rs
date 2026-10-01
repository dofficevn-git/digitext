// Copyright (c) 2026 DigiText. All rights reserved.
// Kết xuất "Tài liệu hướng dẫn gói tin" theo mẫu gói dự án (HMT / DOffice builder): tiêu đề xanh
// đậm, thanh mục nền xanh nhạt, bảng hai cột nhãn / giá trị có viền, bảng danh sách văn bản, chân
// trang nhỏ. Một font TTF; chữ đậm giả bằng chế độ tô + viền (Tr 2).
use super::{f_2bf, f_2eo, f_2k2, f_2rm, f_qn, f_zc, pdfa, PdfError, C_0, C_1};
use lopdf::content::{Content, Operation};
use lopdf::{dictionary, Document, Object, Stream, StringFormat};

#[derive(Debug, Clone)]
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
}
impl<'a> Layout<'a> {
    fn new_page(&mut self) {
        self.pages.push(Page::new());
        self.y = C_0 - MARGIN;
    }
    fn ensure(&mut self, h: f32) {
        if self.y - h < BOTTOM {
            self.new_page();
        }
    }
    fn wrap(&self, s: &str, size: f32, width: f32) -> Result<Vec<String>, PdfError> {
        let mut out = Vec::new();
        for para in s.split('\n') {
            out.extend(f_2rm(&self.face, para, size, width, "")?);
        }
        Ok(out)
    }
    fn width(&self, s: &str, size: f32) -> Result<f32, PdfError> {
        Ok(f_2bf(&f_2eo(&self.face, s)?, size))
    }
    fn centered(&mut self, s: &str, size: f32, bold: bool, c: (f32, f32, f32), lead: f32) -> Result<(), PdfError> {
        for line in self.wrap(s, size, C_1 - MARGIN * 2.0)? {
            self.ensure(lead);
            self.y -= lead;
            let w = self.width(&line, size)?;
            let x = ((C_1 - w) / 2.0).max(MARGIN);
            let y = self.y;
            let (face, page) = (&self.face, self.pages.last_mut().expect("có trang"));
            page.text(face, &line, size, x, y, bold, c)?;
        }
        Ok(())
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

pub fn render_manual(blocks: &[ManualBlock], font: &[u8]) -> Result<Vec<u8>, PdfError> {
    let face = ttf_parser::Face::parse(font, 0).map_err(|e| PdfError::BadFont(e.to_string()))?;
    let cw = C_1 - MARGIN * 2.0;
    let mut lay = Layout { face, pages: Vec::new(), y: 0.0 };
    lay.new_page();
    let mut all_text: Vec<String> = Vec::new();
    for b in blocks {
        match b {
            ManualBlock::Title(t) => {
                all_text.push(t.clone());
                lay.y -= 6.0;
                lay.centered(t, 15.0, true, BLUE, 20.0)?;
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
                let hdr: Vec<_> = headers.iter().enumerate().map(|(i, h)| (xs[i].0, xs[i].1, h.clone(), true, Some(CELL))).collect();
                lay.row(&hdr, BODY)?;
                for r in rows {
                    for c in r {
                        all_text.push(c.clone());
                    }
                    let cells: Vec<_> = r.iter().enumerate().map(|(i, c)| (xs[i].0, xs[i].1, c.clone(), false, None)).collect();
                    lay.row(&cells, BODY)?;
                }
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
            "MediaBox" => vec![0.into(), 0.into(), C_1.into(), C_0.into()],
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
