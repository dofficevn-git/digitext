import traceback
from flask import Flask, render_template, request, jsonify, send_file, send_from_directory, session, redirect, url_for,send_from_directory,make_response
from werkzeug.utils import secure_filename
import os
import sys
# ✅ THÊM IMPORT Ở ĐẦU FILE
from openpyxl import Workbook
from openpyxl.styles import Font, Alignment, PatternFill, Border, Side
from openpyxl.utils import get_column_letter
import io
import shutil

import json
import uuid
from datetime import datetime
from threading import Thread
from auth import login_required, role_required, admin_required, get_current_user, can_access_batch
from flask_cors import CORS
from dotenv import load_dotenv  # ✅ THÊM IMPORT

# ✅ LOAD ENVIRONMENT VARIABLES
load_dotenv()

from database import DatabaseManager
from utils import (
    copy_images_to_static, cleanup_after_processing, 
    cleanup_old_temp_files, delete_batch_files,
    find_original_image_path, create_directories, 
    validate_file_extension, get_file_size_mb,
    validate_upload_quota, create_upload_summary
)
from main import DocumentKIE
from work_package_routes import work_package_bp
from parallel_routes import parallel_bp
from external_routes import external_bp
def get_resource_path(relative_path):
    """Get absolute path to resource, works for dev and for PyInstaller"""
    try:
        # PyInstaller creates a temp folder and stores path in _MEIPASS
        base_path = sys._MEIPASS
        print(f"🚀 PyInstaller mode - _MEIPASS: {base_path}")
    except Exception:
        base_path = os.path.abspath(".")
        print(f"🚀 Development mode - base_path: {base_path}")
    
    resource_path = os.path.join(base_path, relative_path)
    print(f"📁 Resource path for '{relative_path}': {resource_path}")
    return resource_path
def ensure_static_assets():
    """Copy static assets từ _internal ra working directory khi chạy EXE"""
    if not getattr(sys, 'frozen', False):
        print("📝 Running as script - skip copying static assets")
        return
    
    print("🚀 Running as EXE - copying static assets...")
    
    # Get paths
    bundled_static = get_resource_path('static')  # _internal/static
    working_dir = os.getcwd()
    runtime_static = os.path.join(working_dir, 'static')  # working_dir/static
    
    # Folders to copy
    static_folders = ['css', 'js', 'images', 'fonts']
    
    for folder in static_folders:
        src = os.path.join(bundled_static, folder)
        dst = os.path.join(runtime_static, folder)
        
        if os.path.exists(src):
            # Remove old files if exist
            if os.path.exists(dst):
                shutil.rmtree(dst)
            
            # Copy from bundled to runtime
            shutil.copytree(src, dst)
            print(f"✅ Copied static/{folder} -> {dst}")
        else:
            print(f"⚠️ Bundled static/{folder} not found: {src}")
    
    print("✅ Static assets copied successfully!")

# ✅ SỬA HÀM ensure_static_folders() (dòng ~55):
def ensure_static_folders():
    """Ensure static folders exist at runtime - ALWAYS USE WORKING DIRECTORY"""
    # ✅ QUAN TRỌNG: LUÔN DÙNG WORKING DIRECTORY, KHÔNG DÙNG _MEIPASS
    working_dir = os.getcwd()  # ✅ LẤY THƯ MỤC HIỆN TẠI NƠI EXE ĐANG CHẠY
    
    runtime_folders = [
        'static',
        'static/processed_images',
        'uploads',
        'temp_processing',
        'exports'
    ]
    
    for folder in runtime_folders:
        folder_path = os.path.join(working_dir, folder)  # ✅ DÙNG WORKING DIR
        os.makedirs(folder_path, exist_ok=True)
        print(f"✅ Ensured runtime folder exists: {folder_path}")

# ✅ TÌM ĐOẠN FLASK APP INITIALIZATION (dòng ~70):
if getattr(sys, 'frozen', False):
    # Running as exe - use bundled templates but runtime static folder
    print("🚀 Running as EXE (frozen)")
    template_folder = get_resource_path('templates')
    
    # ✅ FIX: DÙNG WORKING DIRECTORY CHO UPLOAD/STATIC
    working_dir = os.getcwd()
    static_folder = os.path.join(working_dir, 'static')
    
    app = Flask(__name__, 
                static_folder=static_folder, 
                template_folder=template_folder)
    
    print(f"📄 Template folder (bundled): {template_folder}")
    print(f"📁 Static folder (runtime): {static_folder}")
    print(f"🗂️ Working directory: {working_dir}")
    
else:
    # Running as script
    print("🚀 Running as Python script")
    app = Flask(__name__)
    print(f"📄 Template folder: {app.template_folder}")
    print(f"📁 Static folder: {app.static_folder}")

ensure_static_folders()
ensure_static_assets()
CORS(app, 
     origins=["*"],
     methods=["GET", "POST", "OPTIONS"],
     allow_headers=["Content-Type", "Authorization"])

# ✅ THÊM NGAY SAU ĐOẠN TRÊN:
# ✅ FIX UPLOAD FOLDER CONFIG BASED ON FROZEN STATE
if getattr(sys, 'frozen', False):
    # ✅ KHI CHẠY EXE - DÙNG WORKING DIRECTORY
    working_dir = os.getcwd()
    app.config['UPLOAD_FOLDER'] = os.path.join(working_dir, 'uploads')
    app.config['OUTPUT_FOLDER'] = os.path.join(working_dir, 'output')
else:
    # ✅ KHI CHẠY PYTHON SCRIPT - DÙNG .ENV
    app.config['UPLOAD_FOLDER'] = os.getenv('UPLOAD_FOLDER', 'uploads')
    app.config['OUTPUT_FOLDER'] = os.getenv('OUTPUT_FOLDER', 'output')

print(f"📂 Upload folder: {app.config['UPLOAD_FOLDER']}")
print(f"📂 Output folder: {app.config['OUTPUT_FOLDER']}")

# ✅ LOAD CONFIG FROM .ENV
app.secret_key = os.getenv('SECRET_KEY', 'your-very-secret-key-change-this-in-production')
app.config['SESSION_COOKIE_HTTPONLY'] = True
app.config['SESSION_COOKIE_SECURE'] = os.getenv('SESSION_COOKIE_SECURE', 'false').lower() == 'true'
app.config['PERMANENT_SESSION_LIFETIME'] = int(os.getenv('SESSION_LIFETIME', '2592000'))  # 30 days default

# Upload configuration
# app.config['UPLOAD_FOLDER'] = os.getenv('UPLOAD_FOLDER', 'uploads')
# app.config['OUTPUT_FOLDER'] = os.getenv('OUTPUT_FOLDER', 'output')
app.config['MAX_CONTENT_LENGTH'] = int(os.getenv('MAX_UPLOAD_SIZE', '10737418240'))  # 10GB default

# ✅ DATABASE CONFIGURATION FROM .ENV
app.config['SQL_SERVER'] = {
    'server': os.getenv('DB_SERVER', '127.0.0.1,1433'),
    'database': os.getenv('DB_DATABASE', 'kie_pla'),
    'username': os.getenv('DB_USERNAME', 'sa'),
    'password': os.getenv('DB_PASSWORD', ''),
    'driver': os.getenv('DB_DRIVER', '{ODBC Driver 17 for SQL Server}')
}

print(f"🔧 Database Config:")
print(f"   Server: {app.config['SQL_SERVER']['server']}")
print(f"   Database: {app.config['SQL_SERVER']['database']}")
print(f"   Username: {app.config['SQL_SERVER']['username']}")
print(f"   Driver: {app.config['SQL_SERVER']['driver']}")

# Khởi tạo database manager
db_manager = DatabaseManager(app.config['SQL_SERVER'])
app.config['db_manager'] = db_manager

# ✅ REGISTER WORK PACKAGE BLUEPRINT
app.register_blueprint(work_package_bp)
app.register_blueprint(parallel_bp)
app.register_blueprint(external_bp)
# Tạo thư mục cần thiết
create_directories()

# Khởi tạo KIE system
kie_system = None

def init_kie_system():
    """Initialize KIE system once at startup with API provider selection"""
    global kie_system
    try:
        api_provider = os.getenv("API_PROVIDER", "vllm").lower()
        vllm_enabled = os.getenv("VLLM_ENABLED", "false").lower() == "true"
        
        print(f"🚀 Initializing KIE system with {api_provider.upper()} API...")
        
        if api_provider == "vllm" and vllm_enabled:
            print("🔌 Using VLLM as primary API provider")
        kie_system = DocumentKIE()
        print("✅ KIE system initialized successfully")
        return True
    except Exception as e:
        print(f"❌ Error initializing KIE system: {e}")
        return False

def get_kie_system():
    """Get the pre-initialized KIE system - no checks needed"""
    return kie_system

# API Routes
@app.route('/')
@login_required
def index():
    # ✅ REDIRECT BASED ON ROLE
    user_role = session.get('user_role')
    if user_role == 'cooperator':
        return redirect('/work-packages-page')
    elif user_role == 'customer':
        return render_template('index.html')
    elif user_role == 'admin':
        return redirect('/admin')
    else:
        return render_template('index.html')

@app.before_request
def require_login():
    # Các endpoint không cần authentication
    allowed_endpoints = ['login_page', 'login', 'static']
    
    if request.endpoint not in allowed_endpoints:
        if 'user_id' not in session:
            return redirect('/login')

