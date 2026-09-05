import os
import uuid
from datetime import datetime
from flask import Blueprint, request, jsonify
from auth import login_required, get_current_user
from external_api import get_external_api

# Create blueprint
external_bp = Blueprint('external', __name__)
EXTERNAL_FILE_SERVER_PATH = os.getenv('EXTERNAL_FILE_SERVER_PATH', '\\\\103.154.62.17\\UploadFile')

@external_bp.route('/api/external/projects', methods=['GET'])
@login_required
def get_external_projects():
    """Get list of external projects"""
    try:
        external_api = get_external_api()
        projects = external_api.get_projects()
        
        return jsonify({
            'success': True,
            'projects': projects
        })
        
    except Exception as e:
        print(f"❌ Error getting external projects: {e}")
        return jsonify({
            'success': False,
            'error': str(e)
        }), 500

@external_bp.route('/api/external/projects/<project_id>/files', methods=['GET'])
@login_required
def get_external_project_files(project_id):
    """Get files for a specific external project"""
    try:
        external_api = get_external_api()
        files = external_api.get_project_files(project_id)
        
        # Group files by directory path for folder tree structure
        folder_tree = build_folder_tree(files)
        
        return jsonify({
            'success': True,
            'files': files,
            'folder_tree': folder_tree,
            'project_id': project_id
        })
        
    except Exception as e:
        print(f"❌ Error getting external project files: {e}")
        return jsonify({
            'success': False,
            'error': str(e)
        }), 500
def generate_external_batch_name(project_name, department_name='', folder_path=''):
    """Generate descriptive batch name with full path"""
    
    # Bắt đầu với tên dự án
    name_parts = [project_name]
    
    # Thêm phòng ban nếu có
    if department_name:
        name_parts.append(department_name)
    
    # ✅ XỬ LÝ FOLDER PATH ĐỂ HIỂN THỊ FULL HIERARCHY
    if folder_path:
        # Split folder path và thêm tất cả các level
        folder_parts = [part.strip() for part in folder_path.split('/') if part.strip()]
        
        if folder_parts:
            # Thêm tất cả folder parts để có full path
            name_parts.extend(folder_parts)
        else:
            # Nếu có folder_path nhưng rỗng, là root của department
            if department_name:
                name_parts.append("Root")
    else:
        # Không có folder_path
        if department_name:
            name_parts.append("Root")
        else:
            # Toàn bộ dự án
            name_parts.append("Toàn bộ dự án")
    
    # Kết hợp với separator và timestamp
    batch_name = "_".join(name_parts)
    timestamp = datetime.now().strftime('%Y%m%d_%H%M%S')
    
    return f"{batch_name} - {timestamp}"
