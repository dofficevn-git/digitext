import os
import fitz  # PyMuPDF
from pathlib import Path

def test_unc_pdf_split():
    """Test tách PDF từ UNC path - đơn giản thôi"""
    
    # ✅ UNC PATH - THAY ĐỔI ĐƯỜNG DẪN NÀY
    unc_path = r"//103.154.62.17/UploadFile/TTLTLSTQ/CCKL/1/01/004/1.pdf"
    
    # Hoặc test với local file trước
    
    print(f"🔧 Testing PDF split from UNC path...")
    print(f"📁 Path: {unc_path}")
    
    try:
        # Kiểm tra file có tồn tại không
        if not os.path.exists(unc_path):
            print(f"❌ File không tồn tại: {unc_path}")
            
            
        
        # Mở PDF
        print(f"📖 Đang mở PDF...")
        doc = fitz.open(unc_path)
        total_pages = len(doc)
        
        print(f"📊 PDF có {total_pages} trang")
        
        if total_pages < 2:
            print(f"⚠️ PDF chỉ có {total_pages} trang, không cần tách")
            doc.close()
            return
        
        # Tách: trang 1 = header, còn lại = content
        cut_point = 1
        print(f"✂️ Cắt tại trang: {cut_point}")
        
        # Tạo folder output
        output_dir = Path("split_output")
        output_dir.mkdir(exist_ok=True)
        
        pdf_name = Path(unc_path).stem
        
        # Tách header (trang 1)
        header_doc = fitz.open()
        header_doc.insert_pdf(doc, from_page=0, to_page=0)
        header_path = output_dir / f"{pdf_name}_header.pdf"
        header_doc.save(header_path)
        header_doc.close()
        print(f"✅ Header saved: {header_path}")
        
        # Tách content (trang 2 trở đi)
        content_doc = fitz.open()
        content_doc.insert_pdf(doc, from_page=1, to_page=total_pages-1)
        content_path = output_dir / f"{pdf_name}_content.pdf"
        content_doc.save(content_path)
        content_doc.close()
        print(f"✅ Content saved: {content_path}")
        
        doc.close()
        
        print(f"\n🎉 Tách PDF thành công!")
        print(f"   📄 Gốc: {total_pages} trang")
        print(f"   📋 Header: 1 trang")
        print(f"   📰 Content: {total_pages-1} trang")
        
    except Exception as e:
        print(f"❌ Lỗi: {e}")

if __name__ == "__main__":
    test_unc_pdf_split()