@app.route('/login')
def login_page():
    if 'user_id' in session:
        user_role = session.get('user_role')
        if user_role == 'admin':
            return redirect('/admin')
        elif user_role == 'cooperator':
            return redirect('/work-packages-page')  # ✅ COOPERATOR REDIRECT
        elif user_role == 'customer':
            return redirect('/')
    return render_template('login.html')

@app.route('/admin')
@admin_required
def admin_panel():
    return render_template('admin.html')

@app.route('/api/auth/login', methods=['POST'])
def login():
    try:
        data = request.get_json()
        username = data.get('username')
        password = data.get('password')
        
        user = db_manager.authenticate_user(username, password)
        
        if user:
            session['user_id'] = user['id']
            session['username'] = user['username']
            session['full_name'] = user['full_name']
            session['user_role'] = user['role']
            session['email'] = user['email']
            
            # ✅ RETURN REDIRECT URL BASED ON ROLE
            redirect_url = '/'
            if user['role'] == 'admin':
                redirect_url = '/admin'
            elif user['role'] == 'cooperator':
                redirect_url = '/work-packages-page'
            elif user['role'] == 'customer':
                redirect_url = '/'
            
            return jsonify({
                'message': 'Login successful',
                'user': user,
                'redirect_url': redirect_url  # ✅ ADD REDIRECT URL
            })
        else:
            return jsonify({'error': 'Invalid credentials'}), 401
            
    except Exception as e:
        return jsonify({'error': str(e)}), 500


@app.route('/api/auth/me', methods=['GET'])
@login_required
def get_current_user_info():
    return jsonify(get_current_user())

@app.route('/logout')
def logout():
    session.clear()
    return redirect('/login')

# Admin API Routes
@app.route('/api/admin/dashboard', methods=['GET'])
@admin_required
def admin_dashboard():
    try:
        # Get statistics
        users = db_manager.get_users()
        
        # ✅ FIX: Sử dụng function đúng để lấy batches
        batches_result = db_manager.get_accessible_batches_paginated(
            user_id=session['user_id'], 
            user_role=session['user_role'],
            page=1,
            page_size=1000  # Lấy tất cả để đếm
        )
        
        stats = {
            'total_users': len(users),
            'total_customers': len([u for u in users if u['role'] == 'customer']),
            'total_cooperators': len([u for u in users if u['role'] == 'cooperator']),
            'total_batches': batches_result['total_count']  # ✅ Dùng total_count
        }
        
        return jsonify(stats)
    except Exception as e:
        print(f"❌ Error in admin_dashboard: {e}")
        import traceback
        traceback.print_exc()
        return jsonify({'error': str(e)}), 500
@app.route('/api/admin/users/<user_id>', methods=['GET'])
@admin_required
def admin_get_user(user_id):
    """Lấy thông tin user để edit"""
    try:
        conn = db_manager.get_connection()
        cursor = conn.cursor()
        
        cursor.execute('''
            SELECT id, username, full_name, email, role, status, created_at
            FROM users 
            WHERE id = ?
        ''', (user_id,))
        
        row = cursor.fetchone()
        conn.close()
        
        if not row:
            return jsonify({'error': 'User not found'}), 404
            
        user = {
            'id': row[0],
            'username': row[1],
            'full_name': row[2],
            'email': row[3],
            'role': row[4],
            'status': row[5],
            'created_at': row[6].isoformat() if row[6] else None
        }
        
        return jsonify(user)
        
    except Exception as e:
        return jsonify({'error': str(e)}), 500
@app.route('/api/admin/users', methods=['GET'])
@admin_required
def admin_get_users():
    try:
        role_filter = request.args.get('role')
        users = db_manager.get_users(role_filter=role_filter)
        return jsonify(users)
    except Exception as e:
        return jsonify({'error': str(e)}), 500

@app.route('/api/admin/users', methods=['POST'])
@admin_required
def admin_create_user():
    try:
        data = request.get_json()
        
        user_id = db_manager.create_user(
            username=data['username'],
            password=data['password'],
            full_name=data['full_name'],
            email=data.get('email', ''),
            role=data['role'],
            created_by=session['user_id']
        )
        
        return jsonify({'message': 'User created successfully', 'user_id': user_id})
        
    except ValueError as e:
        return jsonify({'error': str(e)}), 400
    except Exception as e:
        return jsonify({'error': str(e)}), 500

@app.route('/api/admin/users/<user_id>', methods=['DELETE'])
@admin_required
def admin_delete_user(user_id):
    try:
        db_manager.delete_user(user_id)
        return jsonify({'message': 'User deleted successfully'})
    except Exception as e:
        return jsonify({'error': str(e)}), 500

@app.route('/api/admin/relationships', methods=['GET'])
@admin_required
def admin_get_relationships():
    try:
        relationships = db_manager.get_user_relationships()
        return jsonify(relationships)
    except Exception as e:
        return jsonify({'error': str(e)}), 500

@app.route('/api/admin/relationships', methods=['POST'])
@admin_required
def admin_create_relationship():
    try:
        data = request.get_json()
        customer_id = data['customer_id']
        created_by = session['user_id']
        
        # Support both single cooperator_id and cooperator_ids array
        cooperator_ids = data.get('cooperator_ids', [])
        if not cooperator_ids and data.get('cooperator_id'):
            cooperator_ids = [data['cooperator_id']]
        
        if not cooperator_ids:
            return jsonify({'error': 'Vui lòng chọn ít nhất 1 cooperator'}), 400
        
        success_count = 0
        errors = []
        for coop_id in cooperator_ids:
            try:
                db_manager.assign_cooperator_to_customer(
                    customer_id=customer_id,
                    cooperator_id=coop_id,
                    created_by=created_by
                )
                success_count += 1
            except ValueError as e:
                errors.append(str(e))
        
        if success_count > 0:
            msg = f'Đã gán {success_count} cooperator thành công'
            if errors:
                msg += f' ({len(errors)} bỏ qua do đã tồn tại)'
            return jsonify({'message': msg})
        else:
            return jsonify({'error': '; '.join(errors) or 'Không gán được cooperator nào'}), 400
        
    except KeyError as e:
        return jsonify({'error': f'Missing field: {e}'}), 400
    except Exception as e:
        return jsonify({'error': str(e)}), 500

@app.route('/api/admin/relationships/<customer_id>/<cooperator_id>', methods=['DELETE'])
@admin_required
def admin_delete_relationship(customer_id, cooperator_id):
    try:
        db_manager.remove_cooperator_from_customer(customer_id, cooperator_id)
        return jsonify({'message': 'Relationship removed successfully'})
    except Exception as e:
        return jsonify({'error': str(e)}), 500

@app.route('/api/admin/batches', methods=['GET'])
@admin_required
def admin_get_batches():
    try:
        # ✅ FIX: Sử dụng function paginated và lấy đúng structure
        batches_result = db_manager.get_accessible_batches_paginated(
            user_id=session['user_id'], 
            user_role=session['user_role'],
            page=1,
            page_size=100  # Lấy 100 batches đầu tiên
        )
        
        # ✅ FIX: Return đúng structure - lấy 'batches' key từ result
        return jsonify(batches_result['batches'])
    except Exception as e:
        print(f"❌ Error in admin_get_batches: {e}")
        import traceback
        traceback.print_exc()
        return jsonify({'error': str(e)}), 500
@app.route('/api/batches', methods=['GET'])
@login_required
def get_batches_paginated():
    """Lấy danh sách batch với pagination, search, filter"""
    try:
        # Get query parameters
        page = int(request.args.get('page', 1))
        page_size = int(request.args.get('page_size', 10))
        search = request.args.get('search', '').strip()
        status_filter = request.args.get('status', '').strip()
        
        # Validate parameters
        if page < 1:
            page = 1
        if page_size < 1 or page_size > 100:
            page_size = 10
        
        # Get paginated batches from database
        result = db_manager.get_accessible_batches_paginated(
            user_id=session['user_id'],
            user_role=session['user_role'],
            page=page,
            page_size=page_size,
            search=search,
            status_filter=status_filter
        )
        
        return jsonify({
            'batches': result['batches'],
            'pagination': {
                'current_page': page,
                'page_size': page_size,
                'total_items': result['total_count'],
                'total_pages': result['total_pages'],
                'has_prev': page > 1,
                'has_next': page < result['total_pages']
            },
            'filters': {
                'search': search,
                'status': status_filter
            }
        })
        
    except Exception as e:
        return jsonify({'error': str(e)}), 500

@app.route('/api/admin/users/<user_id>', methods=['PUT'])
@admin_required
def admin_update_user(user_id):
    """Cập nhật thông tin user"""
    try:
        data = request.get_json()
        
        # Validate required fields
        update_data = {}
        
        if 'full_name' in data:
            update_data['full_name'] = data['full_name']
        
        if 'email' in data:
            update_data['email'] = data['email']
            
        if 'role' in data:
            update_data['role'] = data['role']
            
        if 'status' in data:
            update_data['status'] = data['status']
            
        if 'password' in data and data['password'].strip():
            # Chỉ update password nếu có giá trị
            update_data['password'] = data['password']
        
        # Update user
        db_manager.update_user(user_id, **update_data)
        
        return jsonify({'message': 'User updated successfully'})
        
    except Exception as e:
        return jsonify({'error': str(e)}), 500
