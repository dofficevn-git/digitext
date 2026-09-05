from datetime import datetime
from flask import Blueprint, request, jsonify, session, send_from_directory, current_app
from auth import login_required, role_required, get_current_user
from work_package_manager import WorkPackageManager
from database import DatabaseManager  # ✅ THÊM IMPORT
import os

# Tạo blueprint
work_package_bp = Blueprint('work_packages', __name__)

def get_wp_manager():
    """Get WorkPackageManager instance với db_manager từ app context"""
    try:
        # ✅ LẤY DB_MANAGER TỪ APP CONFIG
        db_manager = current_app.config.get('db_manager')
        if not db_manager:
            # Fallback: tạo mới với config đúng
            db_config = {
                'server': os.getenv('DB_SERVER', '127.0.0.1,1433'),
                'database': os.getenv('DB_DATABASE', 'kie'),
                'username': os.getenv('DB_USERNAME', 'sa'),
                'password': os.getenv('DB_PASSWORD', ''),
                'driver': os.getenv('DB_DRIVER', '{ODBC Driver 17 for SQL Server}')
            }
            db_manager = DatabaseManager(db_config)
        
        return WorkPackageManager(db_manager)
    except Exception as e:
        print(f"❌ Error getting WorkPackageManager: {e}")
        raise

# ===== SERVE HTML PAGES =====
@work_package_bp.route('/manage-packages-page')
@login_required
@role_required('customer')
def serve_manage_packages():
    """Serve trang quản lý gói cho customer"""
    return send_from_directory('templates', 'manage_packages.html')

@work_package_bp.route('/create-package-page')
@login_required
@role_required('customer')
def serve_create_package():
    """Serve trang tạo gói mới"""
    return send_from_directory('templates', 'create_package.html')

@work_package_bp.route('/package/<package_id>/detail-page')
@login_required
def serve_package_detail(package_id):
    """Serve trang chi tiết gói"""
    return send_from_directory('templates', 'package_detail.html')

@work_package_bp.route('/package/<package_id>/review-page')
@login_required
@role_required('customer')
def serve_package_review(package_id):
    """Serve trang review gói"""
    return send_from_directory('templates', 'package_review.html')

@work_package_bp.route('/work-packages-page')
@login_required
@role_required('cooperator')
def serve_work_packages():
    """Serve trang gói công việc cho cooperator"""
    return send_from_directory('templates', 'work_packages.html')

@work_package_bp.route('/work-item/<item_id>/edit-page')
@login_required
def serve_work_item_editor(item_id):
    """Serve trang editor mảnh"""
    return send_from_directory('templates', 'work_item_editor.html')

@work_package_bp.route('/customer-statistics-page')
@login_required
@role_required('customer')
def serve_customer_statistics():
    """Serve trang thống kê cho customer"""
    return send_from_directory('templates', 'customer_statistics.html')

@work_package_bp.route('/cooperator-statistics-page')
@login_required
@role_required('cooperator')
def serve_cooperator_statistics():
    """Serve trang thống kê cho cooperator"""
    return send_from_directory('templates', 'cooperator_statistics.html')

# ===== API ENDPOINTS =====

@work_package_bp.route('/api/available-pieces-count')
@login_required
@role_required('customer')
def get_available_pieces_count():
    """API lấy số lượng mảnh có thể assign"""
    try:
        user = get_current_user()
        wp_manager = get_wp_manager()  # ✅ SỬ DỤNG FUNCTION
        count = wp_manager.db.get_available_pieces_count(user['id'])
        
        return jsonify({
            'success': True,
            'count': count
        })
        
    except Exception as e:
        print(f"❌ Error in get_available_pieces_count: {e}")
        import traceback
        traceback.print_exc()
        return jsonify({
            'success': False,
            'error': str(e)
        }), 500

@work_package_bp.route('/api/cooperators')
@login_required
@role_required('customer')
def get_cooperators():
    """API lấy danh sách cooperators của customer"""
    try:
        user = get_current_user()
        wp_manager = get_wp_manager()  # ✅ SỬ DỤNG FUNCTION
        cooperators = wp_manager.get_cooperators_for_customer(user['id'])
        
        return jsonify({
            'success': True,
            'cooperators': cooperators
        })
        
    except Exception as e:
        print(f"❌ Error in get_cooperators: {e}")
        import traceback
        traceback.print_exc()
        return jsonify({
            'success': False,
            'error': str(e)
        }), 500

@work_package_bp.route('/api/packages', methods=['GET', 'POST'])
@login_required
def handle_packages():
    """API handle packages - GET: list, POST: create"""
    
    if request.method == 'GET':
        try:
            user = get_current_user()
            page = int(request.args.get('page', 1))
            search = request.args.get('search', '')
            status_filter = request.args.get('status_filter', '')  # ✅ THÊM STATUS FILTER
            
            wp_manager = get_wp_manager()
            
            if user['role'] == 'customer':
                result = wp_manager.get_customer_packages(
                    user['id'], 
                    search_term=search, 
                    page=page,
                    status_filter=status_filter  # ✅ THÊM PARAMETER
                )
            elif user['role'] == 'cooperator':
                result = wp_manager.get_cooperator_packages(
                    user['id'], 
                    search_term=search, 
                    page=page,
                    status_filter=status_filter  # ✅ THÊM PARAMETER
                )
            else:
                return jsonify({'success': False, 'error': 'Invalid role'}), 403
            
            return jsonify({
                'success': True,
                **result
            })
            
        except Exception as e:
            print(f"❌ Error in handle_packages GET: {e}")
            import traceback
            traceback.print_exc()
            return jsonify({
                'success': False,
                'error': str(e)
            }), 500
    
    elif request.method == 'POST':
        try:
            user = get_current_user()
            
            if user['role'] != 'customer':
                return jsonify({'success': False, 'error': 'Only customers can create packages'}), 403
            
            data = request.get_json()
            
            # Validate input
            if not data.get('name') or not data.get('name').strip():
                return jsonify({'success': False, 'error': 'Tên gói không được để trống'}), 400
            
            # Hỗ trợ cả cooperator_ids (list) và cooperator_id (string) backward compatible
            cooperator_ids = data.get('cooperator_ids', [])
            if not cooperator_ids and data.get('cooperator_id'):
                cooperator_ids = [data['cooperator_id']]
            
            if not cooperator_ids:
                return jsonify({'success': False, 'error': 'Phải chọn ít nhất 1 cooperator'}), 400
                
            if not data.get('item_count') or int(data.get('item_count')) <= 0:
                return jsonify({'success': False, 'error': 'Số lượng mảnh phải lớn hơn 0'}), 400
            
            # ✅ THÊM BATCH FILTER VÀ FRAGMENTATION MODE
            batch_mode = data.get('batch_mode', 'all')
            selected_batch_ids = data.get('selected_batch_ids', [])
            is_fragmented = data.get('is_fragmented', True)  # ✅ MẶC ĐỊNH LÀ PHÂN MẢNH
            
            # Validate batch selection
            if batch_mode == 'specific' and not selected_batch_ids:
                return jsonify({'success': False, 'error': 'Vui lòng chọn ít nhất 1 batch'}), 400
            
            wp_manager = get_wp_manager()
            
            # ✅ TẠO GÓI VỚI NHIỀU COOPERATORS
            if batch_mode == 'specific':
                result = wp_manager.create_package_with_batch_filter(
                    name=data['name'].strip(),
                    customer_id=user['id'],
                    cooperator_ids=cooperator_ids,
                    item_count=int(data['item_count']),
                    batch_filter=selected_batch_ids,
                    is_fragmented=is_fragmented
                )
            else:
                result = wp_manager.create_package(
                    name=data['name'].strip(),
                    customer_id=user['id'],
                    cooperator_ids=cooperator_ids,
                    item_count=int(data['item_count']),
                    is_fragmented=is_fragmented
                )
            
            return jsonify(result)
            
        except ValueError as e:
            return jsonify({'success': False, 'error': str(e)}), 400
        except PermissionError as e:
            return jsonify({'success': False, 'error': str(e)}), 403
        except Exception as e:
            print(f"❌ Error in handle_packages POST: {e}")
            import traceback
            traceback.print_exc()
            return jsonify({'success': False, 'error': str(e)}), 500

