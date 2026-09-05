import os
import shutil
import time
from datetime import datetime

def copy_images_to_static(doc_id, image_paths):
    """Copy images to static folder để serve qua web"""
    static_paths = {}
    
    try:
        for key, path in image_paths.items():
            if path and os.path.exists(path):
                file_ext = os.path.splitext(path)[1]
                static_filename = f"{doc_id}_{key}{file_ext}"
                
                # Đảm bảo thư mục static/processed_images tồn tại
                static_dir = os.path.join("static", "processed_images")
                os.makedirs(static_dir, exist_ok=True)
                
                static_path = os.path.join(static_dir, static_filename)
                
                # Copy file
                shutil.copy2(path, static_path)
                static_paths[key] = static_path
                print(f"📁 Copied {key}: {path} -> {static_path}")
            else:
                static_paths[key] = None
                
    except Exception as e:
        print(f"❌ Error copying images: {e}")
    
    return static_paths

def cleanup_temp_files(doc_id, keep_files=None):
    """Dọn dẹp các file temp không cần thiết"""
    if keep_files is None:
        keep_files = []
    
    temp_dir = os.path.join("temp_processing", f"doc_{doc_id}")
    
    if not os.path.exists(temp_dir):
        return
    
    try:
        for file in os.listdir(temp_dir):
            file_path = os.path.join(temp_dir, file)
            
            # Giữ lại các file cần thiết
            should_keep = False
            for keep_pattern in keep_files:
                if keep_pattern in file:
                    should_keep = True
                    break
            
            if not should_keep:
                os.remove(file_path)
                print(f"🗑️ Deleted temp file: {file_path}")
        
        # Xóa thư mục nếu rỗng
        if not os.listdir(temp_dir):
            os.rmdir(temp_dir)
            print(f"🗑️ Deleted empty temp dir: {temp_dir}")
            
    except Exception as e:
        print(f"⚠️ Error cleaning up temp files for doc {doc_id}: {e}")

# utils.py - Sửa hàm cleanup_after_processing (dòng 67)
def cleanup_after_processing(doc_id, results, keep_original=False):
    """Dọn dẹp sau khi xử lý xong document - XÓA TOÀN BỘ TEMP FOLDER"""
    temp_dir = os.path.join("temp_processing", f"doc_{doc_id}")
    
    if not os.path.exists(temp_dir):
        print(f"🔍 Temp dir not found (already cleaned?): {temp_dir}")
        return
    
    try:
        if keep_original:
            print(f"⚠️ Keeping temp folder for debugging: {temp_dir}")
            return
        
        # ✅ XÓA TOÀN BỘ THƯMỤC TEMP SAU KHI COPY XONG
        import shutil
        shutil.rmtree(temp_dir)
        print(f"✅ Deleted entire temp folder: {temp_dir}")
        
    except Exception as e:
        print(f"⚠️ Error cleaning up temp for doc {doc_id}: {e}")
def cleanup_old_temp_files():
    """Cleanup toàn bộ temp folder - xóa các thư mục cũ hơn 24h"""
    temp_base_dir = "temp_processing"
    if os.path.exists(temp_base_dir):
        current_time = time.time()
        
        for item in os.listdir(temp_base_dir):
            item_path = os.path.join(temp_base_dir, item)
            if os.path.isdir(item_path):
                # Kiểm tra thời gian tạo
                creation_time = os.path.getctime(item_path)
                if current_time - creation_time > 24 * 3600:  # 24 hours
                    try:
                        shutil.rmtree(item_path)
                        print(f"🗑️ Deleted old temp directory: {item_path}")
                    except Exception as e:
                        print(f"⚠️ Error deleting {item_path}: {e}")