@app.route('/api/batch/<batch_id>/documents', methods=['GET'])
@login_required
def get_batch_documents_paginated(batch_id):
    """API lấy documents với pagination, search, filter"""
    try:
        # Check access permission
        user = get_current_user()
        if not can_access_batch(db_manager, user['id'], user['role'], batch_id):
            return jsonify({'error': 'Access denied'}), 403
        
        # Get query parameters
        page = int(request.args.get('page', 1))
        page_size = int(request.args.get('page_size', 10))
        search = request.args.get('search', '').strip()
        status_filter = request.args.get('status', '').strip()
        
        # Validate parameters
        if page < 1:
            page = 1
        if page_size < 1 or page_size > 100:
            page_size = 10
        
        print(f"📊 Getting documents for batch {batch_id}, page {page}, size {page_size}")
        
        # Get paginated documents from database
        result = db_manager.get_batch_documents_paginated(
            batch_id=batch_id,
            page=page,
            page_size=page_size,
            search=search,
            status_filter=status_filter
        )
        
        return jsonify({
            'documents': result['documents'],
            'pagination': {
                'current_page': page,
                'page_size': page_size,
                'total_items': result['total_count'],
                'total_pages': result['total_pages'],
                'has_prev': page > 1,
                'has_next': page < result['total_pages']
            },
            'filters': {
                'search': search,
                'status': status_filter
            }
        })
        
    except Exception as e:
        print(f"❌ Error in get_batch_documents_paginated: {e}")
        import traceback
        traceback.print_exc()
        return jsonify({'error': str(e)}), 500
@app.route('/api/batch/create', methods=['POST'])
@login_required
def create_batch_only():
    """Tạo batch rỗng trước khi upload files"""
    try:
        data = request.get_json()
        batch_id = data.get('batch_id')
        batch_name = data.get('batch_name')
        status = data.get('status', 'uploading')
        
        if not batch_id or not batch_name:
            return jsonify({'error': 'Missing batch_id or batch_name'}), 400
        
        db_manager.create_batch(
            batch_id=batch_id,
            batch_name=batch_name,
            status=status,
            created_by=get_current_user()['id'],
            batch_type='local'
        )
        
        return jsonify({
            'success': True,
            'batch_id': batch_id,
            'message': f'Batch {batch_name} created'
        })
        
    except Exception as e:
        return jsonify({'error': str(e)}), 500

@app.route('/api/batch/<batch_id>/finalize', methods=['POST'])
@login_required
def finalize_batch(batch_id):
    """Cập nhật batch sau khi upload xong"""
    try:
        data = request.get_json()
        total_files = data.get('total_files', 0)
        status = data.get('status', 'uploaded')
        
        db_manager.update_batch(
            batch_id=batch_id,
            total_files=total_files,
            status=status
        )
        
        return jsonify({'success': True})
        
    except Exception as e:
        return jsonify({'error': str(e)}), 500
@app.route('/api/upload_folder', methods=['POST'])
@login_required
def upload_folder():
    """Upload folder chứa tài liệu với validation đầy đủ"""
    try:
        # Kiểm tra files trong request
        if 'files' not in request.files:
            return jsonify({'error': 'No files uploaded'}), 400
        
        files = request.files.getlist('files')
        batch_name = request.form.get('batch_name', f'Batch_{datetime.now().strftime("%Y%m%d_%H%M%S")}')
        
        if not files or len(files) == 0:
            return jsonify({'error': 'No files selected'}), 400
        
        # Lấy thông tin user hiện tại
        current_user = get_current_user()
        
        # Kiểm tra quota upload (tùy chọn)
        # quota_valid, quota_message = validate_upload_quota(
        #     current_user['id'], 
        #     current_user['role'], 
        #     files
        # )
        
        # if not quota_valid:
        #     return jsonify({'error': quota_message}), 400
        
        # Tạo batch ID
        batch_id = str(uuid.uuid4())
        
        print(f"🚀 Starting upload for user: {current_user['full_name']}")
        print(f"📦 Batch: {batch_name} (ID: {batch_id})")
        print(f"📁 Files to process: {len(files)}")
        
        # Tạo batch trong database
        db_manager.create_batch(batch_id, batch_name, 'uploading', current_user['id'], batch_type='local')
        
        # Đảm bảo upload folder tồn tại
        os.makedirs(app.config['UPLOAD_FOLDER'], exist_ok=True)
        
        # Xử lý từng file
        saved_files = 0
        failed_files = []
        total_size_mb = 0
        
        for file in files:
            if file.filename == '':
                continue
                
            original_filename = file.filename
            filename = secure_filename(original_filename)
            
            # Kiểm tra extension
            if not validate_file_extension(filename):
                failed_files.append({
                    'filename': original_filename,
                    'error': 'Unsupported file format. Supported: PDF, PNG, JPG, JPEG, BMP, TIFF'
                })
                continue
            
            try:
                # Tạo document ID và path
                doc_id = str(uuid.uuid4())
                file_path = os.path.join(app.config['UPLOAD_FOLDER'], f"{doc_id}_{filename}")
                
                # Lưu file
                file.save(file_path)
                
                # Kiểm tra file đã lưu thành công
                if not os.path.exists(file_path):
                    raise Exception("File was not saved properly")
                
                # Lấy thông tin file
                file_size_mb = get_file_size_mb(file_path)
                total_size_mb += file_size_mb
                
                # Lưu document vào database
                db_manager.create_document(doc_id, batch_id, original_filename, file_path, current_user['id'])
                
                saved_files += 1
                print(f"✅ Uploaded: {original_filename} ({file_size_mb}MB)")
                
            except Exception as e:
                print(f"❌ Failed to upload {original_filename}: {e}")
                failed_files.append({
                    'filename': original_filename,
                    'error': str(e)
                })
                
                # Cleanup file nếu đã tạo nhưng lỗi
                try:
                    if 'file_path' in locals() and os.path.exists(file_path):
                        os.remove(file_path)
                except:
                    pass
        
        # Cập nhật kết quả batch
        if saved_files > 0:
            db_manager.update_batch(
                batch_id, 
                total_files=saved_files, 
                status='uploaded'
            )
            
            # Tạo response
            response_data = {
                'batch_id': batch_id,
                'batch_name': batch_name,
                'total_files': saved_files,
                'total_size_mb': round(total_size_mb, 2),
                'uploaded_by': current_user['full_name'],
                'message': f'Successfully uploaded {saved_files} files ({round(total_size_mb, 2)}MB)'
            }
            
            # Thêm thông tin về files lỗi nếu có
            if failed_files:
                response_data['failed_files'] = failed_files
                response_data['failed_count'] = len(failed_files)
                response_data['message'] += f'. {len(failed_files)} files failed to upload.'
            
            # Tạo upload summary
            summary = create_upload_summary(batch_id, saved_files, failed_files, current_user)
            response_data['summary'] = summary
            
            print(f"✅ Upload completed: {saved_files}/{saved_files + len(failed_files)} files successful")
            
            return jsonify(response_data)
            
        else:
            # Không có file nào thành công - xóa batch
            try:
                db_manager.delete_batch(batch_id)
            except:
                pass
            
            error_message = 'No files were uploaded successfully.'
            if failed_files:
                error_message += f' All {len(failed_files)} files failed.'
            
            print(f"❌ Upload failed: {error_message}")
            
            return jsonify({
                'error': error_message,
                'failed_files': failed_files,
                'failed_count': len(failed_files)
            }), 400
        
    except Exception as e:
        print(f"❌ Critical error in upload_folder: {str(e)}")
        
        # Cleanup batch nếu đã tạo
        try:
            if 'batch_id' in locals():
                db_manager.delete_batch(batch_id)
        except:
            pass
            
        return jsonify({
            'error': f'Upload failed due to system error: {str(e)}'
        }), 500
@app.route('/api/upload_single_file', methods=['POST'])
@login_required
def upload_single_file():
    """Upload từng file một thay vì toàn bộ batch"""
    try:
        if 'file' not in request.files:
            return jsonify({'success': False, 'error': 'No file'}), 400
        
        batch_id = request.form.get('batch_id')
        if not batch_id:
            return jsonify({'success': False, 'error': 'No batch_id'}), 400
        
        file = request.files['file']
        if file.filename == '':
            return jsonify({'success': False, 'error': 'Empty filename'}), 400
        
        # Validate extension
        valid_extensions = {'.pdf', '.png', '.jpg', '.jpeg', '.bmp', '.tiff'}
        file_ext = os.path.splitext(file.filename)[1].lower()
        if file_ext not in valid_extensions:
            return jsonify({'success': False, 'error': 'Invalid file type'}), 400
        
        # Save file
        filename = secure_filename(file.filename)
        batch_folder = os.path.join(app.config['UPLOAD_FOLDER'], batch_id)
        os.makedirs(batch_folder, exist_ok=True)
        
        file_path = os.path.join(batch_folder, filename)
        file.save(file_path)
        
        # Create document in database
        doc_id = str(uuid.uuid4())
        db_manager.create_document(
            doc_id=doc_id,
            batch_id=batch_id,
            filename=filename,
            file_path=file_path,
            created_by=get_current_user()['id']
        )
        
        return jsonify({
            'success': True,
            'doc_id': doc_id,
            'filename': filename
        })
        
    except Exception as e:
        print(f"❌ Error uploading single file: {str(e)}")
        return jsonify({'success': False, 'error': str(e)}), 500
@app.route('/api/process_batch/<batch_id>', methods=['POST'])
def process_batch(batch_id):
    """Xử lý toàn bộ batch với flow mới"""
    try:
        retry = request.args.get('retry', '0') == '1'
        
        # Kiểm tra batch tồn tại
        batch_status = db_manager.get_batch_status(batch_id)
        if not batch_status:
            return jsonify({'error': 'Batch not found'}), 404
        
        if retry:
            # Reset trạng thái documents nếu retry
            db_manager.update_batch(batch_id, processed_files=0, status='uploaded')
            print(f"🔄 Reset batch {batch_id} for retry")
        
        # Chạy xử lý trong background thread
        thread = Thread(target=process_batch_background, args=(batch_id,))
        thread.daemon = True
        thread.start()
        
        return jsonify({'message': 'Processing started', 'batch_id': batch_id})
        
    except Exception as e:
        return jsonify({'error': str(e)}), 500