@work_package_bp.route('/api/packages/<package_id>', methods=['GET', 'DELETE'])
@login_required
def handle_package_detail(package_id):
    """API handle package detail - GET: detail, DELETE: delete"""
    
    if request.method == 'GET':
        try:
            user = get_current_user()
            wp_manager = get_wp_manager()  # ✅ SỬ DỤNG FUNCTION
            package = wp_manager.get_package_detail(package_id, user['id'], user['role'])
            
            return jsonify({
                'success': True,
                'package': package
            })
            
        except ValueError as e:
            return jsonify({'success': False, 'error': str(e)}), 404
        except PermissionError as e:
            return jsonify({'success': False, 'error': str(e)}), 403
        except Exception as e:
            print(f"❌ Error in handle_package_detail GET: {e}")
            import traceback
            traceback.print_exc()
            return jsonify({'success': False, 'error': str(e)}), 500
    
    elif request.method == 'DELETE':
        try:
            user = get_current_user()
            
            if user['role'] != 'customer':
                return jsonify({'success': False, 'error': 'Only customers can delete packages'}), 403
            
            wp_manager = get_wp_manager()  # ✅ SỬ DỤNG FUNCTION
            result = wp_manager.delete_package(package_id, user['id'])
            
            return jsonify(result)
            
        except ValueError as e:
            return jsonify({'success': False, 'error': str(e)}), 404
        except PermissionError as e:
            return jsonify({'success': False, 'error': str(e)}), 403
        except Exception as e:
            print(f"❌ Error in handle_package_detail DELETE: {e}")
            import traceback
            traceback.print_exc()
            return jsonify({'success': False, 'error': str(e)}), 500

@work_package_bp.route('/api/packages/<package_id>/cooperators', methods=['GET', 'POST', 'DELETE'])
@login_required
@role_required('customer')
def manage_package_cooperators(package_id):
    """API quản lý cooperators của package"""
    user = get_current_user()
    wp_manager = get_wp_manager()
    conn = wp_manager.db.get_connection()
    cursor = conn.cursor()
    
    try:
        # Verify package belongs to customer
        cursor.execute('SELECT customer_id FROM work_packages WHERE id = ?', (package_id,))
        pkg = cursor.fetchone()
        if not pkg:
            conn.close()
            return jsonify({'success': False, 'error': 'Gói không tồn tại'}), 404
        if pkg[0] != user['id']:
            conn.close()
            return jsonify({'success': False, 'error': 'Bạn không có quyền'}), 403
        
        if request.method == 'GET':
            # Lấy cooperators hiện tại của package
            cursor.execute('''
                SELECT u.id, u.full_name, u.username
                FROM work_package_cooperators wpc
                JOIN users u ON wpc.cooperator_id = u.id
                WHERE wpc.package_id = ?
                ORDER BY u.full_name
            ''', (package_id,))
            current = [{'id': r[0], 'full_name': r[1], 'username': r[2]} for r in cursor.fetchall()]
            
            # Lấy tất cả cooperators của customer (chưa có trong package)
            cursor.execute('''
                SELECT u.id, u.full_name, u.username
                FROM user_relationships ur
                JOIN users u ON ur.cooperator_id = u.id
                WHERE ur.customer_id = ? AND u.status = 'active' AND u.role = 'cooperator'
                  AND u.id NOT IN (
                    SELECT cooperator_id FROM work_package_cooperators WHERE package_id = ?
                  )
                ORDER BY u.full_name
            ''', (user['id'], package_id))
            available = [{'id': r[0], 'full_name': r[1], 'username': r[2]} for r in cursor.fetchall()]
            
            conn.close()
            return jsonify({'success': True, 'current': current, 'available': available})
        
        elif request.method == 'POST':
            data = request.get_json()
            cooperator_ids = data.get('cooperator_ids', [])
            if not cooperator_ids:
                conn.close()
                return jsonify({'success': False, 'error': 'Chọn ít nhất 1 CTV'}), 400
            
            added = 0
            for coop_id in cooperator_ids:
                # Verify cooperator belongs to customer
                cursor.execute(
                    'SELECT 1 FROM user_relationships WHERE customer_id = ? AND cooperator_id = ?',
                    (user['id'], coop_id)
                )
                if not cursor.fetchone():
                    continue
                # Check not already in package
                cursor.execute(
                    'SELECT 1 FROM work_package_cooperators WHERE package_id = ? AND cooperator_id = ?',
                    (package_id, coop_id)
                )
                if cursor.fetchone():
                    continue
                import uuid as uuid_mod
                cursor.execute(
                    'INSERT INTO work_package_cooperators (id, package_id, cooperator_id) VALUES (?, ?, ?)',
                    (str(uuid_mod.uuid4()), package_id, coop_id)
                )
                added += 1
            
            conn.commit()
            conn.close()
            return jsonify({'success': True, 'message': f'Đã thêm {added} CTV vào gói'})
        
        elif request.method == 'DELETE':
            data = request.get_json()
            cooperator_id = data.get('cooperator_id', '')
            if not cooperator_id:
                conn.close()
                return jsonify({'success': False, 'error': 'Thiếu cooperator_id'}), 400
            
            cursor.execute(
                'DELETE FROM work_package_cooperators WHERE package_id = ? AND cooperator_id = ?',
                (package_id, cooperator_id)
            )
            conn.commit()
            conn.close()
            return jsonify({'success': True, 'message': 'Đã xóa CTV khỏi gói'})
    
    except Exception as e:
        conn.close()
        print(f"❌ Error in manage_package_cooperators: {e}")
        import traceback
        traceback.print_exc()
        return jsonify({'success': False, 'error': str(e)}), 500

@work_package_bp.route('/api/packages/<package_id>/items')
@login_required
def get_package_items(package_id):
    """API lấy danh sách mảnh trong gói với status filter"""
    try:
        user = get_current_user()
        page = int(request.args.get('page', 1))
        search = request.args.get('search', '')
        status_filter = request.args.get('status_filter', '')  # ✅ THÊM STATUS FILTER
        
        wp_manager = get_wp_manager()
        
        # Kiểm tra quyền truy cập package trước
        wp_manager.get_package_detail(package_id, user['id'], user['role'])
        
        # ✅ CTV + pending/rejected → chỉ trả 1 mảnh + lock
        if user['role'] == 'cooperator' and (not status_filter or status_filter in ('pending', 'rejected', 'pending,rejected')):
            result = wp_manager.get_next_work_item_for_cooperator(
                package_id, user['id']
            )
            return jsonify({
                'success': True,
                **result
            })
        
        # Lấy items với status filter (customer/admin hoặc CTV xem mảnh đã xong)
        result = wp_manager.get_package_items(
            package_id, 
            search_term=search, 
            page=page,
            status_filter=status_filter  # ✅ THÊM PARAMETER
        )
        
        return jsonify({
            'success': True,
            **result
        })
        
    except ValueError as e:
        return jsonify({'success': False, 'error': str(e)}), 404
    except PermissionError as e:
        return jsonify({'success': False, 'error': str(e)}), 403
    except Exception as e:
        print(f"❌ Error in get_package_items: {e}")
        import traceback
        traceback.print_exc()
        return jsonify({'success': False, 'error': str(e)}), 500

@work_package_bp.route('/api/rejected-items')
@login_required
@role_required('cooperator')
def get_rejected_items():
    """API lấy danh sách mảnh bị rejected của cooperator"""
    try:
        user = get_current_user()
        page = int(request.args.get('page', 1))
        search = request.args.get('search', '')
        
        wp_manager = get_wp_manager()  # ✅ SỬ DỤNG FUNCTION
        result = wp_manager.get_rejected_items(
            user['id'], search_term=search, page=page
        )
        
        return jsonify({
            'success': True,
            **result
        })
        
    except Exception as e:
        print(f"❌ Error in get_rejected_items: {e}")
        import traceback
        traceback.print_exc()
        return jsonify({
            'success': False,
            'error': str(e)
        }), 500

@work_package_bp.route('/api/work-items/<item_id>')
@login_required
def get_work_item_detail(item_id):
    """API lấy chi tiết work item"""
    try:
        user = get_current_user()
        wp_manager = get_wp_manager()  # ✅ SỬ DỤNG FUNCTION
        item = wp_manager.get_work_item_detail(item_id, user['id'], user['role'])
        
        return jsonify({
            'success': True,
            'item': item
        })
        
    except ValueError as e:
        return jsonify({'success': False, 'error': str(e)}), 404
    except PermissionError as e:
        return jsonify({'success': False, 'error': str(e)}), 403
    except Exception as e:
        print(f"❌ Error in get_work_item_detail: {e}")
        import traceback
        traceback.print_exc()
        return jsonify({'success': False, 'error': str(e)}), 500