@external_bp.route('/api/external/create-batch', methods=['POST'])
@login_required
def create_batch_from_external():
    """Create batch from external project files"""
    try:
        from app import db_manager
        
        user = get_current_user()
        data = request.get_json()
        
        project_id = data.get('project_id')
        project_name = data.get('project_name')
        department_id = data.get('department_id', '')
        department_name = data.get('department_name', '')
        selected_files = data.get('selected_files', [])
        batch_name = data.get('batch_name')
        folder_path = data.get('folder_path', '')
        
        if not project_id or not selected_files:
            return jsonify({
                'success': False,
                'error': 'Thiếu thông tin project_id hoặc selected_files'
            }), 400
        
        # Generate batch name
        if not batch_name:
            batch_name = generate_external_batch_name(
                project_name, 
                department_name, 
                folder_path
            )
        
        # ✅ CREATE EXTERNAL BATCH
        batch_id = str(uuid.uuid4())
        db_manager.create_batch(
            batch_id=batch_id,
            batch_name=batch_name,
            status='uploaded',
            created_by=user['id'],
            batch_type='external'  # ✅ ĐÁNH DẤU LÀ EXTERNAL
        )
        
        # Create documents from selected files (existing code)
        created_documents = 0
        failed_files = []
        
        for file_info in selected_files:
            try:
                doc_id = str(uuid.uuid4())
                file_path = file_info.get('duongDanFile', '')
                filename = file_info.get('tenFile', '')
                van_ban_id = file_info.get('vanBanId', '')
                
                if not os.path.exists(file_path):
                    failed_files.append({
                        'filename': filename,
                        'error': f'File không tồn tại: {file_path}'
                    })
                    continue
                
                # Create document record - BỎ vanBanId vì function chưa support
                db_manager.create_document(
                    doc_id=doc_id,
                    batch_id=batch_id,
                    filename=filename,
                    file_path=file_path,
                    created_by=user['id']
                )
                
                # Store external metadata
                external_metadata = {
                    'external_source': True,
                    'project_id': project_id,
                    'project_name': project_name,
                    'department_id': department_id,
                    'department_name': department_name,
                    'folder_path': folder_path,
                    'server_path': file_path,
                    'van_ban_id': van_ban_id
                }
                
                db_manager.update_document(
                    doc_id,
                    kie_header_data=external_metadata
                )
                
                created_documents += 1
                
            except Exception as e:
                print(f"❌ Error creating document for {filename}: {e}")
                failed_files.append({
                    'filename': filename,
                    'error': str(e)
                })
        
        # Update batch totals
        db_manager.update_batch(
            batch_id,
            total_files=created_documents
        )
        
        response_data = {
            'success': True,
            'batch_id': batch_id,
            'batch_name': batch_name,
            'batch_type': 'external',  # ✅ RETURN BATCH TYPE
            'created_documents': created_documents,
            'failed_files': failed_files,
            'message': f'Đã tạo external batch với {created_documents} files'
        }
        
        if failed_files:
            response_data['warning'] = f'{len(failed_files)} files không thể tạo'
        
        return jsonify(response_data)
        
    except Exception as e:
        print(f"❌ Error creating batch from external: {e}")
        return jsonify({
            'success': False,
            'error': str(e)
        }), 500

def build_folder_tree(files):
    """Build folder tree structure from file paths"""
    tree = {}
    
    for file_info in files:
        path = file_info.get('duongDanFile', '')
        filename = file_info.get('tenFile', '')
        
        if not path:
            continue
        
        # Split path into parts (handle both / and \ separators)
        path_parts = path.replace('\\', '/').split('/')
        
        # Remove filename from path parts
        if filename in path_parts:
            path_parts.remove(filename)
        
        # Build tree structure
        current_level = tree
        current_path = ""
        
        for part in path_parts:
            if not part:
                continue
                
            current_path = f"{current_path}/{part}" if current_path else part
            
            if part not in current_level:
                current_level[part] = {
                    'folders': {},
                    'files': [],
                    'path': current_path,
                    'name': part
                }
            
            current_level = current_level[part]['folders']
    
    return tree

@external_bp.route('/api/external/projects/<project_id>/departments', methods=['GET'])
@login_required
def get_external_project_departments(project_id):
    """Get departments (phong ban) for a specific external project"""
    try:
        external_api = get_external_api()
        projects = external_api.get_projects()
        
        # Find the specific project
        project = next((p for p in projects if p['id'] == project_id), None)
        if not project:
            return jsonify({
                'success': False,
                'error': 'Project not found'
            }), 404
        
        # Get departments from phongBanDTOs
        departments = project.get('phongBanDTOs', [])
        
        return jsonify({
            'success': True,
            'departments': departments,
            'project_id': project_id,
            'project_name': project.get('tenDuAn', '')
        })
        
    except Exception as e:
        print(f"❌ Error getting project departments: {e}")
        return jsonify({
            'success': False,
            'error': str(e)
        }), 500