def process_batch_background(batch_id):
    """Xử lý batch trong background với flow mới"""
    try:
        # Cập nhật status batch
        db_manager.update_batch(batch_id, status='processing')
        
        # Lấy danh sách documents cần xử lý
        documents = db_manager.get_documents_for_processing(batch_id)
        
        kie = get_kie_system()
        if not kie:
            raise Exception("Cannot initialize KIE system")
        
        processed_count = 0
        total_count = len(documents)
        
        for doc_id, file_path, filename in documents:
            try:
                print(f"Processing document: {filename}")
                
                # Cập nhật status document
                db_manager.update_document(doc_id, status='processing')
                
                # Xử lý document với flow mới, truyền doc_id để tạo thư mục riêng
                results = kie.process_document(file_path, doc_id)
                
                # Copy images to static folder
                static_paths = copy_images_to_static(doc_id, results['image_paths'])
                
                # Extract document type từ header KIE results
                document_type = results.get('kie_header_data', {}).get('document_type', {}).get('value', 'N/A')
                
                # Lưu kết quả vào database
                kie_header_data = results.get('kie_header_data', {})
                kie_content_data = results.get('kie_content_data', {})
                header_detection = results.get('header_detection', {})
                
                db_manager.update_document(
                    doc_id,
                    kie_header_data=kie_header_data,
                    kie_content_data=kie_content_data,
                    cut_point=results.get('cut_point', 0),
                    image_full_path=static_paths.get('full'),
                    image_header_path=static_paths.get('header'),
                    image_content_path=static_paths.get('content'),
                    header_detection=header_detection,
                    total_pages=results.get('total_pages', 1),
                    status='completed',
                    processed_at=datetime.now(),
                    document_type=document_type
                )
                
                processed_count += 1
                
                # ✅ CẬP NHẬT PROGRESS BATCH LIÊN TỤC
                db_manager.update_batch(batch_id, processed_files=processed_count)
                
                # Cleanup temp files
                cleanup_after_processing(doc_id, results)
                
                print(f"✅ Successfully processed: {filename} ({processed_count}/{total_count})")
                
            except Exception as e:
                print(f"❌ Error processing {filename}: {str(e)}")
                try:
                    error_data = {'error': str(e), 'timestamp': datetime.now().isoformat()}
                    
                    db_manager.update_document(
                        doc_id,
                        status='error',
                        kie_header_data=error_data,
                        kie_content_data=error_data
                    )
                except Exception as update_error:
                    print(f"❌ Error updating document status: {update_error}")
                
                # Cleanup kể cả khi lỗi
                cleanup_after_processing(doc_id, None, keep_original=True)
        
        # ✅ CẬP NHẬT BATCH STATUS CUỐI CÙNG
        update_batch_status_after_document_change(batch_id)
        
        print(f"✅ Batch {batch_id} processing completed - {processed_count}/{total_count} documents processed")
        
    except Exception as e:
        print(f"❌ Error processing batch {batch_id}: {str(e)}")
        try:
            db_manager.update_batch(batch_id, status='error')
        except:
            pass


@app.route('/api/batch/<batch_id>/refresh-status', methods=['POST'])
@login_required
def refresh_batch_status(batch_id):
    """API để manual refresh batch status"""
    try:
        # Check access permission
        if not can_access_batch(db_manager, session['user_id'], session['user_role'], batch_id):
            return jsonify({'error': 'Access denied'}), 403
        
        # Update batch status
        update_batch_status_after_document_change(batch_id)
        
        # Get updated status
        batch_status = db_manager.get_batch_status(batch_id)
        
        return jsonify({
            'message': 'Batch status refreshed successfully',
            'batch_status': batch_status
        })
        
    except Exception as e:
        return jsonify({'error': str(e)}), 500

@app.route('/api/process_document/<doc_id>', methods=['POST'])
def process_single_document(doc_id):
    """Xử lý một document riêng lẻ"""
    try:
        retry = request.args.get('retry', '0') == '1'
        
        # Lấy thông tin document
        document = db_manager.get_document(doc_id)
        if not document:
            return jsonify({'error': 'Document not found'}), 404
        
        current_status = document['status']
        
        if retry or current_status == 'error':
            # Reset status nếu retry hoặc đang lỗi
            db_manager.update_document(doc_id, status='pending', processed_at=None)
            print(f"🔄 Reset document {document['filename']} for retry")
        elif current_status == 'completed':
            return jsonify({'error': 'Document already completed. Use retry=1 to reprocess'}), 400
        elif current_status == 'processing':
            return jsonify({'error': 'Document is currently being processed'}), 400
        
        # Chạy xử lý trong background thread
        thread = Thread(target=process_single_document_background, args=(doc_id,))
        thread.daemon = True
        thread.start()
        
        return jsonify({
            'message': f'Started processing document: {document["filename"]}',
            'doc_id': doc_id
        })
        
    except Exception as e:
        return jsonify({'error': str(e)}), 500

def process_single_document_background(doc_id):
    """Xử lý một document trong background với KIE data tách riêng"""
    try:
        document = db_manager.get_document(doc_id)
        print(f"Processing single document: {document['filename']}")
        
        # ✅ LẤY BATCH_ID TRƯỚC KHI XỬ LÝ
        batch_id = None
        
        # Lấy batch_id từ document
        conn = db_manager.get_connection()
        cursor = conn.cursor()
        cursor.execute('SELECT batch_id FROM documents WHERE id = ?', (doc_id,))
        result = cursor.fetchone()
        if result:
            batch_id = result[0]
        conn.close()
        
        # Cập nhật status document
        db_manager.update_document(doc_id, status='processing')
        
        # ✅ CẬP NHẬT BATCH STATUS KHI BẮT ĐẦU
        if batch_id:
            update_batch_status_after_document_change(batch_id)
        
        # Khởi tạo KIE system
        kie = get_kie_system()
        if not kie:
            raise Exception("Cannot initialize KIE system")
        
        # Xử lý document với flow mới
        results = kie.process_document(document['file_path'], doc_id)
        
        # Copy images to static folder
        static_paths = copy_images_to_static(doc_id, results['image_paths'])
        
        # ✅ FIX: Extract document type với flat format
        kie_header_data = results.get('kie_header_data', {})
        document_type = "N/A"
        
        if isinstance(kie_header_data, dict):
            # ✅ FLAT FORMAT - Direct access
            document_type = kie_header_data.get('document_type', 'N/A')
            if not isinstance(document_type, str):
                # ✅ BACKWARD COMPATIBILITY
                if isinstance(document_type, dict) and 'value' in document_type:
                    document_type = document_type['value']
                else:
                    document_type = str(document_type) if document_type else 'N/A'
        
        kie_content_data = results.get('kie_content_data', {})
        header_detection = results.get('header_detection', {})
        
        # Lưu kết quả vào database với KIE data tách riêng
        db_manager.update_document(
            doc_id,
            kie_header_data=kie_header_data,
            kie_content_data=kie_content_data,
            cut_point=results.get('cut_point', 0),
            image_full_path=static_paths.get('full'),
            image_header_path=static_paths.get('header'),
            image_content_path=static_paths.get('content'),
            header_detection=header_detection,
            total_pages=results.get('total_pages', 1),
            status='completed',
            processed_at=datetime.now(),
            document_type=document_type  # ✅ Now guaranteed to be string
        )
        
        # ✅ CẬP NHẬT BATCH STATUS SAU KHI HOÀN THÀNH
        if batch_id:
            update_batch_status_after_document_change(batch_id)
        
        # Cleanup temp files
        cleanup_after_processing(doc_id, results)
        
        print(f"✅ Successfully processed single document: {document['filename']}")
        
    except Exception as e:
        print(f"❌ Error processing single document: {str(e)}")
        try:
            # Cập nhật status lỗi
            db_manager.update_document(
                doc_id,
                status='error',
                kie_header_data={'error': str(e)},
                kie_content_data={'error': str(e)}
            )
            
            # ✅ CẬP NHẬT BATCH STATUS KHI CÓ LỖI
            if batch_id:
                update_batch_status_after_document_change(batch_id)
            
            # Cleanup kể cả khi lỗi, nhưng giữ original để debug
            cleanup_after_processing(doc_id, None, keep_original=True)
        except:
            pass

@app.route('/api/document/<doc_id>/status', methods=['GET'])
def get_document_status(doc_id):
    """Lấy trạng thái xử lý của một document"""
    try:
        status = db_manager.get_document_status(doc_id)
        if not status:
            return jsonify({'error': 'Document not found'}), 404
        return jsonify(status)
    except Exception as e:
        return jsonify({'error': str(e)}), 500

@app.route('/api/document/<doc_id>', methods=['GET'])
def get_document(doc_id):
    """Lấy thông tin chi tiết document"""
    try:
        document = db_manager.get_document(doc_id)
        if not document:
            return jsonify({'error': 'Document not found'}), 404
        return jsonify(document)
    except Exception as e:
        return jsonify({'error': str(e)}), 500