@work_package_bp.route('/api/work-items/<item_id>/navigation')
@login_required
def get_item_navigation(item_id):
    """API lấy navigation items trong package"""
    try:
        user = get_current_user()
        wp_manager = get_wp_manager()  # ✅ SỬ DỤNG FUNCTION
        
        # Lấy thông tin item để có package_id
        item = wp_manager.get_work_item_detail(item_id, user['id'], user['role'])
        
        # Lấy navigation
        navigation = wp_manager.get_package_navigation_items(
            item['package_id'], item_id
        )
        
        return jsonify({
            'success': True,
            'navigation': navigation
        })
        
    except ValueError as e:
        return jsonify({'success': False, 'error': str(e)}), 404
    except PermissionError as e:
        return jsonify({'success': False, 'error': str(e)}), 403
    except Exception as e:
        print(f"❌ Error in get_item_navigation: {e}")
        import traceback
        traceback.print_exc()
        return jsonify({'success': False, 'error': str(e)}), 500

@work_package_bp.route('/api/work-items/<item_id>/navigate', methods=['POST'])
@login_required
@role_required('cooperator')
def navigate_work_item(item_id):
    """CTV chuyển sang mảnh trước/tiếp theo (prev/next)"""
    try:
        user = get_current_user()
        data = request.get_json() or {}
        direction = data.get('direction', 'next')  # 'next' hoặc 'prev'
        
        wp_manager = get_wp_manager()
        
        # Lấy package_id của item hiện tại
        db_manager = current_app.config.get('db_manager')
        conn = db_manager.get_connection()
        cursor = conn.cursor()
        
        cursor.execute('SELECT package_id FROM work_items WHERE id = ?', (item_id,))
        row = cursor.fetchone()
        if not row:
            conn.close()
            return jsonify({'success': False, 'error': 'Không tìm thấy mảnh'}), 404
        
        package_id = row[0]
        
        # Unlock mảnh hiện tại
        cursor.execute('''
            UPDATE work_items SET locked_by = NULL, locked_at = NULL
            WHERE id = ? AND locked_by = ?
        ''', (item_id, user['id']))
        conn.commit()
        conn.close()
        
        # Lấy mảnh tiếp/trước
        result = wp_manager.get_next_work_item_for_cooperator(
            package_id, user['id'], current_item_id=item_id, direction=direction
        )
        
        if not result['items']:
            return jsonify({'success': False, 'error': 'Không có mảnh nào khác'})
        
        return jsonify({'success': True})
        
    except Exception as e:
        print(f"❌ Error navigating work item: {e}")
        return jsonify({'success': False, 'error': str(e)}), 500

@work_package_bp.route('/api/work-items/<item_id>/status', methods=['PUT'])
@login_required
def update_work_item_status(item_id):
    """Update work item status với external sync fix VÀ KIE DATA UPDATE + DOCUMENT METADATA"""
    try:
        data = request.get_json()
        new_status = data.get('status')
        review_comment = data.get('review_comment')
        kie_data = data.get('kie_data')  # ✅ LẤY KIE DATA
        document_metadata = data.get('document_metadata')  # ✅ THÊM DOCUMENT METADATA
        
        user = get_current_user()
        
        # Validate status
        valid_statuses = ['pending', 'completed', 'recompleted', 'rejected', 'approved']
        if new_status not in valid_statuses:
            return jsonify({'error': f'Invalid status: {new_status}'}), 400
        
        # ✅ GET DB_MANAGER
        db_manager = current_app.config.get('db_manager')
        if not db_manager:
            db_config = {
                'server': os.getenv('DB_SERVER', '127.0.0.1,1433'),
                'database': os.getenv('DB_DATABASE', 'kie_pla'),
                'username': os.getenv('DB_USERNAME', 'sa'),
                'password': os.getenv('DB_PASSWORD', ''),
                'driver': os.getenv('DB_DRIVER', '{ODBC Driver 17 for SQL Server}')
            }
            db_manager = DatabaseManager(db_config)
        
        # Get work item info
        conn = db_manager.get_connection()
        cursor = conn.cursor()
        
        cursor.execute('''
            SELECT wi.doc_id, wi.section_type, d.vanBanId, wi.status
            FROM work_items wi
            JOIN documents d ON wi.doc_id = d.id  
            WHERE wi.id = ?
        ''', (item_id,))
        
        work_item_info = cursor.fetchone()
        conn.close()
        
        if not work_item_info:
            return jsonify({'error': 'Work item not found'}), 404
        
        doc_id, section_type, van_ban_id, current_status = work_item_info
        
        # ✅ CHẶN XÁC NHẬN TRÙNG: CTV chỉ được confirm mảnh pending/rejected
        if user['role'] == 'cooperator' and new_status in ('completed', 'recompleted'):
            if current_status not in ('pending', 'rejected'):
                return jsonify({'error': f'Mảnh này đã được xử lý rồi (trạng thái: {current_status})'}), 409
        
        # ✅ CẬP NHẬT DOCUMENT METADATA NẾU CÓ (document_original_type, document_note)
        metadata_updated = False
        if document_metadata and new_status in ['completed', 'recompleted']:
            print(f"📝 Updating document metadata for doc {doc_id}")
            print(f"   Metadata: {document_metadata}")
            
            # Chỉ cập nhật các fields được phép
            update_data = {}
            if 'document_original_type' in document_metadata:
                update_data['document_original_type'] = document_metadata['document_original_type']
            if 'document_note' in document_metadata:
                update_data['document_note'] = document_metadata['document_note']
            
            if update_data:
                db_manager.update_document(doc_id, **update_data)
                metadata_updated = True
                print(f"   ✅ Updated document metadata: {list(update_data.keys())}")
        
        # ✅ CẬP NHẬT KIE DATA NẾU CÓ (KHI COOPERATOR LƯU)
        kie_updated = False
        if kie_data and new_status in ['completed', 'recompleted']:
            print(f"💾 Updating KIE data for item {item_id} (section: {section_type})")
            print(f"   New KIE fields: {list(kie_data.keys())}")
            
            # ✅ SMART MERGE: Lấy existing KIE data và merge
            existing_doc = db_manager.get_document(doc_id)
            
            if section_type == 'header':
                existing_kie = existing_doc.get('kie_header_data') or {}
                merged_kie = smart_merge_kie_data(existing_kie, kie_data)
                
                db_manager.update_document(
                    doc_id,
                    kie_header_data=merged_kie,
                    processed_at=datetime.now()
                )
                kie_updated = True
                print(f"   ✅ Updated header KIE: {len(merged_kie)} fields")
            else:  # content
                existing_kie = existing_doc.get('kie_content_data') or {}
                merged_kie = smart_merge_kie_data(existing_kie, kie_data)
                
                db_manager.update_document(
                    doc_id,
                    kie_content_data=merged_kie,
                    processed_at=datetime.now()
                )
                kie_updated = True
                print(f"   ✅ Updated content KIE: {len(merged_kie)} fields")
        
        # ✅ UPDATE WORK ITEM STATUS
        db_manager.update_work_item_status(
            item_id, 
            new_status, 
            reviewed_by=user['id'] if new_status in ['rejected', 'approved'] else None,
            review_comment=review_comment,
            completed_by=user['id'] if new_status in ['completed', 'recompleted'] else None
        )
        
        print(f"✅ Work item {item_id} status updated to {new_status}")
        
        external_sync_result = None
        
        # ✅ EXTERNAL SYNC KHI APPROVED
        if new_status == 'approved' and van_ban_id:
            try:
                from external_api import get_external_api
                
                # Get complete document data  
                document = db_manager.get_document(doc_id)
                
                if document:
                    # ✅ MERGE HEADER VÀ CONTENT DATA - ƯU TIÊN HEADER → CONTENT
                    header_data = document.get('kie_header_data', {})
                    content_data = document.get('kie_content_data', {})
                    
                    # Tạo combined data từ header
                    combined_data = {}
                    if isinstance(header_data, dict):
                        combined_data.update(header_data)
                    
                    # ✅ MERGE SUMMARY - ƯU TIÊN HEADER → CONTENT
                    header_summary = header_data.get('summary', '').strip() if isinstance(header_data, dict) else ''
                    content_summary = content_data.get('summary', '').strip() if isinstance(content_data, dict) else ''
                    
                    final_summary = ""
                    if header_summary and header_summary not in ['', 'N/A', 'Không có', 'null']:
                        final_summary = header_summary
                        print(f"🎯 Using HEADER summary for sync: {final_summary[:50]}...")
                    elif content_summary and content_summary not in ['', 'N/A', 'Không có', 'null']:
                        final_summary = content_summary
                        print(f"🎯 Using CONTENT summary for sync (fallback): {final_summary[:50]}...")
                    else:
                        print(f"⚠️ No summary found for external sync")
                    
                    # ✅ CẬP NHẬT SUMMARY TRONG COMBINED DATA
                    combined_data['summary'] = final_summary
                    
                    print(f"🔄 Syncing to external server:")
                    print(f"   vanBanId: {van_ban_id}")
                    print(f"   Summary length: {len(final_summary)}")
                    print(f"   Document type: {combined_data.get('document_type', 'N/A')}")
                    
                    # Send to external API
                    external_api = get_external_api()
                    sync_success = external_api.update_document(van_ban_id, combined_data)
                    
                    external_sync_result = {
                        'attempted': True,
                        'success': sync_success,
                        'van_ban_id': van_ban_id,
                        'summary_used': final_summary[:100] + '...' if len(final_summary) > 100 else final_summary
                    }
                    
                    if sync_success:
                        print(f"✅ External sync successful for vanBanId: {van_ban_id}")
                    else:
                        print(f"❌ External sync failed for vanBanId: {van_ban_id}")
                        
            except Exception as e:
                print(f"❌ External sync error: {e}")
                external_sync_result = {
                    'attempted': True,
                    'success': False,
                    'error': str(e),
                    'van_ban_id': van_ban_id
                }
        
        return jsonify({
            'success': True,
            'message': f'Work item status updated to {new_status}',
            'new_status': new_status,
            'kie_updated': kie_updated,  # ✅ THÔNG BÁO CÓ UPDATE KIE
            'metadata_updated': metadata_updated,  # ✅ THÔNG BÁO CÓ UPDATE METADATA
            'external_sync': external_sync_result
        })
        
    except Exception as e:
        print(f"❌ Error updating work item status: {e}")
        import traceback
        traceback.print_exc()
        return jsonify({
            'success': False, 
            'error': str(e)
        }), 500

