import os
from typing import Dict, List
from dotenv import load_dotenv
load_dotenv()
class DocumentTypeConfig:
    """Configuration cho từng loại tài liệu và builder prompt KIE 1 bước"""
    DEFAULT_PAGES_TO_ANALYZE: int = int(os.getenv('DEFAULT_PAGES_TO_ANALYZE', '10'))
    DEFAULT_MAX_PAGES_FOR_FULL: int = int(os.getenv('DEFAULT_MAX_PAGES_FOR_FULL', '15'))
    
    # Danh sách loại văn bản CHUẨN (01–32)
    DOCUMENT_TYPES: List[str] = [
        "Nghị quyết",          # 01
        "Quyết định",          # 02
        "Chỉ thị",             # 03
        "Quy chế",             # 04
        "Quy định",            # 05
        "Thông cáo",           # 06
        "Thông báo",           # 07
        "Hướng dẫn",           # 08
        "Chương trình",        # 09
        "Kế hoạch",            # 10
        "Phương án",           # 11
        "Đề án",               # 12
        "Dự án",               # 13
        "Báo cáo",             # 14
        "Tờ trình",            # 15
        "Giấy ủy quyền",       # 16
        "Phiếu gửi",           # 17
        "Phiếu chuyển",        # 18
        "Phiếu báo",           # 19
        "Biên bản",            # 20
        "Hợp đồng",            # 21
        "Công văn",            # 22
        "Công điện",           # 23
        "Bản ghi nhớ",         # 24
        "Bản thỏa thuận",      # 25
        "Giấy mời",            # 26
        "Giấy giới thiệu",     # 27
        "Giấy nghỉ phép",      # 28
        "Thư công",            # 29
        "Bản đồ",              # 30
        "Bản vẽ kỹ thuật",     # 31
        "Phiếu chi",           # 32,
        "Thông tri",
        "Bảng tổng hợp",
        "Ủy nhiệm chi",
        "Bản kiểm điểm",
        "Nội quy",
        "Phiếu giới thiệu",
        "Phiếu",
        "Kết luận",
        "Nghị định",
        "Thông tư",
        "Đề nghị",
        "Khác"
    ]
    # ✅ SYMBOL TO DOCUMENT TYPE MAPPING
    DOCUMENT_SYMBOL_TO_TYPE_MAP: Dict[str, List[str]] = {
        "Kết luận": ["KL", "kl", "Kl"],
        "Công văn": ["CV", "cv", "Cv"],
        "Quyết định": ["QD", "qd", "Qd", "QĐ", "qđ"],
        "Nghị quyết": ["NQ", "nq", "Nq"],
        "Thông báo": ["TB", "tb", "Tb"],
        "Chỉ thị": ["CT", "ct", "Ct"],
        "Báo cáo": ["BC", "bc", "Bc"],
        "Tờ trình": ["TTr", "ttr", "Ttr"],
        "Kế hoạch": ["KH", "kh", "Kh"],
        "Phiếu chi": ["PC", "pc", "Pc"],
        "Nghị định": ["NĐ", "nđ", "Nđ", "ND", "nd", "Nd"],
        "Thông tư": ["TT", "tt", "Tt"],
        "Giấy mời": ["GM", "gm", "Gm"],
        "Chương trình": ["CTr", "ctr", "Ctr"],
        "Đề án": ["ĐA", "đa", "Đa", "DA", "da", "Da"],
        "Hướng dẫn": ["HD", "hd", "Hd"],
        "Biên bản": ["BB", "bb", "Bb"],
        "Công điện": ["CĐ", "cđ", "Cđ", "CD", "cd", "Cd"],
        "Ủy nhiệm chi": ["UNC", "unc", "Unc", "BN", "bn", "Bn"],
        "Bảng tổng hợp": ["BTH", "bth", "Bth"],
        "Giấy giới thiệu": ["GGT", "ggt", "Ggt"],
        "Đề nghị": ["DN", "dn", "Dn"],
        "Phương án": ["PA", "pa", "Pa"],
        "Dự án": ["DA", "da", "Da"],
        "Quy định": ["QĐinh", "qđinh", "Qđinh"],
        "Quy chế": ["QC", "qc", "Qc"],
        "Thông cáo": ["TC", "tc", "Tc"],
        "Thông tri": ["TTri", "ttri", "Ttri"],
        "Nội quy": ["NQuy", "nquy", "Nquy"],
    }
    # Config số trang và prompt tóm tắt theo loại (tùy chọn, có thể mở rộng dần)
    DOCUMENT_CONFIGS: Dict[str, Dict] = {
        "Phiếu chi": {
            "pages_to_analyze": 1,
            "max_pages_for_full": 1,
            "summary_prompt": (
                "Lấy nội dung phiếu chi, thường ở dòng có chữ 'Nội dung:'. Không giải thích, không thêm bớt."
                "Trả về JSON { \"summary\": \"...\" }"
            ),
        },
         "Ủy nhiệm chi": {
            "pages_to_analyze": 1,
            "max_pages_for_full": 1,
            "summary_prompt": (
                "Lấy nội dung ủy nhiệm chi, thường là các dòng ở trong cột 'Nội dung thanh toán'. Không giải thích, không thêm bớt."
                "Trả về JSON { \"summary\": \"...\" }"
            ),
        },
       
        # Có thể bổ sung thêm các loại khác khi cần
    }
    SIGNATURE_REQUIRED_TYPES: List[str] = [
        "Bảng tổng hợp",
    ]
    DEFAULT_SUMMARY_PROMPT: str = (
        "Hãy tóm tắt ngắn gọn nội dung chính, yêu cầu/điểm nhấn, đối tượng liên quan (nếu có). "
        "Trả về JSON { \"summary\": \"...\" }"
    )

    # Gợi ý cho từng trường theo loại tài liệu (nhúng vào 1 prompt duy nhất)
    KIE_FIELD_HINTS: Dict[str, Dict] = {
       "document_type": {  # ✅ THÊM
        "base": "là một trong các loại: " + ", ".join(DOCUMENT_TYPES) + ". Loại văn bản thường thể hiện sẵn ở giữa văn bản. Nếu không chắc chắn, chọn 'Khác'. Công văn thường không có tên loại văn bản, chỉ có trích yếu nằm dưới số và ký hiệu văn bản.",
        "per_type": {
            "Công văn": "Là loại văn bản không thể hiện tên loại văn bản trong tài liệu"
        },
    },
         "document_number_and_symbol": {  # ✅ NEW UNIFIED FIELD
        "base": "là số và ký hiệu của văn bản, thường sau 'Số:' hoặc 'Văn bản số:'.",
        "per_type": {
            "Phiếu chi": "Thường dạng PC1234, PC-001/2024, hoặc 123/PC-VP",
            "Công văn": "Thường dạng 232/UBND-PTQD, 1234/CV-VP, 567/UBND-KT",
            "Quyết định": "Thường dạng 1234/QĐ-UBND, 567/QĐ-VP, 89/QĐ-TCHC",
            "Thông tư": "Thường dạng 04/1234/TT-BTC, 12/TT-BGDĐT",
            "Nghị quyết": "Thường dạng 15/NQ-CP, 123/NQ-HĐND",
            "Ủy nhiệm chi": "Thường dạng BN1234/23289_3232_3232",
        },
    },
         "handwritten_number": {
            "base": "Số được viết tay, luôn luôn nằm ở góc phía trên bên phải.",
            "per_type": {
               
            },
        },
        "issue_date": {
            "base": "Từ cụm 'ngày X tháng Y năm Z', chuẩn hóa DD/MM/YYYY.",
            "per_type": {},
        },
         "supervising_authority": {
        "base": "Cơ quan chủ quản, thường nằm phía trên cơ quan ban hành, có thể có nhiều dòng. Lấy hết tên các cơ quan chủ quản.",
        "per_type": {
           
        },
            },
        "issuing_authority": {
            "base": "Tên cơ quan/đơn vị ban hành (thường đầu trang góc trái, có thể có nhiều dòng, phải lấy hết tên các cơ quan, nếu nhiều dòng thì trả về thêm dấu cách giữa các dòng).",
            "per_type": {
                "Ủy nhiệm chi":"Thường ở phía sau chữ 'Đơn vị trả tiền:'"
            },
        },
        "summary": {
            "base": "Có thể gồm nhiều dòng. Không có thì trả về trống.",
            "per_type": {
                "per_type": {
                "Ủy nhiệm chi":"Không cần lấy trích yếu, để trống",
                "Bảng tổng hợp": "Lấy hết thông tin ở dòng có chữ 'Bảng tổng hợp', và dòng dưới nó."
            },
            },
        },
        "signer_name": {
        "base": "Họ tên người ký văn bản, thường ở cuối văn bản.",
        "per_type": {
           
        },
    },
    }
    
    @classmethod
    def get_document_types_list(cls) -> str:
        return ", ".join(cls.DOCUMENT_TYPES)

    @classmethod
    def build_unified_kie_prompt(cls) -> str:
        """Prompt KIE 1 bước: xác định loại + trích xuất các trường theo gợi ý từng loại"""
        types_list = cls.get_document_types_list()

        def field_block(field_key: str, label_vi: str) -> str:
            cfg = cls.KIE_FIELD_HINTS.get(field_key, {"base": "", "per_type": {}})
            base = cfg.get("base", "").strip()
            per_type = cfg.get("per_type", {}) or {}
            lines = [f"- Cơ bản: {base}"] if base else []
            if per_type:
                lines.append("- Gợi ý theo loại:")
                for t, hint in per_type.items():
                    lines.append(f"  • Nếu loại tài liệu là '{t}': {hint}")
            return f"{label_vi}:\n" + ("\n".join(lines) if lines else "")

        guidance = "\n\n".join(
            [
                field_block("document_number", "2) Số văn bản"),
                field_block("document_symbol", "3) Ký hiệu văn bản"),
                field_block("handwritten_number", "4) Số viết tay"),
                field_block("issue_date", "5) Ngày ban hành"),
                field_block("supervising_authority", "6) Cơ quan chủ quản"),  # ✅ THÊM
                field_block("issuing_authority", "7) Cơ quan ban hành"),      # ✅ CẬP NHẬT SỐ THỨ TỰ
                field_block("signer_name", "8) Họ tên người ký"),             # ✅ THÊM
                field_block("summary", "9) Trích yếu"),     
            ]
        )

        prompt = f"""
    Bạn là chuyên gia trích xuất thông tin từ tài liệu hành chính Việt Nam.

    Yêu cầu:
    Trích xuất các trường trong văn bản như theo hướng dẫn bên dưới.
    3) Không bịa; để giá trị rỗng nếu không thấy. Chuẩn hóa ngày dạng DD/MM/YYYY.
    4) Cố gắng trích xuất số viết tay ở góc trên bên phải.
    5) Kết quả trả về số và ký hiệu của văn bản không được chứa "Số" hoặc "Văn bản số"...
    6) Ủy nhiệm chi không cần trích yếu, để trống.
    7) Cơ quan chủ quản thường ở phía trên cơ quan ban hành.
    8) Họ tên người ký thường ở cuối văn bản, có thể kèm chức vụ.

    Hướng dẫn cho từng trường:
    {guidance}

    Trả về JSON đơn giản theo cấu trúc sau:
    {{
        "document_type": "",
        "document_number_and_symbol": "",
        "handwritten_number": "",
        "issue_date": "DD/MM/YYYY",
        "supervising_authority": "",
        "issuing_authority": "",
        "signer_name": "",
        "summary": ""
    }}
    """
        return prompt.strip()

    # API cho phần content summary
    @classmethod
    def has_specific_config(cls, document_type: str) -> bool:
        return document_type in cls.DOCUMENT_CONFIGS

    @classmethod
    def get_pages_to_analyze(cls, document_type: str) -> int:
        """✅ FIX: Trả về default nếu không có config riêng"""
        config = cls.DOCUMENT_CONFIGS.get(document_type, {})
        return config.get("pages_to_analyze", cls.DEFAULT_PAGES_TO_ANALYZE)

    @classmethod
    def get_max_pages_for_full(cls, document_type: str) -> int:
        """✅ NEW: Lấy giới hạn số trang tối đa cho ảnh full"""
        config = cls.DOCUMENT_CONFIGS.get(document_type, {})
        return config.get("max_pages_for_full", cls.DEFAULT_MAX_PAGES_FOR_FULL)

    @classmethod
    def get_summary_prompt(cls, document_type: str) -> str:
        return cls.DOCUMENT_CONFIGS.get(document_type, {}).get(
            "summary_prompt", cls.DEFAULT_SUMMARY_PROMPT
        )

    # Hỗ trợ kiểm tra/normalize loại tài liệu
    @classmethod
    def is_valid_document_type(cls, document_type: str) -> bool:
        return document_type in cls.DOCUMENT_TYPES

    @classmethod
    def get_all_document_types(cls) -> List[str]:
        return cls.DOCUMENT_TYPES[:]
    @classmethod
    def get_config_summary(cls, document_type: str) -> Dict:
        """✅ NEW: Lấy tổng hợp config cho loại tài liệu"""
        return {
            "document_type": document_type,
            "has_specific_config": cls.has_specific_config(document_type),
            "pages_to_analyze": cls.get_pages_to_analyze(document_type),
            "max_pages_for_full": cls.get_max_pages_for_full(document_type),
            "uses_default_pages_to_analyze": document_type not in cls.DOCUMENT_CONFIGS or "pages_to_analyze" not in cls.DOCUMENT_CONFIGS.get(document_type, {}),
            "uses_default_max_pages_for_full": document_type not in cls.DOCUMENT_CONFIGS or "max_pages_for_full" not in cls.DOCUMENT_CONFIGS.get(document_type, {}),
            "summary_prompt_type": "specific" if cls.has_specific_config(document_type) else "default"
        }
    @classmethod
    def get_export_prompt_for_vintern(cls) -> str:
        """Get prompt specifically formatted for Vintern fine-tuning export"""
        return cls.build_unified_kie_prompt()

    @classmethod
    def get_expected_output_fields(cls) -> List[str]:
        """Get list of fields to extract for training data"""
        return [
            "document_type",
            "document_number_and_symbol", 
            "handwritten_number",
            "issue_date",
            "supervising_authority",
            "issuing_authority",
            "signer_name",
            "summary"
        ]
    @classmethod
    def build_finetune_kie_prompt(cls) -> str:
        types_list = cls.get_document_types_list()
        prompt = f"""
        Bạn là chuyên gia trích xuất thông tin từ tài liệu hành chính Việt Nam. Danh sách các loại văn bản: {types_list}. Hãy trích xuất các thông tin sau: Loại văn bản, Số và ký hiệu văn bản, Số viết tay ở góc trên bên phải, Cơ quan chủ quản, Cơ quan ban hành, Ngày tháng năm ban hành, Trích yếu, Người ký.
        Trả về JSON theo cấu trúc sau: 
        {{
            "document_type": "",
            "document_number_and_symbol": "",
            "handwritten_number": "",
            "issue_date": "DD/MM/YYYY",
            "supervising_authority": "",
            "issuing_authority": "",
            "signer_name": "",
            "summary": ""
        }}
        """
        return prompt.strip()
    @classmethod
    def needs_signature_analysis(cls, document_type: str) -> bool:
        """Kiểm tra loại tài liệu có cần phân tích signature không"""
        return document_type in cls.SIGNATURE_REQUIRED_TYPES

    @classmethod
    def get_signature_required_types(cls) -> List[str]:
        """Lấy danh sách các loại cần signature analysis"""
        return cls.SIGNATURE_REQUIRED_TYPES[:]

    @classmethod
    def create_signature_prompt(cls) -> str:
        """Tạo prompt đơn giản cho signature analysis - chỉ lấy ngày tháng"""
        return """
        Từ ảnh signature/cuối tài liệu, hãy trích xuất ngày tháng năm ban hành/ký.
        Tìm cụm "ngày X tháng Y năm Z" hoặc định dạng ngày tương tự trong vùng cuối tài liệu.
        
        Trả về JSON:
        {
            "signature_date": "DD/MM/YYYY"
        }
        
        Nếu không tìm thấy ngày tháng, trả về signature_date là chuỗi rỗng.
        """
    @classmethod
    def detect_document_type_from_symbol(cls, document_symbol: str) -> str:
        """
        Detect document type từ symbol pattern
        Tìm kiếm trong DOCUMENT_SYMBOL_TO_TYPE_MAP
        """
        if not document_symbol or not document_symbol.strip():
            return "Khác"
        
        symbol = document_symbol.strip()
        print(f"🔍 Detecting document type from symbol: '{symbol}'")
        
        # ✅ EXACT MATCH với các pattern trong map
        for doc_type, patterns in cls.DOCUMENT_SYMBOL_TO_TYPE_MAP.items():
            for pattern in patterns:
                if symbol.upper().startswith(pattern.upper()):
                    print(f"✅ Symbol match found: '{symbol}' → '{doc_type}' (pattern: '{pattern}')")
                    return doc_type
        
        # ✅ PARTIAL MATCH - tìm pattern có trong symbol
        for doc_type, patterns in cls.DOCUMENT_SYMBOL_TO_TYPE_MAP.items():
            for pattern in patterns:
                if pattern.upper() in symbol.upper():
                    print(f"✅ Partial symbol match: '{symbol}' → '{doc_type}' (contains: '{pattern}')")
                    return doc_type
        
        print(f"⚠️ No symbol match found for: '{symbol}'")
        return "Khác"