@app.route('/api/document/<doc_id>', methods=['PUT'])
def update_document(doc_id):
    """Cập nhật thông tin document với KIE data tách riêng"""
    try:
        data = request.get_json()
        
        update_data = {
            'processed_at': datetime.now()
        }
        
        # Cập nhật header KIE data
        if 'kie_header_data' in data:
            update_data['kie_header_data'] = data['kie_header_data']
        
        # Cập nhật content KIE data
        if 'kie_content_data' in data:
            update_data['kie_content_data'] = data['kie_content_data']
        
        # ✅ THÊM 2 TRƯỜNG MỚI
        if 'document_original_type' in data:
            update_data['document_original_type'] = data['document_original_type']
        
        if 'document_note' in data:
            update_data['document_note'] = data['document_note']
        
        # Gọi update một lần với tất cả dữ liệu
        db_manager.update_document(doc_id, **update_data)
        
        return jsonify({'success': True, 'message': 'Document updated successfully'})
        
    except Exception as e:
        print(f"❌ Error updating document: {str(e)}")
        import traceback
        traceback.print_exc()
        return jsonify({'success': False, 'error': str(e)}), 500

@app.route('/api/field_definitions', methods=['GET'])
def get_field_definitions():
    """Lấy định nghĩa các trường"""
    try:
        fields = db_manager.get_field_definitions()
        return jsonify(fields)
    except Exception as e:
        return jsonify({'error': str(e)}), 500

@app.route('/api/batch/<batch_id>/status', methods=['GET'])
def get_batch_status(batch_id):
    """Lấy trạng thái xử lý batch"""
    try:
        progress = db_manager.get_batch_status(batch_id)
        if not progress:
            return jsonify({'error': 'Batch not found'}), 404
        return jsonify(progress)
    except Exception as e:
        return jsonify({'error': str(e)}), 500

@app.route('/api/cleanup/temp_files', methods=['POST'])
def cleanup_temp_files_api():
    """API để dọn dẹp manual các temp files"""
    try:
        data = request.get_json()
        doc_id = data.get('doc_id')
        
        if doc_id:
            # Cleanup cho document cụ thể
            cleanup_after_processing(doc_id, None, keep_original=True)
            return jsonify({'message': f'Cleaned up temp files for document {doc_id}'})
        else:
            # Cleanup toàn bộ temp folder
            cleanup_old_temp_files()
            return jsonify({'message': 'Cleaned up old temp files (>24h)'})
                
    except Exception as e:
        return jsonify({'error': str(e)}), 500


@app.route('/api/batch/<batch_id>', methods=['DELETE'])
@login_required
def delete_batch(batch_id):
    """Xóa batch và toàn bộ documents liên quan - KHÔNG XÓA FILE EXTERNAL"""
    try:
        # Check access permission
        if not can_access_batch(db_manager, session['user_id'], session['user_role'], batch_id):
            return jsonify({'error': 'Access denied'}), 403
        
        # ✅ LẤY THÔNG TIN BATCH TRƯỚC KHI XÓA
        batch_info = db_manager.get_batch_info(batch_id)
        if not batch_info:
            print(f"⚠️ Batch {batch_id} not found, may already be deleted")
            return jsonify({'error': 'Batch not found or already deleted'}), 404
        
        batch_type = batch_info.get('batch_type', 'local')  # ✅ LẤY BATCH TYPE
        
        print(f"🗑️ User {session.get('username')} đang xóa {batch_type} batch: {batch_id}")
        
        # ✅ XÓA BATCH VÀ TẤT CẢ DỮ LIỆU LIÊN QUAN
        document_data = db_manager.delete_batch(batch_id)
        
        # ✅ XÓA FILES VẬT LÝ THEO BATCH TYPE
        if document_data:
            if batch_type == 'local':
                print(f"🗑️ Đang xóa {len(document_data)} LOCAL files...")
                delete_batch_files(document_data)  # ✅ XÓA TẤT CẢ FILES
            elif batch_type == 'external':
                print(f"🔒 Đang xóa PROCESSED files cho {len(document_data)} EXTERNAL files...")
                # ✅ IMPORT FUNCTION NÀY
                from utils import delete_batch_processed_files_only
                delete_batch_processed_files_only(document_data)  # ✅ CHỈ XÓA PROCESSED
            else:
                print(f"⚠️ Unknown batch_type: {batch_type}, treating as local")
                delete_batch_files(document_data)
        
        return jsonify({
            'message': f'{batch_type.title()} batch đã được xóa thành công ({len(document_data)} documents)',
            'deleted_documents': len(document_data),
            'batch_type': batch_type,
            'files_deleted': batch_type == 'local',  # ✅ CHỈ XÓA FILES KHI LOCAL
            'preserved_originals': batch_type == 'external'  # ✅ PRESERVED KHI EXTERNAL
        })
        
    except ValueError as e:
        return jsonify({'error': str(e)}), 404
    except PermissionError as e:
        return jsonify({'error': str(e)}), 403
    except Exception as e:
        print(f"❌ Critical error deleting batch {batch_id}: {str(e)}")
        import traceback
        traceback.print_exc()
        return jsonify({'error': f'Lỗi hệ thống khi xóa batch: {str(e)}'}), 500
@app.route('/api/document/<doc_id>/recut', methods=['POST'])
def recut_document(doc_id):
    """Cắt lại ảnh với điểm cắt mới"""
    try:
        data = request.get_json()
        new_cut_point = data.get('cut_point')
        
        if not new_cut_point or new_cut_point < 50:
            return jsonify({'error': 'Điểm cắt phải >= 50px'}), 400
        
        print(f"🔄 Recut request for doc {doc_id} with cut point: {new_cut_point}")
        
        # Lấy thông tin document
        document = db_manager.get_document(doc_id)
        if not document:
            return jsonify({'error': 'Document not found'}), 404
            
        print(f"📄 Document found: {document.get('filename', 'N/A')}")
        
        # ✅ FIX: Tìm ảnh gốc trong temp folder
        temp_dir = os.path.join("temp_processing", f"doc_{doc_id}")
        original_full_path = None
        
        print(f"🔍 Looking for original image in: {temp_dir}")
        
        if os.path.exists(temp_dir):
            for file in os.listdir(temp_dir):
                # Tìm file full_document
                if file.startswith('full_document'):
                    original_full_path = os.path.join(temp_dir, file)
                    print(f"✅ Found original image: {original_full_path}")
                    break
        
        if not original_full_path or not os.path.exists(original_full_path):
            print(f"❌ Original image not found in {temp_dir}")
            # List files in temp dir for debugging
            if os.path.exists(temp_dir):
                files = os.listdir(temp_dir)
                print(f"📁 Files in temp dir: {files}")
            return jsonify({'error': 'Không tìm thấy ảnh gốc để cắt lại. Temp files có thể đã bị xóa.'}), 404
        
        # Cắt lại với điểm cắt mới
        kie = get_kie_system()
        if not kie:
            return jsonify({'error': 'KIE system not available'}), 500
            
        print(f"✂️ Cutting image with new cut point: {new_cut_point}")
        
        # ✅ FIX: Sử dụng hàm phù hợp dựa trên loại file
        file_ext = os.path.splitext(original_full_path)[1].lower()
        
        if file_ext == '.png' and 'full_document' in os.path.basename(original_full_path):
            # PDF đã được convert thành full_document.png
            header_path, content_path = kie.crop_full_document_sections(original_full_path, new_cut_point)
        else:
            # Ảnh đơn
            header_path, content_path = kie.crop_image_sections(original_full_path, new_cut_point)
        
        if not header_path:
            return jsonify({'error': 'Lỗi khi cắt ảnh'}), 500
        
        print(f"✅ Cut completed. Header: {header_path}, Content: {content_path}")
        
        # Copy ảnh mới vào static folder
        image_paths = {
            'full': original_full_path,
            'header': header_path,
            'content': content_path
        }
        static_paths = copy_images_to_static(doc_id, image_paths)
        
        print(f"📁 Static paths: {static_paths}")
        
        # Cập nhật database
        update_data = {
            'cut_point': new_cut_point,
            'image_header_path': static_paths.get('header'),
            'image_content_path': static_paths.get('content')
        }
        
        db_manager.update_document(doc_id, **update_data)
        
        print(f"✅ Database updated successfully")
        
        return jsonify({
            'message': 'Đã cắt lại ảnh thành công',
            'cut_point': new_cut_point,
            'image_paths': static_paths
        })
        
    except Exception as e:
        print(f"❌ Error in recut_document: {str(e)}")
        import traceback
        traceback.print_exc()
        return jsonify({'error': str(e)}), 500
@app.route('/api/document/<doc_id>/auto_cut', methods=['POST'])
def auto_cut_document(doc_id):
    """Tự động detect và cắt lại với YOLO"""
    try:
        # Lấy thông tin document
        document = db_manager.get_document(doc_id)
        if not document or not document['image_paths']['full']:
            return jsonify({'error': 'Không tìm thấy ảnh gốc'}), 404
        
        # Tìm ảnh gốc trong temp folder
        original_full_path = find_original_image_path(doc_id)
        if not original_full_path:
            return jsonify({'error': 'Không tìm thấy ảnh gốc để detect'}), 404
        
        # Detect header với YOLO
        kie = get_kie_system()
        header_detection = kie.detect_header_with_yolo(original_full_path)
        
        if not header_detection:
            return jsonify({'error': 'Không thể detect header'}), 500
        
        new_cut_point = header_detection['bottom_y']
        
        # Cắt lại với điểm cắt mới
        header_path, content_path = kie.crop_image_sections(original_full_path, new_cut_point)
        
        if not header_path:
            return jsonify({'error': 'Lỗi khi cắt ảnh'}), 500
        
        # Copy ảnh mới vào static folder
        image_paths = {
            'full': original_full_path,
            'header': header_path,
            'content': content_path
        }
        static_paths = copy_images_to_static(doc_id, image_paths)
        
        # Cập nhật database
        db_manager.update_document(
            doc_id,
            cut_point=new_cut_point,
            image_header_path=static_paths.get('header'),
            image_content_path=static_paths.get('content'),
            header_detection=header_detection
        )
        
        return jsonify({
            'message': 'Đã tự động detect và cắt thành công',
            'cut_point': new_cut_point,
            'image_paths': static_paths,
            'header_detection': header_detection
        })
        
    except Exception as e:
        return jsonify({'error': str(e)}), 500