@work_package_bp.route('/api/packages/<package_id>/bulk-review', methods=['POST'])
@login_required
@role_required('customer')
def bulk_review_items(package_id):
    """API review nhiều items cùng lúc"""
    try:
        user = get_current_user()
        data = request.get_json()
        
        if not data.get('item_ids') or not isinstance(data['item_ids'], list):
            return jsonify({'success': False, 'error': 'item_ids array is required'}), 400
            
        if not data.get('action') or data['action'] not in ['approve', 'reject']:
            return jsonify({'success': False, 'error': 'action must be approve or reject'}), 400
        
        wp_manager = get_wp_manager()  # ✅ SỬ DỤNG FUNCTION
        
        # Kiểm tra quyền truy cập package
        wp_manager.get_package_detail(package_id, user['id'], user['role'])
        
        result = wp_manager.bulk_review_items(
            item_ids=data['item_ids'],
            action=data['action'],
            comment=data.get('comment', ''),
            reviewed_by=user['id']
        )
        
        return jsonify(result)
        
    except ValueError as e:
        return jsonify({'success': False, 'error': str(e)}), 400
    except PermissionError as e:
        return jsonify({'success': False, 'error': str(e)}), 403
    except Exception as e:
        print(f"❌ Error in bulk_review_items: {e}")
        import traceback
        traceback.print_exc()
        return jsonify({'success': False, 'error': str(e)}), 500


@work_package_bp.route('/api/packages/<package_id>/approve-all', methods=['POST'])
@login_required
@role_required('customer')
def approve_all_completed_items(package_id):
    """
    API phê duyệt tất cả items có status completed/recompleted trong package
    - Batch external sync cho tất cả items được approve
    - Trả về số lượng approved và skipped
    """
    try:
        user = get_current_user()
        wp_manager = get_wp_manager()
        
        # ✅ KIỂM TRA QUYỀN TRUY CẬP PACKAGE
        package_detail = wp_manager.get_package_detail(package_id, user['id'], user['role'])
        
        print(f"🎯 Approve-all request for package {package_id} by user {user['username']}")
        
        # ✅ LẤY TẤT CẢ ITEMS CÓ STATUS COMPLETED/RECOMPLETED
        conn = wp_manager.db.get_connection()
        cursor = conn.cursor()
        
        cursor.execute('''
            SELECT id, status
            FROM work_items 
            WHERE package_id = ? 
            AND status IN ('completed', 'recompleted')
        ''', (package_id,))
        
        eligible_items = cursor.fetchall()
        conn.close()
        
        item_ids = [row[0] for row in eligible_items]
        
        print(f"   Found {len(item_ids)} eligible items (completed/recompleted)")
        
        # ✅ NẾU KHÔNG CÓ ITEMS NÀO
        if not item_ids:
            total_items = package_detail.get('total_items', 0)
            return jsonify({
                'success': True,
                'message': 'Không có mảnh nào cần phê duyệt',
                'approved_count': 0,
                'skipped_count': total_items
            })
        
        # ✅ SỬ DỤNG bulk_review_items ĐỂ APPROVE TẤT CẢ
        result = wp_manager.bulk_review_items(
            item_ids=item_ids,
            review_status='approved',
            review_comment='Phê duyệt tất cả mảnh đã hoàn thành',
            reviewed_by=user['id']
        )
        
        # ✅ TÍNH TOÁN KẾT QUẢ
        approved_count = result.get('updated_count', 0)
        total_items = package_detail.get('total_items', 0)
        skipped_count = total_items - approved_count
        
        print(f"   ✅ Approved: {approved_count}, Skipped: {skipped_count}")
        
        return jsonify({
            'success': True,
            'message': f'Đã phê duyệt {approved_count} mảnh',
            'approved_count': approved_count,
            'skipped_count': skipped_count,
            'external_sync': True  # Batch sync đã được thực hiện trong bulk_review_items
        })
        
    except ValueError as e:
        print(f"❌ ValueError in approve_all: {e}")
        return jsonify({'success': False, 'error': str(e)}), 400
    except PermissionError as e:
        print(f"❌ PermissionError in approve_all: {e}")
        return jsonify({'success': False, 'error': str(e)}), 403
    except Exception as e:
        print(f"❌ Error in approve_all_completed_items: {e}")
        import traceback
        traceback.print_exc()
        return jsonify({'success': False, 'error': str(e)}), 500
@work_package_bp.route('/api/packages/<package_id>/review-items')
@login_required
@role_required('customer')
def get_package_review_items(package_id):
    """API lấy danh sách items cần review trong package"""
    try:
        user = get_current_user()
        page = int(request.args.get('page', 1))
        search = request.args.get('search', '')
        status_filter = request.args.get('status_filter', '')
        section_filter = request.args.get('section_filter', '')
        
        wp_manager = get_wp_manager()  # ✅ SỬ DỤNG FUNCTION
        
        # Kiểm tra quyền truy cập package
        wp_manager.get_package_detail(package_id, user['id'], user['role'])
        
        result = wp_manager.get_package_review_items(
            package_id=package_id,
            page=page,
            search_term=search,
            status_filter=status_filter,
            section_filter=section_filter
        )
        
        return jsonify({
            'success': True,
            **result
        })
        
    except ValueError as e:
        return jsonify({'success': False, 'error': str(e)}), 404
    except PermissionError as e:
        return jsonify({'success': False, 'error': str(e)}), 403
    except Exception as e:
        print(f"❌ Error in get_package_review_items: {e}")
        import traceback
        traceback.print_exc()
        return jsonify({'success': False, 'error': str(e)}), 500

@work_package_bp.route('/api/work-items/<item_id>/review', methods=['POST'])
@login_required
@role_required('customer')
def review_work_item(item_id):
    """API review một work item"""
    try:
        user = get_current_user()
        data = request.get_json()
        
        if not data.get('action') or data['action'] not in ['approve', 'reject']:
            return jsonify({'success': False, 'error': 'action must be approve or reject'}), 400
        
        wp_manager = get_wp_manager()  # ✅ SỬ DỤNG FUNCTION
        
        result = wp_manager.review_work_item(
            item_id=item_id,
            action=data['action'],
            comment=data.get('comment', ''),
            reviewed_by=user['id']
        )
        
        return jsonify(result)
        
    except ValueError as e:
        return jsonify({'success': False, 'error': str(e)}), 400
    except PermissionError as e:
        return jsonify({'success': False, 'error': str(e)}), 403
    except Exception as e:
        print(f"❌ Error in review_work_item: {e}")
        import traceback
        traceback.print_exc()
        return jsonify({'success': False, 'error': str(e)}), 500

# ===== STATIC FILE SERVING =====

@work_package_bp.route('/static/images/<path:filename>')
def serve_work_item_image(filename):
    """Serve ảnh mảnh cho work items"""
    try:
        # Kiểm tra đăng nhập
        if 'user_id' not in session:
            return jsonify({'error': 'Authentication required'}), 401
        
        # Đường dẫn đến thư mục chứa ảnh
        image_dir = os.path.join(os.getcwd(), 'static', 'images')
        
        return send_from_directory(image_dir, filename)
        
    except Exception as e:
        print(f"❌ Error serving image {filename}: {e}")
        return jsonify({'error': str(e)}), 404