@external_bp.route('/api/external/folder-contents', methods=['POST'])
@login_required  
def get_folder_contents():
    """Get contents of a specific folder path - direct from project"""
    try:
        data = request.get_json()
        project_id = data.get('project_id')
        folder_path = data.get('folder_path', '')
        
        if not project_id:
            return jsonify({
                'success': False,
                'error': 'Missing project_id'
            }), 400
        
        # ✅ GET FILES FROM EXTERNAL API
        try:
            external_api = get_external_api()
            all_project_files = external_api.get_project_files(project_id)
            print(f"🔍 Retrieved {len(all_project_files)} files from VanBan API")
        except Exception as e:
            print(f"❌ Error getting files from external API: {e}")
            all_project_files = []
        
        # ✅ BUILD BASE PATH - DIRECTLY FROM PROJECT
        full_path = os.path.join(EXTERNAL_FILE_SERVER_PATH, project_id)
        
        if folder_path:
            full_path = os.path.join(full_path, folder_path.replace('/', '\\'))
        
        print(f"🗂️ Browsing project folder: {full_path}")
        
        # Check if path exists
        if not os.path.exists(full_path):
            return jsonify({
                'success': False,
                'error': f'Đường dẫn không tồn tại: {full_path}'
            }), 404
        
        # ✅ CREATE MAP FROM FILE PATH TO idVanBan
        file_path_to_info = {}
        for file_info in all_project_files:
            file_path = file_info.get('duongDanFile', '')
            if file_path:
                id_van_ban = file_info.get('idVanBan', '')
                file_path_to_info[file_path] = {
                    'vanBanId': str(id_van_ban) if id_van_ban else '',
                    'idVanBan': id_van_ban,
                    'idFile': file_info.get('idFile', ''),
                    'idDuAn': file_info.get('idDuAn', project_id)
                }
        
        # Get folder contents
        folder_files = []
        subfolders = []
        
        try:
            for item in os.listdir(full_path):
                item_path = os.path.join(full_path, item)
                
                if os.path.isdir(item_path):
                    # It's a subfolder
                    subfolders.append(item)
                elif os.path.isfile(item_path):
                    # It's a file
                    if item.lower().endswith('.pdf'):
                        file_size = os.path.getsize(item_path)
                        file_modified = datetime.fromtimestamp(os.path.getmtime(item_path))
                        
                        # ✅ GET idVanBan FROM MAP
                        file_info = file_path_to_info.get(item_path, {})
                        van_ban_id = file_info.get('vanBanId', '')
                        
                        # ✅ DETERMINE DEPARTMENT FROM PATH
                        # Extract department from relative path structure: project/department/...
                        try:
                            relative_from_project = os.path.relpath(item_path, os.path.join(EXTERNAL_FILE_SERVER_PATH, project_id))
                            path_parts = relative_from_project.split(os.sep)
                            department_id = path_parts[0] if len(path_parts) > 1 else ''
                        except:
                            department_id = ''
                        
                        folder_files.append({
                            'idFile': file_info.get('idFile', f"{project_id}_{item}"),
                            'tenFile': item,
                            'duongDanFile': item_path,
                            'fileSize': file_size,
                            'lastModified': file_modified.isoformat(),
                            'department_id': department_id,  # ✅ AUTO-DETECTED FROM PATH
                            'vanBanId': van_ban_id,
                            'idVanBan': file_info.get('idVanBan', ''),
                            'idDuAn': file_info.get('idDuAn', project_id)
                        })
                        
                        print(f"✅ File: {item} -> idVanBan: {file_info.get('idVanBan', 'N/A')} -> dept: {department_id}")
        
        except PermissionError:
            return jsonify({
                'success': False,
                'error': 'Không có quyền truy cập thư mục'
            }), 403
        
        return jsonify({
            'success': True,
            'files': folder_files,
            'subfolders': sorted(subfolders),
            'folder_path': folder_path,
            'full_path': full_path,
            'total_files': len(folder_files),
            'total_folders': len(subfolders)
        })
        
    except Exception as e:
        print(f"❌ Error getting folder contents: {e}")
        return jsonify({
            'success': False,
            'error': str(e)
        }), 500