@app.route('/api/document/<doc_id>/reprocess', methods=['POST'])
def reprocess_document_kie(doc_id):
    """Xử lý lại KIE cho document với data tách riêng"""
    try:
        # Lấy thông tin document
        document = db_manager.get_document(doc_id)
        if not document or not document['image_paths']['header']:
            return jsonify({'error': 'Không tìm thấy ảnh header'}), 404
        
        # ✅ LẤY ẢNH TỪ STATIC/PROCESSED_IMAGES THAY VÌ TEMP
        header_image_path = document['image_paths']['header']
        
        # Chuyển đổi đường dẫn tương đối sang absolute path
        if not os.path.isabs(header_image_path):
            # Nếu path là processed_images/doc_xxx/header.png
            if getattr(sys, 'frozen', False):
                # EXE mode - dùng working directory
                header_image_path = os.path.join(os.getcwd(), header_image_path)
            else:
                # Script mode - dùng script directory
                header_image_path = os.path.join(os.path.dirname(__file__), header_image_path)
        
        if not os.path.exists(header_image_path):
            return jsonify({'error': f'Không tìm thấy ảnh header tại: {header_image_path}'}), 404
        
        print(f"🔄 Reprocessing KIE for doc {doc_id} with header: {header_image_path}")
        
        # KIE lại header
        kie = get_kie_system()
        kie_header_results = kie.analyze_header_kie(header_image_path)
        
        # Giữ nguyên content data cũ
        kie_content_results = document.get('kie_content_data', {})
        
        # Cập nhật database
        db_manager.update_document(
            doc_id,
            kie_header_data=kie_header_results,
            kie_content_data=kie_content_results,
            processed_at=datetime.now()
        )
        
        print(f"✅ Reprocess KIE completed for doc {doc_id}")
        
        return jsonify({
            'message': 'Đã xử lý lại KIE header thành công',
            'kie_header_data': kie_header_results,
            'kie_content_data': kie_content_results
        })
        
    except Exception as e:
        print(f"❌ Error reprocessing KIE: {e}")
        traceback.print_exc()
        return jsonify({'error': str(e)}), 500

def update_batch_status_after_document_change(batch_id):
    """Kiểm tra và cập nhật status batch sau khi document thay đổi"""
    try:
        # Lấy tất cả documents trong batch
        documents = db_manager.get_documents_by_batch(batch_id)
        
        if not documents:
            # ✅ NẾU KHÔNG CÒN DOCUMENT NÀO - CẬP NHẬT BATCH
            db_manager.update_batch(
                batch_id,
                total_files=0,
                processed_files=0,
                status='empty'
            )
            print(f"✅ Batch {batch_id} is now empty")
            return
        
        total_docs = len(documents)
        completed_docs = len([d for d in documents if d['status'] == 'completed'])
        error_docs = len([d for d in documents if d['status'] == 'error'])
        processing_docs = len([d for d in documents if d['status'] == 'processing'])
        pending_docs = len([d for d in documents if d['status'] == 'pending'])
        
        # Xác định batch status
        if completed_docs == total_docs:
            new_batch_status = 'completed'
        elif error_docs == total_docs:
            new_batch_status = 'error'  
        elif processing_docs > 0:
            new_batch_status = 'processing'
        elif pending_docs > 0:
            new_batch_status = 'uploaded'
        elif completed_docs > 0:
            new_batch_status = 'processing'
        else:
            new_batch_status = 'uploaded'
        
        # Cập nhật batch
        db_manager.update_batch(
            batch_id,
            total_files=total_docs,  # ✅ CẬP NHẬT LẠI TOTAL FILES
            processed_files=completed_docs,
            status=new_batch_status
        )
        
        print(f"✅ Updated batch {batch_id}: {completed_docs}/{total_docs} completed, status: {new_batch_status}")
        
    except Exception as e:
        print(f"❌ Error updating batch status: {e}")

@app.route('/export-training-data')
@login_required
@role_required('admin')  # Chỉ admin mới được export
def export_training_data_page():
    return render_template('export_training_data.html')

@app.route('/api/export/batches', methods=['GET'])
@login_required
@role_required('admin')
def get_export_batches():
    """Lấy danh sách batch có thể export"""
    try:
        # Lấy batches có documents với header KIE data
        conn = db_manager.get_connection()
        cursor = conn.cursor()
        
        # ✅ FIX: Sử dụng CAST và LEN thay vì so sánh trực tiếp với ntext
        cursor.execute('''
            SELECT DISTINCT b.id, b.name, b.created_at, 
                   COUNT(d.id) as total_docs,
                   COUNT(CASE WHEN d.kie_header_data IS NOT NULL 
                              AND LEN(CAST(d.kie_header_data AS NVARCHAR(MAX))) > 0
                              AND d.image_header_path IS NOT NULL 
                         THEN 1 END) as valid_headers
            FROM batches b
            LEFT JOIN documents d ON b.id = d.batch_id
            WHERE b.status = 'completed'
            GROUP BY b.id, b.name, b.created_at
            HAVING COUNT(CASE WHEN d.kie_header_data IS NOT NULL 
                              AND LEN(CAST(d.kie_header_data AS NVARCHAR(MAX))) > 0
                              AND d.image_header_path IS NOT NULL 
                         THEN 1 END) > 0
            ORDER BY b.created_at DESC
        ''')
        
        batches = []
        for row in cursor.fetchall():
            batches.append({
                'id': row[0],
                'name': row[1],
                'created_at': row[2].isoformat() if row[2] else None,
                'total_docs': row[3],
                'valid_headers': row[4]
            })
        
        conn.close()
        return jsonify(batches)
        
    except Exception as e:
        print(f"❌ Error in get_export_batches: {e}")
        import traceback
        traceback.print_exc()
        return jsonify({'error': str(e)}), 500