# ===== ERROR HANDLERS =====

@work_package_bp.errorhandler(404)
def handle_404(e):
    return jsonify({'success': False, 'error': 'Resource not found'}), 404

@work_package_bp.errorhandler(403)
def handle_403(e):
    return jsonify({'success': False, 'error': 'Access forbidden'}), 403

@work_package_bp.errorhandler(500)
def handle_500(e):
    return jsonify({'success': False, 'error': 'Internal server error'}), 500

# ===== HELPER ROUTES FOR DEBUGGING =====

@work_package_bp.route('/api/debug/user-info')
@login_required
def debug_user_info():
    """Debug endpoint để kiểm tra thông tin user"""
    user = get_current_user()
    return jsonify({
        'success': True,
        'user': user,
        'session': dict(session)
    })
@work_package_bp.route('/api/auth/logout', methods=['POST'])
def logout():
    """API đăng xuất"""
    try:
        session.clear()
        return jsonify({'success': True, 'message': 'Đăng xuất thành công'})
    except Exception as e:
        return jsonify({'success': False, 'error': str(e)}), 500

@work_package_bp.route('/api/auth/me')
def get_current_user_info():
    """API lấy thông tin user hiện tại"""
    try:
        user = get_current_user()
        return jsonify({
            'success': True,
            'id': user['id'],
            'username': user['username'], 
            'role': user['role'],
            'full_name': user.get('full_name', user['username'])
        })
    except Exception as e:
        return jsonify({'success': False, 'error': 'Not authenticated'}), 401
# Thêm vào cuối work_package_routes.py

@work_package_bp.route('/api/packages/<package_id>/bulk-action', methods=['POST'])
@login_required
@role_required('customer')
def bulk_action_items(package_id):
    """API bulk action cho items với batch external sync"""
    try:
        user = get_current_user()
        data = request.get_json()
        
        if not data.get('item_ids') or not isinstance(data['item_ids'], list):
            return jsonify({'success': False, 'error': 'item_ids array is required'}), 400
            
        if not data.get('action') or data['action'] not in ['approve', 'reject']:
            return jsonify({'success': False, 'error': 'action must be approve or reject'}), 400
        
        # Kiểm tra lý do từ chối
        if data['action'] == 'reject' and not data.get('review_comment'):
            return jsonify({'success': False, 'error': 'Lý do từ chối là bắt buộc'}), 400
        
        wp_manager = get_wp_manager()
        
        # Kiểm tra quyền truy cập package
        wp_manager.get_package_detail(package_id, user['id'], user['role'])
        
        # Chuyển đổi action thành status
        status = 'approved' if data['action'] == 'approve' else 'rejected'
        
        # ✅ BULK REVIEW VỚI EXTERNAL BATCH SYNC
        result = wp_manager.bulk_review_items(
            item_ids=data['item_ids'],
            review_status=status,
            review_comment=data.get('review_comment', ''),
            reviewed_by=user['id']
        )
        
        # ✅ THÊM THÔNG BÁO VỀ EXTERNAL SYNC
        if status == 'approved':
            result['external_sync'] = True
            result['message'] += ' (Đang đồng bộ với external server)'
        
        return jsonify(result)
        
    except ValueError as e:
        return jsonify({'success': False, 'error': str(e)}), 400
    except PermissionError as e:
        return jsonify({'success': False, 'error': str(e)}), 403
    except Exception as e:
        print(f"❌ Error in bulk_action_items: {e}")
        import traceback
        traceback.print_exc()
        return jsonify({'success': False, 'error': str(e)}), 500

# ✅ FIX work_package_routes.py - FUNCTION smart_merge_kie_data

def smart_merge_kie_data(existing_kie, new_fields):
    """Smart merge KIE data - preserve metadata, update only changed fields"""
    
    # ✅ START WITH EXISTING DATA (preserve everything)
    merged = existing_kie.copy() if existing_kie else {}
    
    # ✅ UPDATE ONLY CHANGED FIELDS
    for field_key, new_value in new_fields.items():
        if new_value is not None and str(new_value).strip() != "":
            # Update non-empty values
            merged[field_key] = str(new_value).strip()
        elif field_key in merged and (new_value == "" or new_value is None):
            # ✅ EXPLICIT EMPTY: User intentionally cleared the field
            merged[field_key] = ""
        # ✅ SKIP: Don't update if new_value is None and field doesn't exist
    
    # ❌ XÓA SECTION NÀY - KHÔNG CẦN AUTO-UPDATE
    # if 'summary' in new_fields:
    #     summary_value = str(new_fields['summary']).strip() if new_fields['summary'] else ""
    #     merged['has_summary'] = bool(summary_value)
    #     if summary_value:
    #         merged['last_summary_update'] = datetime.now().isoformat()
    
    # ✅ FORCE PRESERVE CRITICAL METADATA (never overwrite)
    metadata_fields = [
        'processing_method', 
        'content_length', 
        'extracted_at',
        'external_source', 
        'project_id', 
        'van_ban_id',
        'server_path',
        'auto_batch',
        'has_summary'  # ✅ THÊM has_summary VÀO PRESERVE LIST
    ]
    
    for field in metadata_fields:
        if field in existing_kie and field not in new_fields:
            merged[field] = existing_kie[field]  # ✅ Force preserve
    
    print(f"🔀 Smart merge summary:")
    print(f"   Existing fields: {len(existing_kie)}")
    print(f"   New fields: {len(new_fields)}")
    print(f"   Merged fields: {len(merged)}")
    print(f"   Preserved metadata: {[f for f in metadata_fields if f in merged]}")
    
    return merged
@work_package_bp.route('/api/work-items/<item_id>/update-kie', methods=['PUT'])
@login_required
def update_work_item_kie(item_id):
    """API cập nhật KIE data với smart merge ở backend"""
    try:
        user = get_current_user()
        
        if user['role'] not in ['customer', 'cooperator']:
            return jsonify({'success': False, 'error': 'Không có quyền cập nhật KIE'}), 403
        
        data = request.get_json()
        if not data:
            return jsonify({'success': False, 'error': 'No data provided'}), 400
        
        wp_manager = get_wp_manager()
        
        # Kiểm tra quyền truy cập work item
        try:
            item_detail = wp_manager.get_work_item_detail(item_id, user['id'], user['role'])
        except (ValueError, PermissionError) as e:
            return jsonify({'success': False, 'error': str(e)}), 403
        
        # ✅ SMART MERGE KIE DATA Ở BACKEND
        doc_id = item_detail['doc_id']
        section_type = item_detail['section_type']
        
        print(f"🔄 Smart merging KIE data for item {item_id} (section: {section_type})")
        print(f"   New fields from frontend: {data}")
        
        # ✅ LẤY EXISTING KIE DATA TỪ DATABASE
        existing_doc = wp_manager.db.get_document(doc_id)
        if not existing_doc:
            return jsonify({'success': False, 'error': 'Document not found'}), 404
        
        if section_type == 'header' or section_type == 'full':
            # Header hoặc Full document - dùng kie_header_data
            existing_kie = existing_doc.get('kie_header_data') or {}
        else:  # content
            existing_kie = existing_doc.get('kie_content_data') or {}
        
        print(f"   Existing KIE data: {existing_kie}")
        
        # ✅ SMART MERGE: Preserve metadata, update only changed fields
        merged_kie = smart_merge_kie_data(existing_kie, data)
        
        print(f"   Merged KIE data: {merged_kie}")
        
        # ✅ CẬP NHẬT VÀO DATABASE
        if section_type == 'header' or section_type == 'full':
            wp_manager.db.update_document(
                doc_id,
                kie_header_data=merged_kie,
                processed_at=datetime.now()
            )
        else:  # content
            wp_manager.db.update_document(
                doc_id,
                kie_content_data=merged_kie,
                processed_at=datetime.now()
            )
        
        print(f"✅ Smart merge completed for item {item_id}")
        
        return jsonify({
            'success': True,
            'message': 'KIE data updated successfully',
            'updated_fields': len(data),
            'preserved_fields': len(existing_kie) - len(data)
        })
        
    except Exception as e:
        print(f"❌ Error in smart KIE merge: {e}")
        import traceback
        traceback.print_exc()
        return jsonify({
            'success': False,
            'error': f'Lỗi cập nhật KIE: {str(e)}'
        }), 500
