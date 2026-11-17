// Copyright (c) 2026 DigiText. All rights reserved.
//! Bảng hướng dẫn cho mã phát hiện của phép kiểm **từng gói**: tiêu đề lời thường + cách sửa.
//!
//! Tồn tại vì bên nhận (cổng nhà cung cấp của DO365, báo cáo PDF, màn máy trạm) đều cần CÙNG
//! một câu giải thích. Chép câu chữ sang từng nơi thì vài tháng sau ba nơi lệch nhau — đúng cái
//! bệnh "luật hai nơi" mà cả dự án này đang tránh.
//!
//! `tieu_de` viết cho người đi sửa gói, **không** nhắc tên thẻ XML; chỗ cần chính xác tới tên
//! thẻ thì nói trong `cach_sua`.
//!
//! Thêm `c.warn`/`c.err` mới mà quên thêm vào đây thì test `moi_ma_deu_co_huong_dan` nổ — nó
//! đọc chính mã nguồn `lib.rs` chứ không đọc danh sách viết tay.

/// Một dòng hướng dẫn.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct HuongDan {
    /// Một dòng bằng lời thường: phát hiện này là chuyện gì.
    pub tieu_de: &'static str,
    /// Làm gì để hết.
    pub cach_sua: &'static str,
    /// `true` = **cảnh báo này lúc nộp gói thật có thể thành LỖI**, vì nó nói "phép kiểm chưa
    /// chạy" chứ không nói "đã kiểm và đạt".
    ///
    /// Chỉ đặt `true` cho mã mà **chính bộ kiểm này**, khi có đủ byte tệp, sinh ra được lỗi mà
    /// bây giờ nó không sinh ra được. Không đặt theo phỏng đoán về chính sách nghiệm thu của
    /// bên nhận — chuyện đó bộ kiểm không biết.
    pub nop_that_thanh_loi: bool,
}

const fn hd(tieu_de: &'static str, cach_sua: &'static str) -> HuongDan {
    HuongDan { tieu_de, cach_sua, nop_that_thanh_loi: false }
}

const fn hd_nop(tieu_de: &'static str, cach_sua: &'static str) -> HuongDan {
    HuongDan { tieu_de, cach_sua, nop_that_thanh_loi: true }
}