def delete_batch_files(document_data):
    """Xóa files của batch"""
    for doc_info in document_data:
        # ✅ FIX: Kiểm tra structure của document_data
        if isinstance(doc_info, dict):
            # Nếu là dict, lấy theo key
            doc_id = doc_info.get('id')
            file_path = doc_info.get('file_path')
            image_full = doc_info.get('image_full_path')
            image_header = doc_info.get('image_header_path')
            image_content = doc_info.get('image_content_path')
        elif isinstance(doc_info, (list, tuple)) and len(doc_info) >= 5:
            # Nếu là tuple/list với ít nhất 5 elements
            doc_id, file_path, image_full, image_header, image_content = doc_info[:5]
        else:
            # ✅ HANDLE UNEXPECTED FORMAT
            print(f"⚠️ Unexpected document_data format: {type(doc_info)} - {doc_info}")
            continue
        
        # Xóa uploaded file
        if file_path and os.path.exists(file_path):
            try:
                os.remove(file_path)
                print(f"🗑️ Deleted uploaded file: {file_path}")
            except Exception as e:
                print(f"⚠️ Error deleting file {file_path}: {e}")
        
        # Xóa static images
        for path in [image_full, image_header, image_content]:
            if path and os.path.exists(path):
                try:
                    os.remove(path)
                    print(f"🗑️ Deleted static image: {path}")
                except Exception as e:
                    print(f"⚠️ Error deleting image {path}: {e}")
        
        # Xóa toàn bộ temp folder của document
        if doc_id:
            temp_dir = os.path.join("temp_processing", f"doc_{doc_id}")
            if os.path.exists(temp_dir):
                try:
                    shutil.rmtree(temp_dir)
                    print(f"🗑️ Deleted temp directory: {temp_dir}")
                except Exception as e:
                    print(f"⚠️ Error deleting temp dir {temp_dir}: {e}")
def find_original_image_path(doc_id):
    """Tìm ảnh gốc trong temp folder"""
    temp_dir = os.path.join("temp_processing", f"doc_{doc_id}")
    original_files = []
    
    print(f"🔍 Searching for original image in: {temp_dir}")
    
    # Tìm file ảnh gốc trong temp dir
    if os.path.exists(temp_dir):
        for file in os.listdir(temp_dir):
            file_path = os.path.join(temp_dir, file)
            
            # Ưu tiên tìm file full_document
            if file.startswith('full_document') and os.path.isfile(file_path):
                original_files.insert(0, file_path)  # Đưa lên đầu
                print(f"✅ Found full_document: {file_path}")
            # Backup: tìm các file khác
            elif any(ext in file.lower() for ext in ['.png', '.jpg', '.jpeg', '.pdf', '.bmp', '.tiff']):
                if os.path.isfile(file_path):
                    original_files.append(file_path)
                    print(f"📄 Found backup file: {file_path}")
        
        print(f"📁 All files found: {original_files}")
    else:
        print(f"❌ Temp directory not found: {temp_dir}")
    
    result = original_files[0] if original_files else None
    print(f"🎯 Selected original file: {result}")
    return result

def create_directories():
    """Tạo các thư mục cần thiết"""
    directories = [
        'uploads',
        'output', 
        'static/processed_images',
        'temp_processing',
        'exports'
    ]
    
    for directory in directories:
        os.makedirs(directory, exist_ok=True)
        
def validate_file_extension(filename):
    """Kiểm tra extension file có hợp lệ không"""
    supported_extensions = {'.pdf', '.png', '.jpg', '.jpeg', '.bmp', '.tiff', '.tif'}
    file_ext = os.path.splitext(filename)[1].lower()
    return file_ext in supported_extensions
def get_file_size_mb(file_path):
    """Lấy kích thước file tính bằng MB"""
    try:
        size_bytes = os.path.getsize(file_path)
        size_mb = size_bytes / (1024 * 1024)
        return round(size_mb, 2)
    except:
        return 0
def validate_upload_quota(user_id, user_role, files):
    """Kiểm tra quota upload của user (tùy chọn)"""
    # Có thể implement giới hạn upload theo role
    max_files_per_batch = {
        'admin': 10000,
        'customer': 50000,
        'cooperator': 100
    }
    
    max_allowed = max_files_per_batch.get(user_role, 50)
    
    if len(files) > max_allowed:
        return False, f"Vượt quá giới hạn {max_allowed} files cho role {user_role}"
    
    return True, ""

def create_upload_summary(batch_id, saved_files, failed_files, user_info):
    """Tạo summary cho upload session"""
    summary = {
        'batch_id': batch_id,
        'uploaded_by': user_info['full_name'],
        'uploaded_at': datetime.now().isoformat(),
        'total_attempted': saved_files + len(failed_files),
        'successful': saved_files,
        'failed': len(failed_files),
        'success_rate': (saved_files / (saved_files + len(failed_files)) * 100) if (saved_files + len(failed_files)) > 0 else 0
    }
    
    return summary