@work_package_bp.route('/api/customer-batches', methods=['GET'])
@login_required
@role_required('customer')
def get_customer_batches():
    """API lấy danh sách batch của customer với số mảnh khả dụng"""
    try:
        user = get_current_user()
        wp_manager = get_wp_manager()
        batches = wp_manager.get_customer_batches_with_available_count(user['id'])
        
        return jsonify({
            'success': True,
            'batches': batches
        })
        
    except Exception as e:
        print(f"❌ Error getting customer batches: {e}")
        import traceback
        traceback.print_exc()
        return jsonify({'success': False, 'error': str(e)}), 500

@work_package_bp.route('/api/available-pieces-count-by-batches', methods=['POST'])
@login_required
@role_required('customer')
def get_available_pieces_count_by_batches():
    """API đếm mảnh khả dụng theo batch selection"""
    try:
        user = get_current_user()
        data = request.get_json()
        
        if not data:
            return jsonify({'success': False, 'error': 'No data provided'}), 400
        
        batch_mode = data.get('batch_mode', 'all')  # 'all' hoặc 'specific'
        selected_batch_ids = data.get('selected_batch_ids', [])
        
        wp_manager = get_wp_manager()
        
        if batch_mode == 'all':
            # Đếm tất cả mảnh như cũ
            count = wp_manager.db.get_available_pieces_count(user['id'])
        else:
            # Đếm theo batch được chọn
            count = wp_manager.get_available_pieces_count_by_batches(
                user['id'], selected_batch_ids
            )
        
        return jsonify({
            'success': True,
            'count': count,
            'batch_mode': batch_mode,
            'selected_batches': len(selected_batch_ids) if batch_mode == 'specific' else 'all'
        })
        
    except Exception as e:
        print(f"❌ Error counting pieces by batches: {e}")
        import traceback
        traceback.print_exc()
        return jsonify({'success': False, 'error': str(e)}), 500

@work_package_bp.route('/api/batch-fragmentation-mode/<batch_id>', methods=['GET'])
@login_required
@role_required('customer')
def get_batch_fragmentation_mode(batch_id):
    """API để check xem batch đã có gói với mode nào chưa"""
    try:
        user = get_current_user()
        wp_manager = get_wp_manager()
        
        conn = wp_manager.db.get_connection()
        cursor = conn.cursor()
        
        # Kiểm tra batch có gói nào chưa
        cursor.execute('''
            SELECT TOP 1 wp.is_fragmented, wp.name
            FROM work_packages wp
            JOIN work_items wi ON wp.id = wi.package_id
            JOIN documents d ON wi.doc_id = d.id
            WHERE d.batch_id = ?
            AND wp.customer_id = ?
            ORDER BY wp.created_at DESC
        ''', (batch_id, user['id']))
        
        result = cursor.fetchone()
        conn.close()
        
        if result:
            is_fragmented = bool(result[0]) if result[0] is not None else None
            package_name = result[1]
            
            return jsonify({
                'success': True,
                'has_packages': True,
                'is_fragmented': is_fragmented,
                'mode': 'fragmented' if is_fragmented else 'non-fragmented',
                'package_name': package_name
            })
        else:
            return jsonify({
                'success': True,
                'has_packages': False,
                'is_fragmented': None,
                'mode': None
            })
        
    except Exception as e:
        print(f"❌ Error checking batch fragmentation mode: {e}")
        import traceback
        traceback.print_exc()
        return jsonify({'success': False, 'error': str(e)}), 500

@work_package_bp.route('/api/available-documents-count-by-batch', methods=['POST'])
@login_required
@role_required('customer')
def get_available_documents_count_by_batch():
    """API đếm số tài liệu chưa được gán vào gói nào (cho chế độ không phân mảnh)"""
    try:
        user = get_current_user()
        data = request.get_json()
        
        if not data or not data.get('batch_id'):
            return jsonify({'success': False, 'error': 'Batch ID is required'}), 400
        
        batch_id = data['batch_id']
        wp_manager = get_wp_manager()
        
        conn = wp_manager.db.get_connection()
        cursor = conn.cursor()
        
        # ✅ FIX: Đếm số documents trong batch chưa có work item nào (cả 'full', 'header', 'content')
        cursor.execute('''
            SELECT COUNT(DISTINCT d.id)
            FROM documents d
            JOIN batches b ON d.batch_id = b.id
            WHERE d.batch_id = ?
            AND b.created_by = ?
            AND d.id NOT IN (
                SELECT DISTINCT wi.doc_id
                FROM work_items wi
                JOIN work_packages wp ON wi.package_id = wp.id
                WHERE wp.customer_id = ?
            )
        ''', (batch_id, user['id'], user['id']))
        
        count = cursor.fetchone()[0]
        conn.close()
        
        return jsonify({
            'success': True,
            'count': count,
            'batch_id': batch_id
        })
        
    except Exception as e:
        print(f"❌ Error counting available documents: {e}")
        import traceback
        traceback.print_exc()
        return jsonify({'success': False, 'error': str(e)}), 500

# ===== COOPERATOR STATISTICS API =====
@work_package_bp.route('/api/cooperator-statistics')
@login_required
@role_required('cooperator')
def get_cooperator_statistics():
    """API thống kê sản lượng tiến độ cho cooperator"""
    try:
        user = get_current_user()
        cooperator_id = user['id']
        package_filter = request.args.get('package_id', '')
        date_from = request.args.get('date_from', '')
        date_to = request.args.get('date_to', '')
        
        wp_manager = get_wp_manager()
        conn = wp_manager.db.get_connection()
        cursor = conn.cursor()
        
        # 1. Tổng số mảnh CTV đã xử lý (completed_by = cooperator_id)
        pkg_filter_sql = ''
        params = [cooperator_id]
        if package_filter:
            pkg_filter_sql = ' AND wi.package_id = ?'
            params.append(package_filter)
        if date_from:
            pkg_filter_sql += ' AND wi.completed_at >= CAST(? AS DATE)'
            params.append(date_from)
        if date_to:
            pkg_filter_sql += ' AND wi.completed_at < DATEADD(day, 1, CAST(? AS DATE))'
            params.append(date_to)
        
        cursor.execute(f'''
            SELECT 
                COUNT(*) as total_items,
                COUNT(CASE WHEN wi.status = 'completed' THEN 1 END) as completed,
                COUNT(CASE WHEN wi.status = 'recompleted' THEN 1 END) as recompleted,
                COUNT(CASE WHEN wi.status = 'rejected' THEN 1 END) as rejected,
                COUNT(CASE WHEN wi.status = 'approved' THEN 1 END) as approved
            FROM work_items wi
            WHERE wi.completed_by = ?{pkg_filter_sql}
        ''', params)
        items_row = cursor.fetchone()
        
        # 2. Tổng số mảnh trong các gói được giao (để tính tỷ lệ)
        pkg_params = [cooperator_id]
        pkg_filter_sql2 = ''
        if package_filter:
            pkg_filter_sql2 = ' AND wp.id = ?'
            pkg_params.append(package_filter)
        
        cursor.execute(f'''
            SELECT 
                COUNT(DISTINCT wp.id) as total_packages,
                ISNULL(SUM(CASE WHEN wp.status = 'in_progress' THEN 1 ELSE 0 END), 0) as in_progress,
                ISNULL(SUM(CASE WHEN wp.status = 'has_rejected' THEN 1 ELSE 0 END), 0) as has_rejected,
                ISNULL(SUM(CASE WHEN wp.status = 'review_pending' THEN 1 ELSE 0 END), 0) as review_pending,
                ISNULL(SUM(CASE WHEN wp.status = 'approved' THEN 1 ELSE 0 END), 0) as pkg_approved
            FROM work_packages wp
            JOIN work_package_cooperators wpc ON wpc.package_id = wp.id
            WHERE wpc.cooperator_id = ?{pkg_filter_sql2}
        ''', pkg_params)
        pkg_row = cursor.fetchone()
        
        # 3. Thống kê theo từng gói
        pkg_detail_params = [cooperator_id]
        pkg_detail_date_sql = ''
        if date_from:
            pkg_detail_date_sql += ' AND wi.completed_at >= CAST(? AS DATE)'
            pkg_detail_params.append(date_from)
        if date_to:
            pkg_detail_date_sql += ' AND wi.completed_at < DATEADD(day, 1, CAST(? AS DATE))'
            pkg_detail_params.append(date_to)
        pkg_detail_params.append(cooperator_id)
        
        cursor.execute(f'''
            SELECT 
                wp.id, wp.name, wp.status,
                (SELECT COUNT(*) FROM work_items wi2 WHERE wi2.package_id = wp.id) as total_in_pkg,
                COUNT(wi.id) as my_items,
                COUNT(CASE WHEN wi.status = 'completed' THEN 1 END) as completed,
                COUNT(CASE WHEN wi.status = 'recompleted' THEN 1 END) as recompleted,
                COUNT(CASE WHEN wi.status = 'rejected' THEN 1 END) as rejected,
                COUNT(CASE WHEN wi.status = 'approved' THEN 1 END) as approved
            FROM work_packages wp
            JOIN work_package_cooperators wpc ON wpc.package_id = wp.id
            LEFT JOIN work_items wi ON wi.package_id = wp.id AND wi.completed_by = ?{pkg_detail_date_sql}
            WHERE wpc.cooperator_id = ?
            GROUP BY wp.id, wp.name, wp.status
            ORDER BY wp.name
        ''', pkg_detail_params)
        
        package_stats = []
        for row in cursor.fetchall():
            my_items = row[4] or 0
            done = (row[5] or 0) + (row[6] or 0) + (row[8] or 0)
            package_stats.append({
                'id': row[0],
                'name': row[1],
                'status': row[2],
                'total_in_package': row[3] or 0,
                'my_items': my_items,
                'completed': row[5] or 0,
                'recompleted': row[6] or 0,
                'rejected': row[7] or 0,
                'approved': row[8] or 0,
                'progress': round(done / my_items * 100, 1) if my_items > 0 else 0
            })
        
        conn.close()
        
        total_items = items_row[0] or 0
        done_items = (items_row[1] or 0) + (items_row[2] or 0) + (items_row[4] or 0)
        
        return jsonify({
            'success': True,
            'overview': {
                'total_packages': pkg_row[0] or 0,
                'packages_in_progress': pkg_row[1] or 0,
                'packages_has_rejected': pkg_row[2] or 0,
                'packages_review_pending': pkg_row[3] or 0,
                'packages_approved': pkg_row[4] or 0,
            },
            'work_items': {
                'total': total_items,
                'completed': items_row[1] or 0,
                'recompleted': items_row[2] or 0,
                'rejected': items_row[3] or 0,
                'approved': items_row[4] or 0,
                'progress': round(done_items / total_items * 100, 1) if total_items > 0 else 0
            },
            'package_stats': package_stats
        })
        
    except Exception as e:
        print(f"❌ Error in cooperator statistics: {e}")
        import traceback
        traceback.print_exc()
        return jsonify({'success': False, 'error': str(e)}), 500