@app.route('/api/export/training-data', methods=['POST'])
@login_required
@role_required('admin')
def export_training_data():
    """Export dữ liệu training với format JSONL cho Vintern - với custom prompt và fields"""
    try:
        data = request.get_json()
        batch_ids = data.get('batch_ids', [])
        export_name = data.get('export_name', f'vintern_export_{datetime.now().strftime("%Y%m%d_%H%M%S")}')
        custom_prompt = data.get('custom_prompt', '').strip()  # ✅ CUSTOM PROMPT
        selected_fields = data.get('selected_fields', [])      # ✅ SELECTED FIELDS
        
        if not batch_ids:
            return jsonify({'error': 'Vui lòng chọn ít nhất 1 batch'}), 400
        
        if not custom_prompt:
            return jsonify({'error': 'Vui lòng nhập prompt cho training'}), 400
            
        if not selected_fields:
            return jsonify({'error': 'Vui lòng chọn ít nhất 1 field để export'}), 400
        
        print(f"🚀 Starting export for batches: {batch_ids}")
        print(f"📦 Export name: {export_name}")
        print(f"📝 Custom prompt: {custom_prompt[:100]}...")
        print(f"🔖 Selected fields: {selected_fields}")
        
        # Tạo thư mục export
        export_dir = os.path.join("exports", export_name)
        images_dir = os.path.join(export_dir, "images")
        os.makedirs(images_dir, exist_ok=True)
        
        # Lấy dữ liệu từ database
        conn = db_manager.get_connection()
        cursor = conn.cursor()
        
        placeholders = ','.join(['?' for _ in batch_ids])
        cursor.execute(f'''
            SELECT d.id, d.filename, d.kie_header_data, d.image_header_path, b.name as batch_name
            FROM documents d
            JOIN batches b ON d.batch_id = b.id
            WHERE d.batch_id IN ({placeholders})
              AND d.kie_header_data IS NOT NULL 
              AND LEN(CAST(d.kie_header_data AS NVARCHAR(MAX))) > 0
              AND d.image_header_path IS NOT NULL
              AND d.status = 'completed'
            ORDER BY b.created_at, d.filename
        ''', batch_ids)
        
        documents = cursor.fetchall()
        conn.close()
        
        if not documents:
            return jsonify({'error': 'Không tìm thấy documents hợp lệ trong các batch đã chọn'}), 400
        
        print(f"📊 Found {len(documents)} valid documents")
        
        # ✅ TẠO JSON STRUCTURE CHO SELECTED FIELDS
        json_structure = {field: "" for field in selected_fields}
        json_structure_str = json.dumps(json_structure, ensure_ascii=False, indent=2)
        
        # ✅ FINAL PROMPT WITH JSON STRUCTURE
        final_prompt = f"{custom_prompt}\n\nTrả về JSON theo cấu trúc sau:\n{json_structure_str}"
        
        train_data = []
        val_data = []
        exported_count = 0
        failed_count = 0
        
        for doc in documents:
            doc_id, filename, kie_header_json, image_header_path, batch_name = doc
            
            try:
                # Parse KIE data
                if isinstance(kie_header_json, str):
                    kie_data = json.loads(kie_header_json)
                else:
                    kie_data = kie_header_json
                
                if not kie_data or not isinstance(kie_data, dict):
                    print(f"⚠️ Invalid KIE data for {filename}")
                    failed_count += 1
                    continue
                
                if not os.path.exists(image_header_path):
                    print(f"⚠️ Header image not found: {image_header_path}")
                    failed_count += 1
                    continue
                
                # Copy image to export directory
                image_filename = f"{doc_id}_header.png"
                export_image_path = os.path.join(images_dir, image_filename)
                
                try:
                    import shutil
                    shutil.copy2(image_header_path, export_image_path)
                except Exception as copy_error:
                    print(f"❌ Failed to copy image {image_header_path}: {copy_error}")
                    failed_count += 1
                    continue
                
                # ✅ TẠO EXPECTED ANSWER CHỈ VỚI SELECTED FIELDS
                expected_answer = {}
                for field in selected_fields:
                    expected_answer[field] = kie_data.get(field, '')
                
                # Tạo conversation format cho Vintern
                sample = {
                    "id": f"{batch_name}_{doc_id}",
                    "image": f"{image_filename}",
                    "conversations": [
                        {
                            "from": "human",
                            "value": f"<image>\n{final_prompt}"  # ✅ SỬ DỤNG CUSTOM PROMPT
                        },
                        {
                            "from": "gpt", 
                            "value": json.dumps(expected_answer, ensure_ascii=False, indent=2)
                        }
                    ]
                }
                
                # 80/20 train/val split
                # if exported_count % 5 == 0:  # 20% validation
                #     val_data.append(sample)
                # else:
                train_data.append(sample)
                
                exported_count += 1
                
                if exported_count % 50 == 0:
                    print(f"📈 Exported {exported_count} samples...")
                
            except Exception as e:
                print(f"❌ Error processing {filename}: {e}")
                failed_count += 1
                continue
        
        if exported_count == 0:
            return jsonify({'error': 'Không có dữ liệu hợp lệ để export'}), 400
        
        # Lưu JSONL files
        train_file = os.path.join(export_dir, "train.jsonl")
        val_file = os.path.join(export_dir, "val.jsonl")
        
        with open(train_file, 'w', encoding='utf-8') as f:
            for sample in train_data:
                f.write(json.dumps(sample, ensure_ascii=False) + '\n')
        
        with open(val_file, 'w', encoding='utf-8') as f:
            for sample in val_data:
                f.write(json.dumps(sample, ensure_ascii=False) + '\n')
        
        # ✅ METADATA VỚI CUSTOM CONFIG
        metadata = {
            "export_name": export_name,
            "export_date": datetime.now().isoformat(),
            "exported_by": session.get('username'),
            "batch_ids": batch_ids,
            "total_samples": exported_count,
            "train_samples": len(train_data),
            "val_samples": len(val_data),
            "failed_samples": failed_count,
            "custom_prompt": custom_prompt,           # ✅ LƯU CUSTOM PROMPT
            "selected_fields": selected_fields,       # ✅ LƯU SELECTED FIELDS
            "json_structure": json_structure,         # ✅ JSON STRUCTURE
            "final_prompt": final_prompt              # ✅ FINAL PROMPT USED
        }
        
        metadata_file = os.path.join(export_dir, "metadata.json")
        with open(metadata_file, 'w', encoding='utf-8') as f:
            json.dump(metadata, f, ensure_ascii=False, indent=2)
        
        print(f"✅ Export completed successfully!")
        print(f"📊 Training samples: {len(train_data)}")
        print(f"📊 Validation samples: {len(val_data)}")
        print(f"❌ Failed samples: {failed_count}")
        
        return jsonify({
            'message': 'Export thành công!',
            'export_path': export_dir,
            'metadata': metadata
        })
        
    except Exception as e:
        print(f"❌ Export error: {str(e)}")
        import traceback
        traceback.print_exc()
        return jsonify({'error': f'Export failed: {str(e)}'}), 500
@app.route('/api/document/<doc_id>', methods=['DELETE'])
@login_required
def delete_document(doc_id):
    """Xóa document và tất cả files liên quan"""
    try:
        # ✅ LẤY THÔNG TIN DOCUMENT TRƯỚC KHI XÓA
        document = db_manager.get_document(doc_id)
        if not document:
            return jsonify({'error': 'Document not found'}), 404
        
        # ✅ KIỂM TRA QUYỀN TRUY CẬP BATCH
        conn = db_manager.get_connection()
        cursor = conn.cursor()
        cursor.execute('SELECT batch_id FROM documents WHERE id = ?', (doc_id,))
        result = cursor.fetchone()
        
        if not result:
            conn.close()
            return jsonify({'error': 'Document not found'}), 404
        
        batch_id = result[0]
        conn.close()
        
        # Check batch access permission
        if not can_access_batch(db_manager, session['user_id'], session['user_role'], batch_id):
            return jsonify({'error': 'Access denied'}), 403
        
        print(f"🗑️ User {session.get('username')} đang xóa document: {doc_id}")
        print(f"📄 Document: {document.get('filename', 'N/A')}")
        
        # ✅ XÓA WORK_ITEMS LIÊN QUAN TRƯỚC
        db_manager.delete_work_items_by_doc_ids([doc_id])
        
        # ✅ XÓA DOCUMENT KHỎI DATABASE
        conn = db_manager.get_connection()
        cursor = conn.cursor()
        
        cursor.execute('DELETE FROM documents WHERE id = ?', (doc_id,))
        deleted_count = cursor.rowcount
        
        if deleted_count == 0:
            conn.rollback()
            conn.close()
            return jsonify({'error': 'Document not found or already deleted'}), 404
        
        conn.commit()
        conn.close()
        
        # ✅ XÓA FILES VẬT LÝ
        files_to_delete = []
        
        # Original file
        if document.get('file_path') and os.path.exists(document['file_path']):
            files_to_delete.append(document['file_path'])
        
        # Processed images
        image_paths = document.get('image_paths', {})
        for path_type, path in image_paths.items():
            if path and os.path.exists(path):
                files_to_delete.append(path)
        
        # Temp directory
        from utils import cleanup_document_temp_all_locations
        cleanup_document_temp_all_locations(doc_id)
        
        # Delete individual files
        deleted_files = 0
        for file_path in files_to_delete:
            try:
                os.remove(file_path)
                deleted_files += 1
                print(f"🗑️ Đã xóa file: {file_path}")
            except Exception as e:
                print(f"⚠️ Không thể xóa file {file_path}: {e}")
        
        # ✅ CẬP NHẬT BATCH STATUS SAU KHI XÓA
        update_batch_status_after_document_change(batch_id)
        
        return jsonify({
            'message': f'Document đã được xóa thành công',
            'deleted_files': deleted_files,
            'filename': document.get('filename', 'N/A')
        })
        
    except Exception as e:
        print(f"❌ Critical error deleting document {doc_id}: {str(e)}")
        import traceback
        traceback.print_exc()
        return jsonify({'error': f'Lỗi hệ thống khi xóa document: {str(e)}'}), 500