/// Tra hướng dẫn theo mã. `None` = mã chưa có trong bảng (bên nhận cứ vẽ như cũ, đừng bịa câu).
pub fn huong_dan(ma: &str) -> Option<HuongDan> {
    let h = match ma {
        // ----- XML -----
        "XML-HONG" => hd(
            "Tệp XML không đọc được",
            "Tệp XML này sai cú pháp nên không bộ nào đọc nổi — thường do cắt dở lúc ghi, sai phần khai mã ký tự, hoặc có ký tự điều khiển lọt vào. Mở bằng trình soạn XML, sửa cho well-formed rồi đóng gói lại.",
        ),

        // ----- Cấu trúc gói -----
        "THIEU-METS-GOC" => hd(
            "Gói không có tệp METS ở ngoài cùng",
            "Mỗi gói phải có đúng một METS.xml ngay thư mục gốc — đó là mục lục của cả gói. Nếu bạn nén thêm một lớp thư mục bọc ngoài thì hãy nén lại sao cho METS.xml nằm ngay trong gói.",
        ),
        "THIEU-REPRESENTATIONS" => hd(
            "Gói không có bản đại diện nào",
            "Tạo thư mục representations/rep1/ (hoặc tên bản đại diện của bạn), đặt tệp tài liệu vào representations/rep1/data/ và METS.xml của bản đại diện ở representations/rep1/METS.xml.",
        ),
        "THIEU-METS-BAN-DAI-DIEN" => hd(
            "Bản đại diện thiếu tệp METS riêng",
            "Mỗi thư mục trong representations/ phải có METS.xml của riêng nó, liệt kê các tệp trong data/ của bản đại diện đó. Thiếu tệp này thì không biết bản đại diện gồm những gì.",
        ),
        "THIEU-SCHEMAS" => hd(
            "Gói không kèm tệp lược đồ XSD",
            "Chép các tệp .xsd mô tả EAD/PREMIS bạn dùng vào thư mục schemas/. Gói phải tự mang lược đồ để sau này đọc lại được mà không cần tra ở đâu khác.",
        ),
        "KHONG-CO-TEP-TAI-LIEU" => hd(
            "Không có tệp tài liệu nào trong gói",
            "Tệp tài liệu phải nằm ở representations/<bản đại diện>/data/. Kiểm lại xem bước đóng gói có chép tệp vào đúng chỗ đó không — gói không có data/ thì chỉ là vỏ siêu dữ liệu.",
        ),

        // ----- METS -----
        "METS-THIEU-LOAI-GOI" => hd(
            "METS không nói đây là gói nộp, gói bảo quản hay gói khai thác",
            "Điền thuộc tính csip:OAISPACKAGETYPE ở phần tử <mets> với một trong ba giá trị SIP (nộp lưu), AIP (bảo quản) hoặc DIP (khai thác).",
        ),
        "METS-THIEU-OBJID" => hd(
            "METS không có định danh gói",
            "Điền thuộc tính OBJID ở phần tử <mets>. Giá trị nên trùng tên thư mục gốc của gói để sau này đối chiếu được.",
        ),
        "METS-OBJID-KHAC-TEN-THU-MUC" => hd(
            "Định danh gói khác tên thư mục gói",
            "Sửa một trong hai cho khớp: đổi OBJID trong METS.xml, hoặc đổi tên thư mục gốc của gói. Dấu hai chấm trong OBJID được coi như dấu gạch dưới khi so.",
        ),
        "METS-KHONG-KHAI-PROFILE" => hd(
            "METS không khai theo hồ sơ chuẩn nào",
            "Điền thuộc tính PROFILE ở phần tử <mets>, trỏ tới hồ sơ E-ARK CSIP. Không khai thì bên nhận không biết gói theo quy ước nào để đọc.",
        ),
        "METS-PROFILE-KHONG-CSIP" => hd(
            "METS khai theo hồ sơ chuẩn khác, không phải E-ARK CSIP",
            "Gói đang khai PROFILE của một hồ sơ khác. Nếu bạn có đóng theo E-ARK CSIP thì sửa PROFILE cho đúng; nếu thật sự theo hồ sơ khác thì phải thống nhất lại với bên nhận trước khi giao, vì các phép kiểm sau đều đọc theo CSIP.",
        ),
        "METS-THIEU-STRUCTMAP-CSIP" => hd(
            "METS không có sơ đồ cấu trúc theo E-ARK",
            "Thêm một <structMap> có thuộc tính LABEL=\"CSIP\" mô tả cây thư mục của gói. E-ARK CSIP bắt buộc có sơ đồ này.",
        ),
        "SIP-THIEU-THOA-THUAN-NOP-LUU" => hd(
            "Gói nộp lưu không ghi mã thoả thuận nộp lưu",
            "Thêm <altRecordID TYPE=\"SUBMISSIONAGREEMENT\"> trong phần <metsHdr>, ghi mã thoả thuận/hợp đồng nộp lưu giữa bạn và bên nhận.",
        ),
        "METS-PROFILE-LECH-NHAU" => hd(
            "METS của bản đại diện khai hồ sơ chuẩn khác METS ngoài cùng",
            "Để cùng một giá trị PROFILE cho METS.xml gốc và METS.xml của mọi bản đại diện. Lệch nhau là dấu hiệu gói được ghép từ hai công cụ khác nhau.",
        ),
        "METS-MUC-THIEU-HREF" => hd(
            "METS có mục không chỉ tới tệp nào",
            "Mục này khai trong METS mà không có xlink:href nên không biết trỏ tới tệp nào. Thêm href, hoặc bỏ mục đó nếu nó là rác do công cụ sinh ra.",
        ),
        "METS-TRO-TEP-KHONG-CO" => hd(
            "METS chỉ tới tệp không có trong gói",
            "Hoặc tệp bị thiếu khi đóng gói — chép tệp vào; hoặc đường dẫn trong METS sai — sửa href. Chú ý đường dẫn trong METS là tương đối so với thư mục chứa chính tệp METS đó.",
        ),
        "METS-BAM-KHONG-SHA256" => hd(
            "METS dùng loại mã băm khác SHA-256 nên mã băm KHÔNG được đối chiếu",
            "Tính lại mã băm bằng SHA-256 và đổi CHECKSUMTYPE thành SHA-256. Quan trọng: với loại băm khác, bộ kiểm BỎ QUA việc đối chiếu — tức tệp có bị sửa sau khi đóng gói cũng không ai biết, nên đây không phải chuyện hình thức.",
        ),
        "METS-BAM-SAI" => hd(
            "Tệp đã bị thay đổi sau khi đóng gói",
            "Mã băm SHA-256 khai trong METS không khớp nội dung tệp thật. Thường do sửa tệp rồi quên tính lại mã băm, hoặc tệp hỏng khi chép. Đóng gói lại từ tệp đúng, đừng sửa tay mã băm.",
        ),
        "METS-THIEU-BAM" => hd(
            "METS không ghi mã băm cho tệp",
            "Thêm thuộc tính CHECKSUM (SHA-256) và CHECKSUMTYPE=\"SHA-256\" cho mục này. Không có mã băm thì sau này không chứng minh được tệp còn nguyên.",
        ),
        "METS-SIZE-SAI" => hd(
            "Cỡ tệp khai trong METS khác cỡ thật",
            "Sửa thuộc tính SIZE cho đúng số byte thật của tệp. Lệch cỡ gần như luôn đi kèm việc tệp bị thay sau khi sinh METS — nên đóng gói lại thay vì sửa tay con số.",
        ),
        "METS-MPTR-KHONG-CO" => hd(
            "METS ngoài cùng trỏ tới tệp METS không có",
            "Mục mptr trong sơ đồ cấu trúc trỏ tới METS.xml của một bản đại diện không tồn tại. Kiểm lại danh sách bản đại diện: hoặc thiếu thư mục, hoặc đường dẫn sai.",
        ),
        "METS-TEP-MO-COI" => hd(
            "Có tệp trong gói mà không METS nào nhắc tới",
            "Mọi tệp trong gói phải được một METS liệt kê. Xoá các tệp rác (Thumbs.db, bản nháp, tệp tạm), hoặc khai thêm vào METS nếu đó là tệp cần giữ. Danh sách tệp ở phần «Chỗ bị ảnh hưởng».",
        ),

        // ----- EAD -----
        "EAD-TRUONG-RONG" => hd(
            "Trường mã bắt buộc để trống",
            "Trường này thuộc bảng mã của TT05 nên không được để rỗng. Điền mã hai chữ số đúng bảng mã tương ứng.",
        ),
        "EAD-MA-NGOAI-BANG" => hd(
            "Giá trị không có trong bảng mã",
            "Giá trị phải là mã hai chữ số nằm trong bảng mã của TT05 cho trường đó. Tra lại bảng mã rồi điền đúng, chú ý giữ số 0 ở đầu (ví dụ 01, không phải 1).",
        ),
        "EAD-RISKRECOVERY-SAI" => hd(
            "Cờ dự phòng rủi ro không phải 0 hay 1",
            "Trường riskRecovery chỉ nhận 0 (không có bản dự phòng) hoặc 1 (có). Sửa về một trong hai giá trị đó.",
        ),
        "EAD-THIEU-HO-SO" => hd(
            "Gói không có tệp mô tả hồ sơ",
            "Tạo metadata/descriptive/EAD.xml mô tả hồ sơ (tiêu đề, thời hạn bảo quản, chế độ sử dụng, số lượng văn bản…). Gói không có tệp này thì không biết nó là hồ sơ gì.",
        ),
        "EAD-DIP-THIEU-TRUONG" => hd(
            "Gói khai thác thiếu thông tin về yêu cầu khai thác",
            "Với gói DIP, tệp EAD.xml mô tả YÊU CẦU khai thác chứ không mô tả hồ sơ: phải có mã yêu cầu (requestID), ngày yêu cầu (requestDate) và mục đích sử dụng (purpose).",
        ),
        "EAD-HO-SO-THIEU-TRUONG" => hd(
            "Phần mô tả hồ sơ thiếu trường bắt buộc",
            "Điền đủ các trường bắt buộc của mô tả hồ sơ: tiêu đề, thời hạn bảo quản, chế độ sử dụng, ngôn ngữ, số lượng văn bản, mức độ tin cậy.",
        ),
        "EAD-HO-SO-THIEU-MA" => hd(
            "Mô tả hồ sơ không có mã hồ sơ",
            "Điền arcFileCode (mã hồ sơ lưu trữ) hoặc fileCode. Mã hồ sơ là gốc để so khuôn mã của từng văn bản trong gói, thiếu nó thì cả loạt phép kiểm sau không chạy được.",
        ),
        "EAD-AIP-THIEU-ARCFILECODE" => hd(
            "Gói bảo quản chỉ có mã hồ sơ tạm, chưa có mã lưu trữ",
            "Gói AIP là gói đưa vào bảo quản lâu dài nên nên dùng arcFileCode — mã hồ sơ sau khi đã nhập kho — thay vì chỉ có fileCode của giai đoạn chỉnh lý.",
        ),
        "EAD-HO-SO-LECH-XSD" => hd(
            "Mô tả hồ sơ không khớp lược đồ đi kèm gói",
            "Phần mô tả hồ sơ thiếu trường bắt buộc hoặc có trường không khai trong lược đồ. Sửa theo đúng tệp .xsd mà chính gói mang trong schemas/, hoặc sửa lược đồ nếu lược đồ mới là cái sai.",
        ),
        "EAD-KHONG-CO-XSD" => hd(
            "Gói không có lược đồ để đối chiếu mô tả hồ sơ — phép kiểm này đã bị bỏ",
            "Chép tệp .xsd tương ứng vào schemas/. Lưu ý đây không phải «đã kiểm và đạt»: không có lược đồ thì bộ kiểm KHÔNG đối chiếu được mô tả hồ sơ, sai sót trong đó vẫn còn nguyên mà chưa ai thấy.",
        ),
        "EAD-VAN-BAN-THIEU-MA" => hd(
            "Văn bản không có mã văn bản",
            "Mỗi mô tả văn bản phải có arcDocCode (hoặc docCode). Mã này cũng là tên tệp PDF tương ứng, nên thiếu nó thì không ghép được mô tả với tệp.",
        ),
        "EAD-MA-VAN-BAN-SAI-KHUON" => hd(
            "Mã văn bản không theo khuôn mã hồ sơ cộng số thứ tự",
            "Mã văn bản phải là <mã hồ sơ>.<số thứ tự 7 chữ số>, ví dụ H49.101.01.1993.001.0000001. Giữ đủ 7 chữ số kể cả số 0 ở đầu.",
        ),
        "EAD-KHONG-CO-TEP-DU-LIEU" => hd(
            "Có mô tả văn bản nhưng không có tệp tài liệu tương ứng",
            "Tên tệp trong data/ phải trùng mã văn bản (chỉ khác phần mở rộng). Hoặc tệp bị thiếu — chép vào; hoặc tệp đặt tên khác — đổi tên tệp cho trùng mã.",
        ),
        "EAD-VAN-BAN-LECH-XSD" => hd(
            "Mô tả văn bản không khớp lược đồ đi kèm gói",
            "Mô tả văn bản thiếu trường bắt buộc hoặc có trường lạ so với schemas/EAD_doc.xsd. Sửa theo đúng lược đồ mà gói mang theo.",
        ),
        "EAD-MA-VAN-BAN-TRUNG" => hd(
            "Hai văn bản trở lên dùng cùng một mã",
            "Mỗi văn bản một mã riêng. Mã trùng thường do sao chép mô tả rồi quên sửa số thứ tự. Danh sách mã trùng ở phần «Chỗ bị ảnh hưởng».",
        ),
        "EAD-THIEU-TRICH-YEU" => hd(
            "Có văn bản chưa có trích yếu",
            "Điền trích yếu nội dung cho từng văn bản trong danh sách. Trích yếu là chỗ người tra cứu đọc để biết văn bản nói gì — bỏ trống thì hồ sơ gần như không tra được.",
        ),
        "EAD-TRICH-YEU-TRUNG" => hd(
            "Nhiều văn bản trùng trích yếu",
            "Mở từng văn bản trong danh sách, sửa trích yếu cho đúng nội dung của chính văn bản đó. Trùng nhau hàng loạt thường do chép trích yếu của văn bản đầu cho cả hồ sơ.",
        ),
        "EAD-TOAN-LOAI-KHAC" => hd(
            "Cả hồ sơ đều ghi loại văn bản là «Khác»",
            "Xác định đúng tên loại từng văn bản (quyết định, báo cáo, tờ trình, công văn…) theo bảng loại văn bản. Để tất cả là «Khác» thì mất hẳn khả năng lọc theo loại.",
        ),
        "EAD-RISKRECOVERY-DU-TRANG-THAI" => hd(
            "Khai không có bản dự phòng nhưng vẫn điền tình trạng dự phòng",
            "Hai trường này phải nhất quán: nếu riskRecovery=0 thì để trống riskRecoveryStatus; nếu thật sự có bản dự phòng thì sửa riskRecovery thành 1.",
        ),
        "EAD-NGAY-SAI-DINH-DANG" => hd(
            "Ngày không viết theo dạng ngày/tháng/năm",
            "Ngày phải viết đủ dd/mm/yyyy, ví dụ 09/07/2012. Chỉ ghi năm, hay viết theo dạng yyyy-mm-dd, đều không nhận. Danh sách chỗ sai ở phần «Chỗ bị ảnh hưởng».",
        ),
        "EAD-TOTALDOC-LECH" => hd(
            "Số văn bản khai khác số thật trong gói",
            "Số khai ở totalDoc phải bằng số tệp trong data/ và bằng số mô tả văn bản. Lệch nhau nghĩa là gói thiếu tệp, thiếu mô tả, hoặc con số khai sai — đếm lại rồi đóng gói lại.",
        ),
        "EAD-SO-TRANG-LECH" => hd(
            "Số trang của hồ sơ khác tổng số trang các văn bản",
            "Sửa số trang hồ sơ cho bằng tổng số trang các văn bản, hoặc sửa số trang từng văn bản nếu con số đó mới là cái sai. Thường do đếm tay ở mức hồ sơ.",
        ),

        // ----- PREMIS -----
        "PREMIS-AIP-THIEU-GOC" => hd(
            "Gói bảo quản không có tệp lịch sử bảo quản",
            "Gói AIP phải có metadata/preservation/PREMIS.xml ghi các sự kiện đã tác động lên tài liệu (quét, chuyển dạng, ký số, nhập kho). Đây là chỗ chứng minh nguồn gốc về sau.",
        ),
        "PREMIS-AIP-THIEU-BAN-DAI-DIEN" => hd(
            "Bản đại diện của gói bảo quản thiếu tệp lịch sử bảo quản",
            "Mỗi bản đại diện trong gói AIP phải có tệp PREMIS riêng ở representations/<tên>/metadata/preservation/PREMIS_<tên>.xml. Chú ý tên tệp phải mang đúng tên bản đại diện.",
        ),
        "PREMIS-THIEU-BAM-DUNG" => hd(
            "Lịch sử bảo quản không có mã băm đúng cho tệp",
            "Mỗi tệp trong data/ phải có một mục trong PREMIS với mã băm SHA-256 khớp nội dung tệp thật. Thiếu hoặc sai mã băm nghĩa là tệp không chứng minh được còn nguyên.",
        ),
        "PREMIS-THIEU-ORIGINALNAME" => hd(
            "Lịch sử bảo quản không ghi tên tệp gốc",
            "Thêm originalName cho từng mục trong PREMIS, ghi đúng tên tệp. Thiếu trường này thì phải dò theo mã băm mới biết mục nào ứng với tệp nào — máy ghép tự động sẽ không ghép được.",
        ),
        "PREMIS-FORMATNAME-CHUNG" => hd(
            "Lịch sử bảo quản chỉ ghi định dạng chung là PDF",
            "Ghi rõ định dạng bảo quản thay vì chỉ «Portable Document Format»: nên ghi PDF/A-2b kèm mã PRONOM fmt/477. Ghi chung chung thì sau này không biết tệp có đúng là PDF/A hay không.",
        ),

        // ----- PDF -----
        "PDF-KHONG-PDFA" => hd(
            "Có tệp không phải PDF/A",
            "TT05 Điều 8 khoản 2 điểm a đòi PDF/A hai lớp. Tệp phải tự khai là PDF/A và có hồ sơ màu đi kèm. Sửa ở bước ghép PDF/A rồi làm lại cả lô — sai cấu hình công cụ thì cả lô sai cùng kiểu.",
        ),
        "PDF-THIEU-LOP-CHU" => hd(
            "Có tệp thiếu lớp chữ tìm kiếm được",
            "PDF/A hai lớp gồm ảnh quét và lớp chữ nhận dạng nằm dưới. Chạy lại OCR cho các tệp trong danh sách rồi ghép lại — thiếu lớp chữ thì hồ sơ không tìm được bằng từ khoá.",
        ),
        "PDF-DPI-THAP" => hd(
            "Có tệp quét ở độ phân giải thấp hơn quy định",
            "Quét lại các tệp trong danh sách ở độ phân giải TT05 yêu cầu (bản đồ, bản vẽ đòi cao hơn văn bản thường). Không nâng dpi bằng cách phóng ảnh — bộ kiểm đọc dpi thật của ảnh trong tệp.",
        ),

        // ----- Chữ ký số -----
        "KY-KHONG-TOAN-VEN" => hd(
            "Có tệp mang chữ ký số không còn toàn vẹn",
            "Chữ ký không xác minh được, hoặc tệp đã bị sửa sau khi ký. Ký lại các tệp trong danh sách từ bản đúng. Đừng sửa tệp đã ký — mọi thay đổi sau khi ký đều làm chữ ký mất hiệu lực.",
        ),
        "KY-CHUA-KY" => hd(
            "Có tệp chưa ký số",
            "Ký số các tệp trong danh sách bằng chứng thư của cơ quan/tổ chức. Tệp chưa ký thì không có gì bảo đảm nguồn gốc và tính nguyên vẹn.",
        ),
        "KY-THIEU-CHUNG-THU-ESS" => hd(
            "Chữ ký không gắn chứng thư người ký",
            "Chữ ký thiếu phần gắn chứng thư (ESS signing-certificate) nên chưa đạt mức PAdES cơ bản. Đổi cấu hình công cụ ký sang PAdES (subfilter ETSI.CAdES.detached) rồi ký lại.",
        ),
        "KY-THIEU-TSA" => hd(
            "Chữ ký không có dấu thời gian",
            "Cấu hình công cụ ký trỏ tới một máy cấp dấu thời gian (TSA) rồi ký lại. Không có dấu thời gian thì sau khi chứng thư hết hạn không chứng minh được chữ ký có từ lúc chứng thư còn hiệu lực.",
        ),
        "KY-THIEU-LTV" => hd(
            "Chữ ký chưa có dữ liệu xác minh dài hạn",
            "Gói bảo quản nên ký ở mức PAdES-LT/LTA: nhúng kèm thông tin kiểm tra hiệu lực chứng thư (OCSP hoặc CRL). Bật tuỳ chọn LTV trong công cụ ký rồi ký lại.",
        ),

        // ----- Chưa kiểm -----
        "CHUA-KIEM-NOI-DUNG" => hd_nop(
            "Một số phép kiểm chưa chạy vì chưa có nội dung tệp",
            "Đây KHÔNG phải lỗi của gói, và cũng không phải «đã kiểm và đạt» — những phép kiểm cần đọc byte thật của tệp (PDF/A, lớp chữ, độ phân giải, đối chiếu mã băm) chưa ai chạy. Dùng «Tự kiểm PDF/A» cho vài tệp mẫu để biết trước; lúc nộp gói thật các phép này chạy đủ và CÓ THỂ thành lỗi.",
        ),

        // ----- Kiểm CẢ ĐỢT giao (crate::dot) -----
        // Hai mã mức `Ok` của phép kiểm đợt (`DOT-SACH`, `TONG-DOT`) cố tình KHÔNG có ở đây:
        // phép kiểm đã đạt thì không có gì để sửa.
        "DOT-RONG" => hd(
            "Chưa chọn gói nào",
            "Đợt giao không có gói nào để kiểm. Chọn các gói của đợt rồi kiểm lại.",
        ),
        "TRUNG-OBJID" => hd(
            "Hai gói trở lên cùng một định danh",
            "OBJID là định danh duy nhất của gói trong kho — hai gói cùng OBJID thì kho chỉ giữ được một, gói kia mất. Đóng gói lại các gói trong danh sách với định danh riêng cho từng gói.",
        ),
        "TRUNG-TEN" => hd(
            "Hai gói trở lên cùng tên",
            "Đổi tên để các gói không ghi đè nhau lúc chép. Lưu ý phép so **không phân biệt hoa thường**, vì trên Windows AIP-001 và aip-001 là cùng một chỗ.",
        ),
        "THIEU-OBJID" => hd(
            "Có gói không đọc được định danh — chưa đối chiếu trùng lặp được",
            "Những gói trong danh sách thiếu OBJID nên bị bỏ ra khỏi phép kiểm trùng định danh: chúng có trùng nhau hay không thì chưa ai biết. Sửa OBJID trong METS.xml của từng gói rồi kiểm lại cả đợt.",
        ),
        "NHIEU-PHONG" => hd(
            "Một đợt giao lẫn nhiều mã phông",
            "Thường là chọn nhầm thư mục — danh sách chỉ đúng mấy gói mang mã phông thiểu số, xem chúng trước. Nếu đơn vị bạn giao gộp nhiều phông trong một đợt thật thì bỏ qua cảnh báo này.",
        ),
        "GOI-KHONG-DAT" => hd(
            "Có gói còn lỗi cấu trúc",
            "Mở từng gói trong danh sách bằng «Tự kiểm gói tin», sửa hết lỗi rồi kiểm lại cả đợt. Đừng giao đợt khi còn gói chưa đạt — bên nhận trả cả đợt, không trả riêng gói.",
        ),

        _ => return None,
    };
    Some(h)
}