# ===== CUSTOMER STATISTICS API =====
@work_package_bp.route('/api/customer-statistics')
@login_required
@role_required('customer')
def get_customer_statistics():
    """API thống kê sản lượng tiến độ cho customer"""
    try:
        user = get_current_user()
        customer_id = user['id']
        cooperator_filter = request.args.get('cooperator_id', '')
        date_from = request.args.get('date_from', '')
        date_to = request.args.get('date_to', '')
        
        wp_manager = get_wp_manager()
        conn = wp_manager.db.get_connection()
        cursor = conn.cursor()
        
        # 1. Tổng quan batch & documents
        cursor.execute('''
            SELECT 
                COUNT(DISTINCT b.id) as total_batches,
                ISNULL(SUM(b.total_files), 0) as total_files_uploaded,
                ISNULL(SUM(b.processed_files), 0) as total_files_processed
            FROM batches b
            WHERE b.created_by = ?
        ''', (customer_id,))
        batch_row = cursor.fetchone()
        
        # 2. Tổng quan work items
        wi_filter = ''
        wi_params = [customer_id]
        if cooperator_filter:
            wi_filter = ' AND wi.completed_by = ?'
            wi_params.append(cooperator_filter)
        if date_from:
            wi_filter += ' AND wi.completed_at >= CAST(? AS DATE)'
            wi_params.append(date_from)
        if date_to:
            wi_filter += ' AND wi.completed_at < DATEADD(day, 1, CAST(? AS DATE))'
            wi_params.append(date_to)
        
        cursor.execute(f'''
            SELECT 
                COUNT(*) as total_items,
                COUNT(CASE WHEN wi.status = 'pending' THEN 1 END) as pending,
                COUNT(CASE WHEN wi.status = 'completed' THEN 1 END) as completed,
                COUNT(CASE WHEN wi.status = 'recompleted' THEN 1 END) as recompleted,
                COUNT(CASE WHEN wi.status = 'rejected' THEN 1 END) as rejected,
                COUNT(CASE WHEN wi.status = 'approved' THEN 1 END) as approved
            FROM work_items wi
            JOIN work_packages wp ON wi.package_id = wp.id
            WHERE wp.customer_id = ?{wi_filter}
        ''', wi_params)
        items_row = cursor.fetchone()
        
        # 3. Tổng quan packages
        pkg_filter = ''
        pkg_params = [customer_id]
        if cooperator_filter:
            pkg_filter = ' AND wp.id IN (SELECT package_id FROM work_package_cooperators WHERE cooperator_id = ?)'
            pkg_params.append(cooperator_filter)
        
        cursor.execute(f'''
            SELECT 
                COUNT(*) as total_packages,
                COUNT(CASE WHEN wp.status = 'in_progress' THEN 1 END) as in_progress,
                COUNT(CASE WHEN wp.status = 'has_rejected' THEN 1 END) as has_rejected,
                COUNT(CASE WHEN wp.status = 'review_pending' THEN 1 END) as review_pending,
                COUNT(CASE WHEN wp.status = 'approved' THEN 1 END) as pkg_approved
            FROM work_packages wp
            WHERE wp.customer_id = ?{pkg_filter}
        ''', pkg_params)
        pkg_row = cursor.fetchone()
        
        # 4. Thống kê theo từng cooperator (chỉ đếm mảnh CTV thực sự xác nhận)
        coop_params = []
        coop_date_sql = ''
        if date_from:
            coop_date_sql += ' AND wi.completed_at >= CAST(? AS DATE)'
            coop_params.append(date_from)
        if date_to:
            coop_date_sql += ' AND wi.completed_at < DATEADD(day, 1, CAST(? AS DATE))'
            coop_params.append(date_to)
        coop_params.append(customer_id)
        
        cursor.execute(f'''
            SELECT 
                u.id, u.full_name, u.username,
                COUNT(wi.id) as total_items,
                COUNT(CASE WHEN wi.status = 'completed' THEN 1 END) as completed,
                COUNT(CASE WHEN wi.status = 'recompleted' THEN 1 END) as recompleted,
                COUNT(CASE WHEN wi.status = 'rejected' THEN 1 END) as rejected,
                COUNT(CASE WHEN wi.status = 'approved' THEN 1 END) as approved
            FROM work_package_cooperators wpc
            JOIN users u ON wpc.cooperator_id = u.id
            LEFT JOIN work_items wi ON wi.completed_by = u.id{coop_date_sql}
            WHERE wpc.package_id IN (
                SELECT id FROM work_packages WHERE customer_id = ?
            )
            GROUP BY u.id, u.full_name, u.username
            ORDER BY u.full_name
        ''', coop_params)
        
        cooperator_stats = []
        for row in cursor.fetchall():
            total = row[3] or 0
            done = (row[4] or 0) + (row[5] or 0) + (row[7] or 0)
            cooperator_stats.append({
                'id': row[0],
                'full_name': row[1],
                'username': row[2],
                'total_items': total,
                'completed': row[4] or 0,
                'recompleted': row[5] or 0,
                'rejected': row[6] or 0,
                'approved': row[7] or 0,
                'progress': round(done / total * 100, 1) if total > 0 else 0
            })
        
        conn.close()
        
        total_items = items_row[0] or 0
        done_items = (items_row[2] or 0) + (items_row[3] or 0) + (items_row[5] or 0)
        
        return jsonify({
            'success': True,
            'overview': {
                'total_batches': batch_row[0] or 0,
                'total_files_uploaded': batch_row[1] or 0,
                'total_files_processed': batch_row[2] or 0,
                'total_packages': pkg_row[0] or 0,
                'packages_in_progress': pkg_row[1] or 0,
                'packages_has_rejected': pkg_row[2] or 0,
                'packages_review_pending': pkg_row[3] or 0,
                'packages_approved': pkg_row[4] or 0,
            },
            'work_items': {
                'total': total_items,
                'pending': items_row[1] or 0,
                'completed': items_row[2] or 0,
                'recompleted': items_row[3] or 0,
                'rejected': items_row[4] or 0,
                'approved': items_row[5] or 0,
                'progress': round(done_items / total_items * 100, 1) if total_items > 0 else 0
            },
            'cooperator_stats': cooperator_stats
        })
        
    except Exception as e:
        print(f"❌ Error in customer statistics: {e}")
        import traceback
        traceback.print_exc()
        return jsonify({'success': False, 'error': str(e)}), 500