@external_bp.route('/api/external/create-batch-from-folder', methods=['POST'])
@login_required
def create_batch_from_folder():
    """Create batch from entire folder/project automatically"""
    try:
        from app import db_manager
        
        user = get_current_user()
        data = request.get_json()
        
        project_id = data.get('project_id')
        project_name = data.get('project_name')
        folder_path = data.get('folder_path', '')
        batch_name = data.get('batch_name')
        
        if not project_id:
            return jsonify({
                'success': False,
                'error': 'Thiếu thông tin project_id'
            }), 400
        
        # ✅ GET ALL FILES IN FOLDER (NO DEPARTMENT FILTER)
        all_files = get_all_files_in_folder(project_id, '', folder_path)
        
        if not all_files:
            return jsonify({
                'success': False,
                'error': 'Không tìm thấy file PDF nào trong thư mục này'
            }), 400
        
        # ✅ GENERATE BATCH NAME FROM PROJECT + FOLDER STRUCTURE
        if not batch_name:
            if folder_path:
                folder_parts = [part.strip() for part in folder_path.split('/') if part.strip()]
                batch_name = f"{project_name}_{'/'.join(folder_parts)} - {datetime.now().strftime('%Y%m%d_%H%M%S')}"
            else:
                batch_name = f"{project_name}_Toàn bộ dự án - {datetime.now().strftime('%Y%m%d_%H%M%S')}"
        
        # Create batch
        batch_id = str(uuid.uuid4())
        db_manager.create_batch(
            batch_id=batch_id,
            batch_name=batch_name,
            status='uploaded',
            created_by=user['id'],
            batch_type='external' 
        )
        
        # Create documents from all files
        created_documents = 0
        failed_files = []
        
        for file_info in all_files:
            try:
                doc_id = str(uuid.uuid4())
                file_path = file_info.get('duongDanFile', '')
                filename = file_info.get('tenFile', '')
                van_ban_id = file_info.get('vanBanId', '')
                department_id = file_info.get('department_id', '')  # ✅ AUTO-DETECTED
                
                # Validate file path exists
                if not os.path.exists(file_path):
                    failed_files.append({
                        'filename': filename,
                        'error': f'File không tồn tại: {file_path}'
                    })
                    continue
                
                # Create document record
                db_manager.create_document(
                    doc_id=doc_id,
                    batch_id=batch_id,
                    filename=filename,
                    file_path=file_path,
                    created_by=user['id'],
                    van_ban_id=van_ban_id
                )
                
                # Store external metadata
                external_metadata = {
                    'external_source': True,
                    'project_id': project_id,
                    'project_name': project_name,
                    'department_id': department_id,  # ✅ AUTO-DETECTED FROM PATH
                    'folder_path': folder_path,
                    'server_path': file_path,
                    'auto_batch': True,
                    'van_ban_id': van_ban_id 
                }
                
                db_manager.update_document(
                    doc_id,
                    kie_header_data=external_metadata
                )
                
                created_documents += 1
                
            except Exception as e:
                print(f"❌ Error creating document for {filename}: {e}")
                failed_files.append({
                    'filename': filename,
                    'error': str(e)
                })
        
        # Update batch totals
        db_manager.update_batch(
            batch_id,
            total_files=created_documents
        )
        
        response_data = {
            'success': True,
            'batch_id': batch_id,
            'batch_name': batch_name,
            'created_documents': created_documents,
            'failed_files': failed_files,
            'total_scanned_files': len(all_files),
            'message': f'Đã tạo batch từ {len(all_files)} files với {created_documents} thành công'
        }
        
        if failed_files:
            response_data['warning'] = f'{len(failed_files)} files không thể tạo'
        
        return jsonify(response_data)
        
    except Exception as e:
        print(f"❌ Error creating batch from folder: {e}")
        return jsonify({
            'success': False,
            'error': str(e)
        }), 500