# ✅ FIX utils.py - FUNCTION delete_batch_processed_files_only

def delete_batch_processed_files_only(document_data):
    """Xóa chỉ processed files (images), GIỮ NGUYÊN original files cho external batch"""
    deleted_count = 0
    
    for doc in document_data:
        try:
            doc_id = doc['id']
            
            # ✅ CHỈ XÓA PROCESSED IMAGES - KHÔNG XÓA file_path (original)
            processed_paths = [
                doc.get('image_full_path'),     # Static processed image
                doc.get('image_header_path'),   # Static processed image  
                doc.get('image_content_path')   # Static processed image
            ]
            
            # Xóa processed images trong static folder
            for image_path in processed_paths:
                if image_path and os.path.exists(image_path):
                    # ✅ KIỂM TRA: Chỉ xóa files trong static/ hoặc temp_processing/
                    if ('static' in image_path.lower() or 
                        'temp_processing' in image_path.lower()):
                        try:
                            os.remove(image_path)
                            deleted_count += 1
                            print(f"🗑️ Đã xóa processed image: {image_path}")
                        except Exception as e:
                            print(f"⚠️ Không thể xóa processed image {image_path}: {e}")
                    else:
                        print(f"⚠️ SKIPPED: Không xóa file ngoài static/temp: {image_path}")
            
            # Xóa temp directory nếu có
            temp_dir = os.path.join("temp_processing", f"doc_{doc_id}")
            if os.path.exists(temp_dir):
                import shutil
                try:
                    shutil.rmtree(temp_dir)
                    print(f"🗑️ Đã xóa temp directory: {temp_dir}")
                except Exception as e:
                    print(f"⚠️ Không thể xóa temp directory: {e}")
            
            # ✅ QUAN TRỌNG: KHÔNG XÓA file_path (original external file)
            original_file = doc.get('file_path')
            if original_file:
                # ✅ CHỈ LOG, KHÔNG XÓA
                if '103.154.62.17' in original_file or 'UploadFile' in original_file:
                    print(f"✅ PRESERVED external file: {original_file}")
                else:
                    print(f"✅ PRESERVED local file: {original_file}")
                
        except Exception as e:
            print(f"❌ Error processing document {doc.get('id', 'unknown')}: {e}")
    
    print(f"✅ Deleted {deleted_count} processed files (PRESERVED all original external files)")
    return deleted_count

def cleanup_document_temp_all_locations(doc_id):
    """
    Xóa temp_processing của document ở TẤT CẢ locations có thể:
    - Working directory (runtime)
    - Script directory (dev mode)  
    - _internal directory (frozen EXE - nếu có)
    """
    import sys
    deleted_paths = []
    
    # 1. Working directory (quan trọng nhất khi chạy EXE)
    working_temp = os.path.join(os.getcwd(), "temp_processing", f"doc_{doc_id}")
    
    # 2. Script directory (nếu dev mode)
    script_dir = os.path.dirname(os.path.abspath(__file__))
    script_temp = os.path.join(script_dir, "temp_processing", f"doc_{doc_id}")
    
    # 3. _internal directory (nếu chạy frozen)
    internal_temp = None
    if getattr(sys, 'frozen', False) and hasattr(sys, '_MEIPASS'):
        internal_temp = os.path.join(sys._MEIPASS, "temp_processing", f"doc_{doc_id}")
    
    # Xóa tất cả locations
    all_temp_paths = [working_temp, script_temp]
    if internal_temp:
        all_temp_paths.append(internal_temp)
    
    for temp_path in all_temp_paths:
        if os.path.exists(temp_path):
            try:
                shutil.rmtree(temp_path)
                deleted_paths.append(temp_path)
                print(f"✅ Deleted temp: {temp_path}")
            except Exception as e:
                print(f"⚠️ Failed to delete {temp_path}: {e}")
    
    if deleted_paths:
        print(f"🗑️ Deleted document temp from {len(deleted_paths)} locations")
    else:
        print(f"ℹ️ No temp folder found for doc_{doc_id}")
    
    return deleted_paths