# ===== ADMIN STATISTICS API =====
@work_package_bp.route('/api/admin-statistics')
@login_required
@role_required('admin')
def get_admin_statistics():
    """API thống kê sản lượng tiến độ cho admin - tổng hợp toàn hệ thống"""
    try:
        customer_filter = request.args.get('customer_id', '')
        cooperator_filter = request.args.get('cooperator_id', '')
        date_from = request.args.get('date_from', '')
        date_to = request.args.get('date_to', '')
        
        wp_manager = get_wp_manager()
        conn = wp_manager.db.get_connection()
        cursor = conn.cursor()
        
        # 1. Tổng quan batch & documents
        batch_filter_sql = ''
        batch_params = []
        if customer_filter:
            batch_filter_sql = ' WHERE b.created_by = ?'
            batch_params.append(customer_filter)
        
        cursor.execute(f'''
            SELECT 
                COUNT(DISTINCT b.id) as total_batches,
                ISNULL(SUM(b.total_files), 0) as total_files_uploaded,
                ISNULL(SUM(b.processed_files), 0) as total_files_processed
            FROM batches b{batch_filter_sql}
        ''', batch_params)
        batch_row = cursor.fetchone()
        
        # 2. Tổng quan work items
        wi_conditions = []
        wi_params = []
        if customer_filter:
            wi_conditions.append('wp.customer_id = ?')
            wi_params.append(customer_filter)
        if cooperator_filter:
            wi_conditions.append('wi.completed_by = ?')
            wi_params.append(cooperator_filter)
        if date_from:
            wi_conditions.append('wi.completed_at >= CAST(? AS DATE)')
            wi_params.append(date_from)
        if date_to:
            wi_conditions.append('wi.completed_at < DATEADD(day, 1, CAST(? AS DATE))')
            wi_params.append(date_to)
        wi_where = (' AND ' + ' AND '.join(wi_conditions)) if wi_conditions else ''
        
        cursor.execute(f'''
            SELECT 
                COUNT(*) as total_items,
                COUNT(CASE WHEN wi.status = 'pending' THEN 1 END) as pending,
                COUNT(CASE WHEN wi.status = 'completed' THEN 1 END) as completed,
                COUNT(CASE WHEN wi.status = 'recompleted' THEN 1 END) as recompleted,
                COUNT(CASE WHEN wi.status = 'rejected' THEN 1 END) as rejected,
                COUNT(CASE WHEN wi.status = 'approved' THEN 1 END) as approved
            FROM work_items wi
            JOIN work_packages wp ON wi.package_id = wp.id
            WHERE 1=1{wi_where}
        ''', wi_params)
        items_row = cursor.fetchone()
        
        # 3. Tổng quan packages
        pkg_conditions = []
        pkg_params = []
        if customer_filter:
            pkg_conditions.append('wp.customer_id = ?')
            pkg_params.append(customer_filter)
        if cooperator_filter:
            pkg_conditions.append('wp.id IN (SELECT package_id FROM work_package_cooperators WHERE cooperator_id = ?)')
            pkg_params.append(cooperator_filter)
        pkg_where = (' AND ' + ' AND '.join(pkg_conditions)) if pkg_conditions else ''
        
        cursor.execute(f'''
            SELECT 
                COUNT(*) as total_packages,
                COUNT(CASE WHEN wp.status = 'in_progress' THEN 1 END) as in_progress,
                COUNT(CASE WHEN wp.status = 'has_rejected' THEN 1 END) as has_rejected,
                COUNT(CASE WHEN wp.status = 'review_pending' THEN 1 END) as review_pending,
                COUNT(CASE WHEN wp.status = 'approved' THEN 1 END) as pkg_approved
            FROM work_packages wp
            WHERE 1=1{pkg_where}
        ''', pkg_params)
        pkg_row = cursor.fetchone()
        
        # 4. Thống kê theo từng customer
        admin_cust_params = []
        admin_cust_date_sql = ''
        if date_from:
            admin_cust_date_sql += ' AND wi.completed_at >= CAST(? AS DATE)'
            admin_cust_params.append(date_from)
        if date_to:
            admin_cust_date_sql += ' AND wi.completed_at < DATEADD(day, 1, CAST(? AS DATE))'
            admin_cust_params.append(date_to)
        
        cursor.execute(f'''
            SELECT 
                u.id, u.full_name, u.username,
                COUNT(DISTINCT wp.id) as total_packages,
                COUNT(wi.id) as total_items,
                COUNT(CASE WHEN wi.status = 'completed' THEN 1 END) as completed,
                COUNT(CASE WHEN wi.status = 'recompleted' THEN 1 END) as recompleted,
                COUNT(CASE WHEN wi.status = 'rejected' THEN 1 END) as rejected,
                COUNT(CASE WHEN wi.status = 'approved' THEN 1 END) as approved
            FROM users u
            JOIN work_packages wp ON wp.customer_id = u.id
            LEFT JOIN work_items wi ON wi.package_id = wp.id{admin_cust_date_sql}
            WHERE u.role = 'customer'
            GROUP BY u.id, u.full_name, u.username
            ORDER BY u.full_name
        ''', admin_cust_params)
        
        customer_stats = []
        for row in cursor.fetchall():
            total = row[4] or 0
            done = (row[5] or 0) + (row[6] or 0) + (row[8] or 0)
            customer_stats.append({
                'id': row[0],
                'full_name': row[1],
                'username': row[2],
                'total_packages': row[3] or 0,
                'total_items': total,
                'completed': row[5] or 0,
                'recompleted': row[6] or 0,
                'rejected': row[7] or 0,
                'approved': row[8] or 0,
                'progress': round(done / total * 100, 1) if total > 0 else 0
            })
        
        # 5. Thống kê theo từng cooperator
        admin_coop_params = []
        admin_coop_date_sql = ''
        if date_from:
            admin_coop_date_sql += ' AND wi.completed_at >= CAST(? AS DATE)'
            admin_coop_params.append(date_from)
        if date_to:
            admin_coop_date_sql += ' AND wi.completed_at < DATEADD(day, 1, CAST(? AS DATE))'
            admin_coop_params.append(date_to)
        
        cursor.execute(f'''
            SELECT 
                u.id, u.full_name, u.username,
                COUNT(wi.id) as total_items,
                COUNT(CASE WHEN wi.status = 'completed' THEN 1 END) as completed,
                COUNT(CASE WHEN wi.status = 'recompleted' THEN 1 END) as recompleted,
                COUNT(CASE WHEN wi.status = 'rejected' THEN 1 END) as rejected,
                COUNT(CASE WHEN wi.status = 'approved' THEN 1 END) as approved
            FROM users u
            LEFT JOIN work_items wi ON wi.completed_by = u.id{admin_coop_date_sql}
            WHERE u.role = 'cooperator'
            GROUP BY u.id, u.full_name, u.username
            ORDER BY u.full_name
        ''', admin_coop_params)
        
        cooperator_stats = []
        for row in cursor.fetchall():
            total = row[3] or 0
            done = (row[4] or 0) + (row[5] or 0) + (row[7] or 0)
            cooperator_stats.append({
                'id': row[0],
                'full_name': row[1],
                'username': row[2],
                'total_items': total,
                'completed': row[4] or 0,
                'recompleted': row[5] or 0,
                'rejected': row[6] or 0,
                'approved': row[7] or 0,
                'progress': round(done / total * 100, 1) if total > 0 else 0
            })
        
        conn.close()
        
        total_items = items_row[0] or 0
        done_items = (items_row[2] or 0) + (items_row[3] or 0) + (items_row[5] or 0)
        
        return jsonify({
            'success': True,
            'overview': {
                'total_batches': batch_row[0] or 0,
                'total_files_uploaded': batch_row[1] or 0,
                'total_files_processed': batch_row[2] or 0,
                'total_packages': pkg_row[0] or 0,
                'packages_in_progress': pkg_row[1] or 0,
                'packages_has_rejected': pkg_row[2] or 0,
                'packages_review_pending': pkg_row[3] or 0,
                'packages_approved': pkg_row[4] or 0,
            },
            'work_items': {
                'total': total_items,
                'pending': items_row[1] or 0,
                'completed': items_row[2] or 0,
                'recompleted': items_row[3] or 0,
                'rejected': items_row[4] or 0,
                'approved': items_row[5] or 0,
                'progress': round(done_items / total_items * 100, 1) if total_items > 0 else 0
            },
            'customer_stats': customer_stats,
            'cooperator_stats': cooperator_stats
        })
        
    except Exception as e:
        print(f"❌ Error in admin statistics: {e}")
        import traceback
        traceback.print_exc()
        return jsonify({'success': False, 'error': str(e)}), 500