@app.route('/api/batch/<batch_id>/export-excel', methods=['GET'])
@login_required
def export_batch_to_excel(batch_id):
    """Export batch sang Excel với đầy đủ thông tin KIE + 2 trường mới"""
    try:
        # Check access permission
        if not can_access_batch(db_manager, session['user_id'], session['user_role'], batch_id):
            return jsonify({'error': 'Access denied'}), 403
        
        # Get batch info
        batch_info = db_manager.get_batch_info(batch_id)
        if not batch_info:
            return jsonify({'error': 'Batch not found'}), 404
        
        # Get all documents in batch
        conn = db_manager.get_connection()
        cursor = conn.cursor()
        
        # ✅ THÊM kie_content_data VÀO SELECT
        cursor.execute('''
            SELECT d.id, d.filename, d.file_path, d.kie_header_data, d.kie_content_data,
                   d.status, d.created_at, d.processed_at,
                   d.document_original_type, d.document_note
            FROM documents d
            WHERE d.batch_id = ?
            ORDER BY d.filename
        ''', (batch_id,))
        
        documents = cursor.fetchall()
        conn.close()
        
        if not documents:
            return jsonify({'error': 'No documents found'}), 404
        
        # Create Excel workbook
        wb = Workbook()
        ws = wb.active
        ws.title = "Documents KIE"
        
        # ✅ DEFINE HEADERS - THÊM 2 COLUMNS MỚI
        headers = [
            'STT',
            'Tên file',
            'Đường dẫn file',
            'Loại văn bản',
            'Số văn bản',
            'Ký hiệu văn bản',
            'Số viết tay (Tờ số)',
            'Ngày ban hành',
            'Cơ quan ban hành',
            'Người ký',
            'Trích yếu',
            'Loại hình',        # ✅ MỚI
            'Ghi chú',          # ✅ MỚI
            'Trạng thái'
        ]
        
        # ✅ STYLE HEADER ROW (giữ nguyên)
        header_fill = PatternFill(start_color="4472C4", end_color="4472C4", fill_type="solid")
        header_font = Font(bold=True, color="FFFFFF", size=11)
        header_alignment = Alignment(horizontal="center", vertical="center", wrap_text=True)
        
        thin_border = Border(
            left=Side(style='thin'),
            right=Side(style='thin'),
            top=Side(style='thin'),
            bottom=Side(style='thin')
        )
        
        # Write headers
        for col_idx, header in enumerate(headers, start=1):
            cell = ws.cell(row=1, column=col_idx, value=header)
            cell.fill = header_fill
            cell.font = header_font
            cell.alignment = header_alignment
            cell.border = thin_border
        
        # ✅ SET COLUMN WIDTHS - THÊM 2 COLUMNS MỚI
        column_widths = {
            'A': 6,   # STT
            'B': 40,  # Tên file
            'C': 50,  # Đường dẫn file
            'D': 20,  # Loại văn bản
            'E': 15,  # Số văn bản
            'F': 15,  # Ký hiệu VB
            'G': 15,  # Tờ số
            'H': 15,  # Ngày ban hành
            'I': 30,  # Cơ quan ban hành
            'J': 25,  # Người ký
            'K': 50,  # Trích yếu
            'L': 15,  # ✅ Loại hình (MỚI)
            'M': 40,  # ✅ Ghi chú (MỚI)
            'N': 15   # Trạng thái
        }
        
        for col_letter, width in column_widths.items():
            ws.column_dimensions[col_letter].width = width
        
        # ✅ WRITE DATA ROWS
        row_idx = 2
        for stt, doc_row in enumerate(documents, start=1):
            doc_id = doc_row[0]
            filename = doc_row[1]
            file_path = doc_row[2]
            kie_header_data_json = doc_row[3]
            kie_content_data_json = doc_row[4]  # ✅ THÊM
            status = doc_row[5]
            document_original_type = doc_row[8] or 'Bản chính'  # ✅ MỚI (index thay đổi)
            document_note = doc_row[9] or ''                     # ✅ MỚI (index thay đổi)
            
            # Parse KIE header data
            kie_header_data = {}
            if kie_header_data_json:
                try:
                    kie_header_data = json.loads(kie_header_data_json)
                except:
                    pass
            
            # ✅ Parse KIE content data
            kie_content_data = {}
            if kie_content_data_json:
                try:
                    kie_content_data = json.loads(kie_content_data_json)
                except:
                    pass
            
            # ✅ Extract KIE fields - ƯU TIÊN HEADER, FALLBACK TO CONTENT
            document_type = kie_header_data.get('document_type', '') or kie_content_data.get('document_type', '')
            document_number = kie_header_data.get('document_number', '') or kie_content_data.get('document_number', '')
            document_symbol = kie_header_data.get('document_symbol', '') or kie_content_data.get('document_symbol', '')
            handwritten_number = kie_header_data.get('handwritten_number', '') or kie_content_data.get('handwritten_number', '')
            issue_date = kie_header_data.get('issue_date', '') or kie_content_data.get('issue_date', '')
            issuing_authority = kie_header_data.get('issuing_authority', '') or kie_content_data.get('issuing_authority', '')
            signer_name = kie_header_data.get('signer_name', '') or kie_content_data.get('signer_name', '')
            
            # ✅ QUAN TRỌNG: LẤY SUMMARY TỪ CẢ 2 NGUỒN
            summary = kie_header_data.get('summary', '') or kie_content_data.get('summary', '')
            
            # Write row data
            row_data = [
                stt,                        # STT
                filename,                   # Tên file
                file_path,                  # Đường dẫn
                document_type,              # Loại văn bản
                document_number,            # Số văn bản
                document_symbol,            # Ký hiệu
                handwritten_number,         # Tờ số
                issue_date,                 # Ngày ban hành
                issuing_authority,          # Cơ quan ban hành
                signer_name,                # Người ký
                summary,                    # ✅ Trích yếu (từ cả header và content)
                document_original_type,     # ✅ Loại hình (MỚI)
                document_note,              # ✅ Ghi chú (MỚI)
                status                      # Trạng thái
            ]
            
            for col_idx, value in enumerate(row_data, start=1):
                cell = ws.cell(row=row_idx, column=col_idx, value=value)
                cell.border = thin_border
                
                # Center align for STT and Status
                if col_idx == 1 or col_idx == 14:  # ✅ Cập nhật index status column
                    cell.alignment = Alignment(horizontal="center", vertical="top")
                else:
                    cell.alignment = Alignment(horizontal="left", vertical="top", wrap_text=True)
                
                # ✅ HIGHLIGHT "Bản sao" IN RED
                if col_idx == 12 and value == 'Bản sao':
                    cell.font = Font(color="FF0000", bold=True)
            
            row_idx += 1
        
        # ✅ FREEZE HEADER ROW
        ws.freeze_panes = 'A2'
        
        # ✅ ADD SUMMARY ROW
        summary_row = row_idx + 1
        ws.cell(row=summary_row, column=1, value="Tổng số:")
        ws.cell(row=summary_row, column=1).font = Font(bold=True)
        ws.cell(row=summary_row, column=2, value=f"{len(documents)} documents")
        
        # ✅ COUNT ORIGINAL VS COPY (index thay đổi do thêm kie_content_data)
        original_count = sum(1 for doc in documents if (doc[8] or 'Bản chính') == 'Bản chính')
        copy_count = len(documents) - original_count
        
        ws.cell(row=summary_row + 1, column=1, value="Bản chính:")
        ws.cell(row=summary_row + 1, column=2, value=f"{original_count} documents")
        ws.cell(row=summary_row + 2, column=1, value="Bản sao:")
        ws.cell(row=summary_row + 2, column=2, value=f"{copy_count} documents")
        ws.cell(row=summary_row + 2, column=2).font = Font(color="FF0000")
        
        # Save to bytes buffer
        output = io.BytesIO()
        wb.save(output)
        output.seek(0)
        
        # ✅ GENERATE FILENAME - GIỮ NGUYÊN TIẾNG VIỆT
        batch_name = batch_info.get('name', 'batch')
        timestamp = datetime.now().strftime("%Y%m%d_%H%M%S")
        
        safe_batch_name = "".join(c if c.isalnum() or c in (' ', '_', '-') or ord(c) > 127 else '_' for c in batch_name)
        
        filename = f"{safe_batch_name}_{timestamp}.xlsx"
        
        print(f"✅ Exporting to filename: {filename}")
        
        # ✅ CREATE RESPONSE WITH PROPER UTF-8 ENCODING
        content = output.getvalue()
        
        response = make_response(content)
        response.headers['Content-Type'] = 'application/vnd.openxmlformats-officedocument.spreadsheetml.sheet'
        
        import urllib.parse
        encoded_filename = urllib.parse.quote(filename.encode('utf-8'))
        
        response.headers['Content-Disposition'] = f"attachment; filename*=UTF-8''{encoded_filename}"
        response.headers['Content-Length'] = len(content)
        
        return response
        
    except Exception as e:
        print(f"❌ Error exporting batch to Excel: {str(e)}")
        import traceback
        traceback.print_exc()
        return jsonify({'error': str(e)}), 500

# ✅ THÊM ROUTE MỚI
@app.route('/batch/<batch_id>/documents')
@login_required
def batch_documents_page(batch_id):
    """Trang xem danh sách documents của batch"""
    try:
        # Check access permission
        if not can_access_batch(db_manager, session['user_id'], session['user_role'], batch_id):
            return redirect('/')
        
        # Get batch info
        batch_info = db_manager.get_batch_info(batch_id)
        if not batch_info:
            return redirect('/')
        
        return render_template('batch_documents.html', 
                             batch_id=batch_id,
                             batch_name=batch_info['name'])
    except Exception as e:
        print(f"❌ Error loading batch documents page: {e}")
        return redirect('/')
# ✅ THÊM ROUTE MỚI TRƯỚC if __name__ == '__main__':
@app.route('/uploads/<path:filename>')
@login_required
def serve_upload_file(filename):
    """Serve files từ uploads folder (PDF, images, etc.)"""
    try:
        upload_folder = app.config['UPLOAD_FOLDER']
        file_path = os.path.join(upload_folder, filename)
        
        # Security check: đảm bảo file nằm trong uploads folder
        if not os.path.abspath(file_path).startswith(os.path.abspath(upload_folder)):
            return jsonify({'error': 'Access denied'}), 403
        
        if not os.path.exists(file_path):
            return jsonify({'error': 'File not found'}), 404
        
        # ✅ DETECT FILE TYPE
        file_ext = os.path.splitext(filename)[1].lower()
        
        mimetype_map = {
            '.pdf': 'application/pdf',
            '.png': 'image/png',
            '.jpg': 'image/jpeg',
            '.jpeg': 'image/jpeg',
            '.bmp': 'image/bmp',
            '.tiff': 'image/tiff',
        }
        
        mimetype = mimetype_map.get(file_ext, 'application/octet-stream')
        
        return send_file(file_path, mimetype=mimetype)
        
    except Exception as e:
        print(f"❌ Error serving file {filename}: {e}")
        return jsonify({'error': str(e)}), 500    
if __name__ == '__main__':
    try:
        # Initialize database
        db_manager.init_database()
        print("📊 Database initialized")
        
        # Initialize KIE system
        if not init_kie_system():
            print("❌ Cannot start without KIE system")
            exit(1)
        
        # ✅ GET HOST AND PORT FROM .ENV
        host = os.getenv('FLASK_HOST', '0.0.0.0')
        port = int(os.getenv('FLASK_PORT', '5000'))
        waitress_threads = int(os.getenv('WAITRESS_THREADS', '8'))
        
        from waitress import serve
        print(f"🚀 Starting Waitress server ({waitress_threads} threads)")
        print(f"🌐 http://{host}:{port}")
        print(f"🤖 KIE system ready for processing")
        
        serve(
            app,
            host=host,
            port=port,
            threads=waitress_threads,
            channel_timeout=300,
            recv_bytes=65536,
            url_scheme='http'
        )
        
    except KeyboardInterrupt:
        print("\n👋 Application stopped by user")
    except Exception as e:
        print(f"❌ Failed to start application: {e}")
        exit(1)