def get_all_files_in_folder(project_id, department_id='', folder_path=''):
    """Recursively get all PDF files in folder - direct from project tree"""
    
    try:
        external_api = get_external_api()
        all_project_files = external_api.get_project_files(project_id)
        print(f"🔍 Retrieved {len(all_project_files)} files from VanBan API")
    except Exception as e:
        print(f"❌ Error getting files from external API: {e}")
        return []
    
    # ✅ BUILD TARGET PATH - DIRECT FROM PROJECT
    target_path = os.path.join(EXTERNAL_FILE_SERVER_PATH, project_id)
    
    if folder_path:
        target_path = os.path.join(target_path, folder_path.replace('/', '\\'))
    
    print(f"🔍 Target path for filtering: {target_path}")
    
    filtered_files = []
    
    # ✅ FILTER FILES BY TARGET PATH
    for file_info in all_project_files:
        file_path = file_info.get('duongDanFile', '')
        
        if not file_path:
            continue
        
        # Normalize path for comparison
        normalized_file_path = file_path.replace('\\', '/').lower()
        normalized_target_path = target_path.replace('\\', '/').lower()
        
        # Check if file is in target path or subfolders
        if normalized_file_path.startswith(normalized_target_path):
            # Check if file exists on file system and is PDF
            if os.path.exists(file_path) and file_path.lower().endswith('.pdf'):
                try:
                    file_size = os.path.getsize(file_path)
                    file_modified = datetime.fromtimestamp(os.path.getmtime(file_path))
                    
                    # ✅ AUTO-DETECT DEPARTMENT FROM PATH STRUCTURE
                    try:
                        relative_from_project = os.path.relpath(file_path, os.path.join(EXTERNAL_FILE_SERVER_PATH, project_id))
                        path_parts = relative_from_project.split(os.sep)
                        auto_department_id = path_parts[0] if len(path_parts) > 1 else ''
                        
                        # Generate relative path from target folder
                        relative_dir = os.path.dirname(file_path)
                        relative_path = os.path.relpath(relative_dir, target_path)
                        if relative_path == '.':
                            relative_path = ''
                    except:
                        auto_department_id = ''
                        relative_path = ''
                    
                    id_van_ban = file_info.get('idVanBan', '')
                    
                    filtered_files.append({
                        'idFile': file_info.get('idFile', ''),
                        'tenFile': file_info.get('tenFile', ''),
                        'duongDanFile': file_path,
                        'fileSize': file_size,
                        'lastModified': file_modified.isoformat(),
                        'department_id': auto_department_id,  # ✅ AUTO-DETECTED
                        'vanBanId': str(id_van_ban) if id_van_ban else '',
                        'idVanBan': id_van_ban,
                        'idDuAn': file_info.get('idDuAn', project_id),
                        'relative_path': relative_path
                    })
                    
                    print(f"✅ Found file: {file_info.get('tenFile', '')} -> dept: {auto_department_id} -> idVanBan: {id_van_ban}")
                    
                except Exception as e:
                    print(f"⚠️ Error getting file info for {file_path}: {e}")
                    continue
    
    print(f"✅ Found {len(filtered_files)} PDF files in target folder")
    return filtered_files
@external_bp.route('/api/external/count-files-in-folder', methods=['POST'])
@login_required  
def count_files_in_folder():
    """Count total PDF files in folder and subfolders"""
    try:
        data = request.get_json()
        project_id = data.get('project_id')
        folder_path = data.get('folder_path', '')
        
        if not project_id:
            return jsonify({
                'success': False,
                'error': 'Missing project_id'
            }), 400
        
        # ✅ NO DEPARTMENT FILTER - DIRECT FROM PROJECT
        all_files = get_all_files_in_folder(project_id, '', folder_path)
        
        # Group by subfolder for summary
        folder_summary = {}
        for file_info in all_files:
            rel_path = file_info.get('relative_path', '')
            if not rel_path:
                rel_path = 'Root'
            
            if rel_path not in folder_summary:
                folder_summary[rel_path] = 0
            folder_summary[rel_path] += 1
        
        return jsonify({
            'success': True,
            'total_files': len(all_files),
            'folder_summary': folder_summary,
            'project_id': project_id,
            'folder_path': folder_path
        })
        
    except Exception as e:
        print(f"❌ Error counting files in folder: {e}")
        return jsonify({
            'success': False,
            'error': str(e)
        }), 500