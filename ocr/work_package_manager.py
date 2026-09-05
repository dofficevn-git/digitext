import uuid
import random
from datetime import datetime
from database import DatabaseManager
import json
from external_api import get_external_api

class WorkPackageManager:
    def __init__(self, db_manager):
        self.db = db_manager

    def get_available_pieces(self, customer_id, limit=None):
        """Lấy các mảnh khả dụng (có KIE data hợp lệ) cho customer"""
        conn = self.db.get_connection()
        cursor = conn.cursor()
        
        try:
            # Lấy documents của customer (theo created_by của batch)
            cursor.execute('''
                SELECT 
                    d.id, d.filename, d.image_header_path, d.image_content_path,
                    d.kie_header_data, d.kie_content_data, d.document_type,
                    b.name as batch_name
                FROM documents d
                JOIN batches b ON d.batch_id = b.id
                WHERE b.created_by = ?
            ''', (customer_id,))
            
            documents = cursor.fetchall()
            
            # Lấy danh sách các mảnh đã được gán
            cursor.execute('''
                SELECT doc_id, section_type 
                FROM work_items 
                WHERE package_id IN (
                    SELECT id FROM work_packages WHERE customer_id = ?
                )
            ''', (customer_id,))
            
            # ✅ FIX: Build assigned map with section types
            assigned_pieces = set()
            assigned_full_docs = set()  # Documents với section_type='full'
            
            for row in cursor.fetchall():
                doc_id, section_type = row[0], row[1]
                assigned_pieces.add((doc_id, section_type))
                
                # ✅ Nếu document đã được gán 'full', đánh dấu
                if section_type == 'full':
                    assigned_full_docs.add(doc_id)
            
            # Lọc mảnh khả dụng
            available_pieces = []
            
            for doc in documents:
                doc_id, filename, header_path, content_path, kie_header_data, kie_content_data, doc_type, batch_name = doc
                
                # ✅ FIX: Bỏ qua document đã được gán 'full'
                if doc_id in assigned_full_docs:
                    continue
                
                # Kiểm tra header piece
                if header_path and kie_header_data:
                    piece_key = (doc_id, 'header')
                    if (piece_key not in assigned_pieces and 
                        self.db._has_valid_kie_data(kie_header_data, 'header')):
                        available_pieces.append({
                            'doc_id': doc_id,
                            'filename': filename,
                            'section_type': 'header',
                            'image_path': header_path,
                            'kie_data': kie_header_data,
                            'document_type': doc_type,
                            'batch_name': batch_name
                        })
                
                # Kiểm tra content piece
                if content_path and kie_content_data:
                    piece_key = (doc_id, 'content')
                    if (piece_key not in assigned_pieces and 
                        self.db._has_valid_kie_data(kie_content_data, 'content')):
                        available_pieces.append({
                            'doc_id': doc_id,
                            'filename': filename,
                            'section_type': 'content',
                            'image_path': content_path,
                            'kie_data': kie_content_data,
                            'document_type': doc_type,
                            'batch_name': batch_name
                        })
            
            # Apply limit nếu có
            if limit and len(available_pieces) > limit:
                import random
                available_pieces = random.sample(available_pieces, limit)
            
            print(f"📊 Found {len(available_pieces)} available pieces for customer {customer_id}")
            return available_pieces
            
        except Exception as e:
            print(f"❌ Lỗi get_available_pieces: {e}")
            return []
        finally:
            conn.close()

    def auto_pick_pieces(self, customer_id, requested_count):
        """Tự động chọn mảnh theo số lượng yêu cầu"""
        available_count = self.db.get_available_pieces_count(customer_id)
        
        if requested_count > available_count:
            raise ValueError(f'Chỉ có {available_count} mảnh khả dụng, không thể tạo gói {requested_count} mảnh')
        
        # Lấy random pieces
        pieces = self.get_available_pieces(customer_id, limit=requested_count)
        
        if len(pieces) < requested_count:
            raise ValueError(f'Không đủ mảnh khả dụng. Yêu cầu: {requested_count}, Có: {len(pieces)}')
            
        return pieces

    def create_package(self, name, customer_id, cooperator_ids, item_count, is_fragmented=True):
        """Tạo gói công việc mới với tùy chọn phân mảnh"""
        
        # Validate inputs
        if not name or not name.strip():
            raise ValueError('Tên gói không được để trống')
        
        # Hỗ trợ cả string và list
        if isinstance(cooperator_ids, str):
            cooperator_ids = [cooperator_ids]
        
        if not cooperator_ids:
            raise ValueError('Phải chọn ít nhất 1 cooperator')
        
        # Kiểm tra relationship cho tất cả cooperators
        for coop_id in cooperator_ids:
            if not self._check_customer_cooperator_relationship(customer_id, coop_id):
                raise PermissionError('Bạn không có quyền gán công việc cho cooperator này')
        
        # Auto pick pieces
        pieces = self.auto_pick_pieces(customer_id, item_count)
        
        # Tạo package với chế độ phân mảnh
        package_id = str(uuid.uuid4())
        self.db.create_work_package(package_id, name.strip(), customer_id, cooperator_ids, item_count, is_fragmented)
        
        # ✅ TẠO WORK ITEMS TÙY THEO CHẾ ĐỘ PHÂN MẢNH
        items_data = []
        
        if is_fragmented:
            # Chế độ phân mảnh: Tạo 2 items cho mỗi piece (header + content)
            for piece in pieces:
                item_id = str(uuid.uuid4())
                items_data.append({
                    'id': item_id,
                    'package_id': package_id,
                    'doc_id': piece['doc_id'],
                    'section_type': piece['section_type']
                })
        else:
            # ✅ Chế độ không phân mảnh: Tạo 1 item cho mỗi document (full)
            # Nhóm pieces theo doc_id và chỉ lấy 1 item per document
            seen_docs = set()
            for piece in pieces:
                doc_id = piece['doc_id']
                if doc_id not in seen_docs:
                    item_id = str(uuid.uuid4())
                    items_data.append({
                        'id': item_id,
                        'package_id': package_id,
                        'doc_id': doc_id,
                        'section_type': 'full'  # ✅ SỬ DỤNG 'full' CHO KHÔNG PHÂN MẢNH
                    })
                    seen_docs.add(doc_id)
        
        self.db.create_work_items(items_data)
        
        mode_text = "phân mảnh (Header + Content)" if is_fragmented else "không phân mảnh (Full PDF)"
        return {
            'package_id': package_id,
            'name': name.strip(),
            'total_items': len(items_data),
            'is_fragmented': is_fragmented,
            'message': f'Đã tạo gói "{name}" với {len(items_data)} mảnh - Chế độ: {mode_text}'
        }

    def get_customer_packages(self, customer_id, search_term='', page=1, page_size=10, status_filter=''):
        """Lấy danh sách gói của customer với status filter ĐÚNG LOGIC"""
        
        # ✅ SỬA LOGIC: has_rejected THUỘC VỀ CẢ in_progress VÀ has_rejected
        if status_filter == 'in_progress':
            # "Đang xử lý" bao gồm: in_progress + has_rejected
            actual_filter = 'in_progress,has_rejected'
        else:
            # Các filter khác giữ nguyên
            actual_filter = status_filter
        
        return self.db.get_packages_paginated(
            customer_id=customer_id, 
            search_term=search_term, 
            page=page, 
            page_size=page_size,
            status_filter=actual_filter  # ✅ SỬ DỤNG ACTUAL_FILTER
        )

    # ✅ SỬA FUNCTION get_cooperator_packages - THÊM STATUS_FILTER  
    def get_cooperator_packages(self, cooperator_id, search_term='', page=1, page_size=10, status_filter=''):
        """Lấy danh sách gói được gán cho cooperator với status filter ĐÚNG LOGIC"""
        
        # ✅ SỬA LOGIC TƯƠNG TỰ
        if status_filter == 'in_progress':
            actual_filter = 'in_progress,has_rejected'
        else:
            actual_filter = status_filter
        
        result = self.db.get_packages_paginated(
            cooperator_id=cooperator_id, 
            search_term=search_term, 
            page=page, 
            page_size=page_size,
            status_filter=actual_filter
        )
        
        # Thêm thông tin gói bị rejected (virtual package)
        rejected_items = self.get_rejected_items(cooperator_id)
        if rejected_items['items']:
            result['has_rejected_items'] = True
            result['rejected_count'] = rejected_items['total_count']
        else:
            result['has_rejected_items'] = False
            result['rejected_count'] = 0
            
        return result

    def get_package_items(self, package_id, search_term='', page=1, page_size=5, status_filter=''):
        """Lấy danh sách work items trong package với KIE data và status filter"""
        conn = self.db.get_connection()
        cursor = conn.cursor()
        
        offset = (page - 1) * page_size
        
        base_query = '''
            FROM work_items wi
            JOIN documents d ON wi.doc_id = d.id
            WHERE wi.package_id = ?
        '''
        
        params = [package_id]
        
        if search_term:
            base_query += ' AND d.filename LIKE ?'
            params.append(f'%{search_term}%')
        
        # ✅ THÊM STATUS FILTER CONDITION - MISSING CODE!
        if status_filter:
            if ',' in status_filter:
                # Multiple statuses: "completed,recompleted"
                status_list = status_filter.split(',')
                placeholders = ','.join(['?' for _ in status_list])
                base_query += f' AND wi.status IN ({placeholders})'
                params.extend(status_list)
            else:
                # Single status
                base_query += ' AND wi.status = ?'
                params.append(status_filter)
        
        # Count total với filter
        count_query = f'SELECT COUNT(*) {base_query}'
        cursor.execute(count_query, params)
        total_count = cursor.fetchone()[0]
        
        # ✅ GET ITEMS VỚI FILTER
        data_query = f'''
            SELECT wi.id, wi.doc_id, wi.section_type, wi.status, wi.review_comment,
                wi.reviewed_by, wi.reviewed_at, wi.completed_at,
                d.filename, 
                CASE WHEN wi.section_type = 'header' THEN d.image_header_path 
                     WHEN wi.section_type = 'content' THEN d.image_content_path
                     ELSE NULL END as image_path,
                d.kie_header_data,
                d.kie_content_data,
                d.document_original_type,
                d.document_note,
                d.file_path
            {base_query}
            ORDER BY d.filename, wi.section_type
            OFFSET ? ROWS FETCH NEXT ? ROWS ONLY
        '''
        
        params.extend([offset, page_size])
        cursor.execute(data_query, params)
        
        items = []
        for row in cursor.fetchall():
            # ✅ XỬ LÝ KIE DATA DỰA TRÊN SECTION TYPE
            kie_data = {}
            section_type = row[2]
            
            print(f"🔍 Processing item {row[0]} - section: {section_type}, filename: {row[8]}")
            
            if section_type == 'header':
                # Lấy header KIE data
                kie_header_raw = row[10]  # kie_header_data column
                if kie_header_raw:
                    try:
                        if isinstance(kie_header_raw, str):
                            kie_data = json.loads(kie_header_raw)
                        elif isinstance(kie_header_raw, dict):
                            kie_data = kie_header_raw
                        else:
                            print(f"⚠️ Unexpected header KIE data type: {type(kie_header_raw)}")
                            kie_data = {}
                        
                        print(f"✅ Header KIE data parsed: {len(kie_data)} fields")
                        
                    except json.JSONDecodeError as e:
                        print(f"❌ Error parsing header KIE JSON: {e}")
                        kie_data = {}
                    except Exception as e:
                        print(f"❌ Error processing header KIE: {e}")
                        kie_data = {}
                else:
                    print(f"⚠️ No header KIE data for item {row[0]}")
            
            elif section_type == 'content':
                # Lấy content KIE data
                kie_content_raw = row[11]  # kie_content_data column
                if kie_content_raw:
                    try:
                        if isinstance(kie_content_raw, str):
                            kie_data = json.loads(kie_content_raw)
                        elif isinstance(kie_content_raw, dict):
                            kie_data = kie_content_raw
                        else:
                            print(f"⚠️ Unexpected content KIE data type: {type(kie_content_raw)}")
                            kie_data = {}
                        
                        print(f"✅ Content KIE data parsed: {len(kie_data)} fields")
                        
                    except json.JSONDecodeError as e:
                        print(f"❌ Error parsing content KIE JSON: {e}")
                        kie_data = {}
                    except Exception as e:
                        print(f"❌ Error processing content KIE: {e}")
                        kie_data = {}
                else:
                    print(f"⚠️ No content KIE data for item {row[0]}")
            
            elif section_type == 'full':
                # ✅ XỬ LÝ CHẾ ĐỘ KHÔNG PHÂN MẢNH - LẤY HEADER KIE
                kie_header_raw = row[10]  # kie_header_data column
                if kie_header_raw:
                    try:
                        if isinstance(kie_header_raw, str):
                            kie_data = json.loads(kie_header_raw)
                        elif isinstance(kie_header_raw, dict):
                            kie_data = kie_header_raw
                        else:
                            kie_data = {}
                        print(f"✅ Full document KIE data parsed: {len(kie_data)} fields")
                    except Exception as e:
                        print(f"❌ Error processing full document KIE: {e}")
                        kie_data = {}
                else:
                    print(f"⚠️ No KIE data for full document item {row[0]}")
                
                # ✅ CẢNH BÁO NẾU KHÔNG CÓ PDF PATH
                pdf_path = row[14]  # file_path column
                if not pdf_path:
                    print(f"⚠️ WARNING: No PDF file_path found for full document item {row[0]}")
                    print(f"   Expected PDF path in uploads/ directory")
                else:
                    print(f"✅ PDF path found: {pdf_path}")
            
            else:
                print(f"⚠️ Unknown section type: {section_type}")
            
            # ✅ BUILD ITEM OBJECT
            item = {
                'id': row[0],
                'doc_id': row[1], 
                'section_type': section_type,
                'status': row[3],
                'review_comment': row[4],
                'reviewed_by': row[5],
                'reviewed_at': row[6].isoformat() if row[6] else None,
                'completed_at': row[7].isoformat() if row[7] else None,
                'filename': row[8],
                'image_path': row[9],
                'kie_data': kie_data,
                'document_original_type': row[12] or 'Bản chính',  # ✅ MỚI
                'document_note': row[13] or '',  # ✅ MỚI
                'file_path': row[14] if len(row) > 14 else None  # ✅ PDF PATH
            }
            
            # ✅ THÊM DEBUG INFO
            print(f"📦 Item created: ID={item['id']}, status={item['status']}, kie_fields={len(kie_data)}")
            
            items.append(item)
        
        conn.close()
        
        # ✅ CALCULATE PAGINATION
        total_pages = (total_count + page_size - 1) // page_size
        
        print(f"📊 Package {package_id} items loaded:")
        print(f"   Total: {total_count}, Page: {page}/{total_pages}")
        print(f"   Items returned: {len(items)}")
        print(f"   Search: '{search_term}', Status filter: '{status_filter}'")
        
        return {
            'items': items,
            'total_count': total_count,
            'total_pages': total_pages,
            'current_page': page,
            'page_size': page_size,
            'has_next': page < total_pages,
            'has_prev': page > 1
        }

    def get_next_work_item_for_cooperator(self, package_id, cooperator_id, current_item_id=None, direction='next'):
        """Lấy 1 mảnh pending/rejected cho CTV, lock lại để tránh trùng.
        Hỗ trợ prev/next navigation."""
        conn = self.db.get_connection()
        cursor = conn.cursor()
        
        LOCK_TIMEOUT_MINUTES = 10
        
        locked_item_id = None
        
        if current_item_id and direction:
            # NAVIGATE MODE: CTV đang ở mảnh current_item_id, muốn chuyển prev/next
            # Lấy created_at của mảnh hiện tại để so sánh
            cursor.execute('SELECT created_at FROM work_items WHERE id = ?', (current_item_id,))
            cur_row = cursor.fetchone()
            
            if cur_row:
                cur_created_at = cur_row[0]
                
                if direction == 'next':
                    cursor.execute('''
                        SELECT TOP 1 wi.id 
                        FROM work_items wi
                        WHERE wi.package_id = ? 
                          AND wi.status IN ('pending', 'rejected')
                          AND wi.id != ?
                          AND (wi.locked_by IS NULL OR wi.locked_by = ?
                               OR wi.locked_at < DATEADD(MINUTE, -?, GETDATE()))
                          AND wi.created_at >= ?
                        ORDER BY wi.created_at ASC, wi.id ASC
                    ''', (package_id, current_item_id, cooperator_id, LOCK_TIMEOUT_MINUTES, cur_created_at))
                else:  # prev
                    cursor.execute('''
                        SELECT TOP 1 wi.id 
                        FROM work_items wi
                        WHERE wi.package_id = ? 
                          AND wi.status IN ('pending', 'rejected')
                          AND wi.id != ?
                          AND (wi.locked_by IS NULL OR wi.locked_by = ?
                               OR wi.locked_at < DATEADD(MINUTE, -?, GETDATE()))
                          AND wi.created_at <= ?
                        ORDER BY wi.created_at DESC, wi.id DESC
                    ''', (package_id, current_item_id, cooperator_id, LOCK_TIMEOUT_MINUTES, cur_created_at))
                
                nav_item = cursor.fetchone()
                if nav_item:
                    locked_item_id = nav_item[0]
        
        if not locked_item_id:
            # FIRST LOAD hoặc navigate không tìm thấy → check existing lock hoặc lấy mảnh đầu
            cursor.execute('''
                SELECT wi.id FROM work_items wi
                WHERE wi.package_id = ? 
                  AND wi.locked_by = ? 
                  AND wi.status IN ('pending', 'rejected')
                  AND wi.locked_at > DATEADD(MINUTE, -?, GETDATE())
            ''', (package_id, cooperator_id, LOCK_TIMEOUT_MINUTES))
            
            existing_lock = cursor.fetchone()
            
            if existing_lock:
                locked_item_id = existing_lock[0]
            else:
                # Tìm mảnh pending/rejected chưa ai lock
                cursor.execute('''
                    SELECT TOP 1 wi.id 
                    FROM work_items wi
                    WHERE wi.package_id = ? 
                      AND wi.status IN ('pending', 'rejected')
                      AND (wi.locked_by IS NULL 
                           OR wi.locked_at < DATEADD(MINUTE, -?, GETDATE()))
                    ORDER BY wi.created_at ASC
                ''', (package_id, LOCK_TIMEOUT_MINUTES))
                
                next_item = cursor.fetchone()
                if next_item:
                    locked_item_id = next_item[0]
        
        if not locked_item_id:
            conn.close()
            return {
                'items': [],
                'total_count': 0,
                'total_pages': 0,
                'current_page': 1,
                'page_size': 1,
                'has_next': False,
                'has_prev': False,
                'remaining_count': 0
            }
        
        # Lock mảnh cho CTV này
        cursor.execute('''
            UPDATE work_items 
            SET locked_by = ?, locked_at = GETDATE()
            WHERE id = ?
        ''', (cooperator_id, locked_item_id))
        conn.commit()
        
        # 4. Đếm số mảnh còn lại (pending/rejected, chưa lock hoặc lock hết hạn)
        cursor.execute('''
            SELECT COUNT(*) FROM work_items wi
            WHERE wi.package_id = ? 
              AND wi.status IN ('pending', 'rejected')
        ''', (package_id,))
        remaining_count = cursor.fetchone()[0]
        
        # 5. Lấy full data mảnh đang lock
        cursor.execute('''
            SELECT wi.id, wi.doc_id, wi.section_type, wi.status, wi.review_comment,
                wi.reviewed_by, wi.reviewed_at, wi.completed_at,
                d.filename, 
                CASE WHEN wi.section_type = 'header' THEN d.image_header_path 
                     WHEN wi.section_type = 'content' THEN d.image_content_path
                     ELSE NULL END as image_path,
                d.kie_header_data,
                d.kie_content_data,
                d.document_original_type,
                d.document_note,
                d.file_path
            FROM work_items wi
            JOIN documents d ON wi.doc_id = d.id
            WHERE wi.id = ?
        ''', (locked_item_id,))
        
        row = cursor.fetchone()
        conn.close()
        
        if not row:
            return {
                'items': [],
                'total_count': 0,
                'total_pages': 0,
                'current_page': 1,
                'page_size': 1,
                'has_next': False,
                'has_prev': False,
                'remaining_count': 0
            }
        
        # Parse KIE data
        section_type = row[2]
        kie_data = {}
        
        if section_type == 'header':
            kie_raw = row[10]
        elif section_type == 'content':
            kie_raw = row[11]
        elif section_type == 'full':
            kie_raw = row[10]
        else:
            kie_raw = None
        
        if kie_raw:
            try:
                if isinstance(kie_raw, str):
                    kie_data = json.loads(kie_raw)
                elif isinstance(kie_raw, dict):
                    kie_data = kie_raw
            except (json.JSONDecodeError, Exception):
                kie_data = {}
        
        item = {
            'id': row[0],
            'doc_id': row[1],
            'section_type': section_type,
            'status': row[3],
            'review_comment': row[4],
            'reviewed_by': row[5],
            'reviewed_at': row[6].isoformat() if row[6] else None,
            'completed_at': row[7].isoformat() if row[7] else None,
            'filename': row[8],
            'image_path': row[9],
            'kie_data': kie_data,
            'document_original_type': row[12] or 'Bản chính',
            'document_note': row[13] or '',
            'file_path': row[14] if len(row) > 14 else None
        }
        
        print(f"🔒 Locked item {item['id']} for cooperator {cooperator_id}, remaining: {remaining_count}")
        
        return {
            'items': [item],
            'total_count': remaining_count,
            'total_pages': remaining_count,
            'current_page': 1,
            'page_size': 1,
            'has_next': remaining_count > 1,
            'has_prev': False,
            'remaining_count': remaining_count
        }

    def get_rejected_items(self, cooperator_id, search_term='', page=1, page_size=10):
        """Lấy danh sách mảnh bị rejected của cooperator"""
        return self.db.get_work_items_paginated(
            cooperator_id=cooperator_id,
            status_filter='rejected',
            search_term=search_term,
            page=page,
            page_size=page_size
        )

    def get_work_item_detail(self, item_id, user_id, user_role):
        """Lấy chi tiết work item để edit với thông tin external source và is_fragmented"""
        conn = self.db.get_connection()
        cursor = conn.cursor()
        
        # ✅ THÊM document_original_type, document_note, is_fragmented, file_path VÀO SELECT
        cursor.execute('''
            SELECT wi.id, wi.package_id, wi.doc_id, wi.section_type, wi.status,
                wi.review_comment, wi.completed_at,
                d.filename, wp.name as package_name, wp.cooperator_id, wp.customer_id,
                CASE WHEN wi.section_type = 'header' THEN d.image_header_path 
                     WHEN wi.section_type = 'full' THEN d.file_path
                     ELSE d.image_content_path END as image_path,
                CASE WHEN wi.section_type = 'header' THEN d.kie_header_data 
                     WHEN wi.section_type = 'full' THEN d.kie_header_data
                     ELSE d.kie_content_data END as kie_data,
                d.vanBanId, d.kie_header_data as header_metadata,
                d.document_original_type, d.document_note,
                wp.is_fragmented, d.file_path
            FROM work_items wi
            JOIN documents d ON wi.doc_id = d.id  
            JOIN work_packages wp ON wi.package_id = wp.id
            WHERE wi.id = ?
        ''', (item_id,))
        
        result = cursor.fetchone()
        conn.close()
        
        if not result:
            raise ValueError('Work item not found')
        
        # ✅ KIỂM TRA QUYỀN TRUY CẬP
        cooperator_id = result[9]
        customer_id = result[10]
        package_id = result[1]
        
        if user_role == 'cooperator':
            # Kiểm tra qua junction table
            conn2 = self.db.get_connection()
            cursor2 = conn2.cursor()
            cursor2.execute('''
                SELECT COUNT(*) FROM work_package_cooperators 
                WHERE package_id = ? AND cooperator_id = ?
            ''', (package_id, user_id))
            has_access = cursor2.fetchone()[0] > 0
            conn2.close()
            
            # Fallback: check cột cooperator_id cũ
            if not has_access and cooperator_id != user_id:
                raise PermissionError('You can only access your assigned work items')
        elif user_role == 'customer' and customer_id != user_id:
            raise PermissionError('You can only access your own packages')
        
        # ✅ PARSE KIE DATA VÀ EXTERNAL INFO
        raw_kie_data = result[12]
        van_ban_id = result[13]
        header_metadata_raw = result[14]
        document_original_type = result[15] or 'Bản chính'
        document_note = result[16] or ''
        is_fragmented = bool(result[17]) if result[17] is not None else True
        file_path = result[18]
        
        print(f"🔍 DEBUG - Item {item_id} external info:")
        print(f"   vanBanId: {van_ban_id}")
        print(f"   document_original_type: {document_original_type}")
        print(f"   document_note: {document_note}")
        print(f"   is_fragmented: {is_fragmented}")
        print(f"   section_type: {result[3]}")
        
        # Parse KIE data
        import json
        kie_data = {}
        
        if raw_kie_data:
            try:
                if isinstance(raw_kie_data, str):
                    kie_data = json.loads(raw_kie_data)
                elif isinstance(raw_kie_data, dict):
                    kie_data = raw_kie_data
                else:
                    kie_data = {}
                    
            except (json.JSONDecodeError, Exception) as e:
                print(f"❌ Error parsing KIE data: {e}")
                kie_data = {}
        
        # ✅ PARSE EXTERNAL METADATA
        external_source = False
        if header_metadata_raw:
            try:
                if isinstance(header_metadata_raw, str):
                    header_metadata = json.loads(header_metadata_raw)
                elif isinstance(header_metadata_raw, dict):
                    header_metadata = header_metadata_raw
                else:
                    header_metadata = {}
                
                external_source = (
                    header_metadata.get('external_source', False) or
                    bool(van_ban_id)
                )
            except:
                external_source = bool(van_ban_id)
        else:
            external_source = bool(van_ban_id)
        
        print(f"✅ Item {item_id} - External source: {external_source}")
        
        return {
            'id': result[0],
            'package_id': result[1],
            'doc_id': result[2],
            'section_type': result[3],
            'status': result[4],
            'review_comment': result[5],
            'completed_at': result[6].isoformat() if result[6] else None,
            'filename': result[7],
            'package_name': result[8],
            'image_path': result[11],
            'kie_data': kie_data,
            'external_source': external_source,
            'van_ban_id': van_ban_id,
            'document_original_type': document_original_type,
            'document_note': document_note,
            'is_fragmented': is_fragmented,  # ✅ MỚI
            'file_path': file_path  # ✅ MỚI - ĐỂ HIỂN THỊ PDF FULL
        }

    def get_package_navigation_items(self, package_id, current_item_id):
        """Lấy danh sách items trong package để navigation"""
        conn = self.db.get_connection()
        cursor = conn.cursor()
        
        cursor.execute('''
            SELECT wi.id, d.filename, wi.section_type, wi.status
            FROM work_items wi
            JOIN documents d ON wi.doc_id = d.id
            WHERE wi.package_id = ?
            ORDER BY d.filename, wi.section_type
        ''', (package_id,))
        
        items = cursor.fetchall()
        conn.close()
        
        nav_items = []
        current_index = -1
        
        for i, item in enumerate(items):
            nav_items.append({
                'id': item[0],
                'filename': item[1],
                'section_type': item[2],
                'status': item[3]
            })
            
            if item[0] == current_item_id:
                current_index = i
        
        # Tìm previous và next
        prev_item = nav_items[current_index - 1] if current_index > 0 else None
        next_item = nav_items[current_index + 1] if current_index < len(nav_items) - 1 else None
        
        return {
            'items': nav_items,
            'current_index': current_index,
            'prev_item': prev_item,
            'next_item': next_item,
            'total_items': len(nav_items)
        }

    def sync_approved_document_to_external(self, item_id, kie_data_override=None):
        """Đồng bộ document đã phê duyệt lên external server với smart summary merge"""
        try:
            # Lấy thông tin work item và document
            conn = self.db.get_connection()
            cursor = conn.cursor()
            
            cursor.execute('''
                SELECT wi.doc_id, wi.section_type, d.vanBanId, d.filename,
                    d.kie_header_data, d.kie_content_data
                FROM work_items wi
                JOIN documents d ON wi.doc_id = d.id
                WHERE wi.id = ?
            ''', (item_id,))
            
            result = cursor.fetchone()
            conn.close()
            
            if not result:
                print(f"❌ Work item {item_id} not found for external sync")
                return False
            
            doc_id, section_type, van_ban_id, filename = result[:4]
            kie_header_raw, kie_content_raw = result[4:6]
            
            # ✅ KIỂM TRA CÓ vanBanId KHÔNG (CHỈ EXTERNAL FILES MỚI SYNC)
            if not van_ban_id:
                print(f"⚠️ Document {filename} không có vanBanId - bỏ qua external sync")
                return True  # Not an error, just skip
            
            # ✅ SMART MERGE KIE DATA - ƯU TIÊN HEADER → CONTENT
            combined_kie = {}
            header_summary = ""
            content_summary = ""
            
            # Parse existing KIE data từ database
            if kie_header_raw:
                try:
                    header_kie = json.loads(kie_header_raw) if isinstance(kie_header_raw, str) else kie_header_raw
                    combined_kie.update(header_kie)
                    header_summary = header_kie.get('summary', '').strip()
                except Exception as e:
                    print(f"⚠️ Error parsing header KIE: {e}")
            
            if kie_content_raw:
                try:
                    content_kie = json.loads(kie_content_raw) if isinstance(kie_content_raw, str) else kie_content_raw
                    content_summary = content_kie.get('summary', '').strip()
                    
                    # ✅ SMART MERGE - CHỈ CẬP NHẬT NẾU HEADER KHÔNG CÓ
                    for key, value in content_kie.items():
                        if key not in combined_kie or not combined_kie[key]:
                            combined_kie[key] = value
                            
                except Exception as e:
                    print(f"⚠️ Error parsing content KIE: {e}")
            
            # ✅ FINAL SUMMARY SELECTION - ƯU TIÊN HEADER → CONTENT
            final_summary = ""
            if header_summary and header_summary not in ['', 'N/A', 'Không có', 'null']:
                final_summary = header_summary
                summary_source = "header"
            elif content_summary and content_summary not in ['', 'N/A', 'Không có', 'null']:
                final_summary = content_summary
                summary_source = "content"
            else:
                summary_source = "none"
            
            combined_kie['summary'] = final_summary
            
            # ✅ CHỈ OVERRIDE NẾU CÓ TRUYỀN DATA MỚI
            if kie_data_override:
                combined_kie.update(kie_data_override)
            
            print(f"📤 Syncing to external server:")
            print(f"   vanBanId: {van_ban_id}")
            print(f"   Summary source: {summary_source}")
            print(f"   Final summary: {final_summary[:50]}..." if final_summary else "   No summary")
            print(f"   Combined KIE fields: {len(combined_kie)}")
            
            # ✅ SYNC LEN EXTERNAL SERVER
            external_api = get_external_api()
            success = external_api.update_document(van_ban_id, combined_kie)
            
            if success:
                print(f"✅ External sync successful for document {filename} (vanBanId: {van_ban_id})")
            else:
                print(f"❌ External sync failed for document {filename} (vanBanId: {van_ban_id})")
            
            return success
            
        except Exception as e:
            print(f"❌ Error in external sync for item {item_id}: {e}")
            import traceback
            traceback.print_exc()
            return False

    def update_item_status(self, item_id, status, kie_data=None, user_id=None, reviewed_by=None, review_comment=None):
        """Cập nhật trạng thái work item với external sync"""
        
        # Validate status transition
        valid_statuses = ['pending', 'completed', 'recompleted', 'rejected', 'approved']
        if status not in valid_statuses:
            raise ValueError(f'Invalid status: {status}')
        
        # ✅ CẬP NHẬT WORK ITEM STATUS
        if reviewed_by and status in ['approved', 'rejected']:
            # Customer review action
            self.db.update_work_item_status(
                item_id, status, 
                reviewed_by=reviewed_by, 
                review_comment=review_comment or ''
            )
        else:
            # Cooperator action
            self.db.update_work_item_status(item_id, status)
        
        # ✅ CHỈ CẬP NHẬT KIE DATA KHI COOPERATOR HOÀN THÀNH
        if kie_data and status in ['completed', 'recompleted']:
            self._update_document_kie_data(item_id, kie_data)
        
        # ✅ EXTERNAL SYNC KHI APPROVED - SỬ DỤNG KIE CÓ SẴN
        if status == 'approved':
            print(f"🔄 Starting external sync for approved item {item_id}")
            try:
                # ✅ KHÔNG TRUYỀN kie_data MỚI - SỬ DỤNG DATA CÓ SẴN TRONG DB
                import threading
                sync_thread = threading.Thread(
                    target=self.sync_approved_document_to_external,
                    args=(item_id, None)  # ✅ TRUYỀN None CHO kie_data
                )
                sync_thread.daemon = True
                sync_thread.start()
                print(f"🚀 External sync started in background for item {item_id}")
            except Exception as e:
                print(f"⚠️ Failed to start external sync thread: {e}")
        
        # Cập nhật package status nếu cần
        self._update_package_status_if_needed(item_id)
        
        return {'success': True, 'message': f'Đã cập nhật trạng thái thành {status}'}
    def bulk_review_items(self, item_ids, review_status, review_comment, reviewed_by):
        """Review nhiều items cùng lúc với external batch sync"""
        
        if review_status not in ['approved', 'rejected']:
            raise ValueError('Review status must be approved or rejected')
        
        updated_count = 0
        approved_items = []  # ✅ TRACK APPROVED ITEMS CHO BATCH SYNC
        
        for item_id in item_ids:
            try:
                self.db.update_work_item_status(
                    item_id, review_status, 
                    reviewed_by=reviewed_by, 
                    review_comment=review_comment
                )
                
                # Cập nhật package status
                self._update_package_status_if_needed(item_id)
                updated_count += 1
                
                # ✅ COLLECT APPROVED ITEMS
                if review_status == 'approved':
                    approved_items.append(item_id)
                    
            except Exception as e:
                print(f"Error updating item {item_id}: {e}")
                continue
        
        # ✅ BATCH EXTERNAL SYNC CHO CÁC APPROVED ITEMS
        if approved_items:
            print(f"🔄 Starting batch external sync for {len(approved_items)} approved items")
            try:
                import threading
                batch_sync_thread = threading.Thread(
                    target=self._batch_sync_approved_items,
                    args=(approved_items,)
                )
                batch_sync_thread.daemon = True
                batch_sync_thread.start()
                print(f"🚀 Batch external sync started in background")
            except Exception as e:
                print(f"⚠️ Failed to start batch external sync: {e}")
        
        return {
            'success': True, 
            'updated_count': updated_count,
            'message': f'Đã review {updated_count} mảnh thành {review_status}'
        }

    def _batch_sync_approved_items(self, item_ids):
        """Sync nhiều approved items lên external server với smart summary merge"""
        try:
            conn = self.db.get_connection()
            cursor = conn.cursor()
            
            # Lấy thông tin tất cả items cần sync
            placeholders = ','.join(['?' for _ in item_ids])
            cursor.execute(f'''
                SELECT wi.id, wi.doc_id, wi.section_type, d.vanBanId, d.filename,
                    d.kie_header_data, d.kie_content_data
                FROM work_items wi
                JOIN documents d ON wi.doc_id = d.id
                WHERE wi.id IN ({placeholders}) AND d.vanBanId IS NOT NULL AND d.vanBanId != ''
            ''', item_ids)
            
            results = cursor.fetchall()
            conn.close()
            
            if not results:
                print(f"⚠️ No items with vanBanId found for batch external sync")
                return
            
            # ✅ CHUẨN BỊ DATA CHO BATCH UPDATE VỚI SMART MERGE
            documents_data = []
            
            for result in results:
                item_id, doc_id, section_type, van_ban_id, filename = result[:5]
                kie_header_raw, kie_content_raw = result[5:7]
                
                # ✅ SMART MERGE KIE DATA - ƯU TIÊN HEADER → CONTENT
                combined_kie = {}
                header_summary = ""
                content_summary = ""
                
                # Parse header KIE
                if kie_header_raw:
                    try:
                        header_kie = json.loads(kie_header_raw) if isinstance(kie_header_raw, str) else kie_header_raw
                        combined_kie.update(header_kie)
                        header_summary = header_kie.get('summary', '').strip()
                    except Exception as e:
                        print(f"⚠️ Error parsing header KIE for {filename}: {e}")
                
                # Parse content KIE
                if kie_content_raw:
                    try:
                        content_kie = json.loads(kie_content_raw) if isinstance(kie_content_raw, str) else kie_content_raw
                        content_summary = content_kie.get('summary', '').strip()
                        
                        # ✅ MERGE CONTENT KIE, BUT PRESERVE HEADER FIELDS
                        for key, value in content_kie.items():
                            # ✅ CHỈ CẬP NHẬT NẾU HEADER KHÔNG CÓ HOẶC TRỐNG
                            if key not in combined_kie or not combined_kie[key]:
                                combined_kie[key] = value
                                
                    except Exception as e:
                        print(f"⚠️ Error parsing content KIE for {filename}: {e}")
                
                # ✅ FINAL SUMMARY SELECTION - ƯU TIÊN HEADER → CONTENT
                final_summary = ""
                if header_summary and header_summary not in ['', 'N/A', 'Không có', 'null']:
                    final_summary = header_summary
                    summary_source = "header"
                elif content_summary and content_summary not in ['', 'N/A', 'Không có', 'null']:
                    final_summary = content_summary
                    summary_source = "content"
                else:
                    summary_source = "none"
                
                # ✅ ENSURE FINAL SUMMARY IS IN COMBINED DATA
                combined_kie['summary'] = final_summary
                
                print(f"🔄 Batch item {item_id} ({filename}):")
                print(f"   Summary source: {summary_source}")
                print(f"   Final summary: {final_summary[:50]}..." if final_summary else "   No summary")
                
                documents_data.append({
                    'van_ban_id': van_ban_id,
                    'kie_data': combined_kie,
                    'filename': filename,
                    'item_id': item_id
                })
            
            if documents_data:
                # ✅ GỌI BATCH UPDATE API VỚI SMART SUMMARY
                external_api = get_external_api()
                result = external_api.update_documents_batch(documents_data)
                
                if result.get('success'):
                    print(f"✅ Batch external sync successful for {len(documents_data)} documents")
                else:
                    print(f"❌ Batch external sync failed: {result.get('error', 'Unknown error')}")
                    
                    # ✅ FALLBACK: SYNC TỪNG ITEM VỚI SMART SUMMARY
                    print(f"🔄 Fallback to individual sync with smart summary...")
                    success_count = 0
                    
                    for doc_data in documents_data:
                        try:
                            success = external_api.update_document(doc_data['van_ban_id'], doc_data['kie_data'])
                            if success:
                                success_count += 1
                                print(f"✅ Individual sync successful: {doc_data['filename']}")
                            else:
                                print(f"❌ Individual sync failed: {doc_data['filename']}")
                        except Exception as e:
                            print(f"❌ Individual sync error for {doc_data['filename']}: {e}")
                    
                    print(f"📊 Fallback result: {success_count}/{len(documents_data)} successful")
            
        except Exception as e:
            print(f"❌ Error in batch external sync: {e}")
            import traceback
            traceback.print_exc()

    def delete_package_safely(self, package_id, user_id):
        """Xóa package một cách an toàn (kiểm tra ràng buộc)"""
        conn = self.db.get_connection()
        cursor = conn.cursor()
        
        try:
            # Kiểm tra package tồn tại và quyền
            cursor.execute('''
                SELECT customer_id, name FROM work_packages 
                WHERE id = ?
            ''', (package_id,))
            
            result = cursor.fetchone()
            if not result:
                raise ValueError('Package not found')
            
            customer_id, package_name = result
            if customer_id != user_id:
                raise PermissionError('You can only delete your own packages')
            
            # Đếm work_items trong package
            cursor.execute('SELECT COUNT(*) FROM work_items WHERE package_id = ?', (package_id,))
            item_count = cursor.fetchone()[0]
            
            # Xóa work_items trước
            cursor.execute('DELETE FROM work_items WHERE package_id = ?', (package_id,))
            items_deleted = cursor.rowcount
            
            # Xóa package
            cursor.execute('DELETE FROM work_packages WHERE id = ?', (package_id,))
            
            conn.commit()
            
            print(f"✅ Đã xóa package '{package_name}' và {items_deleted} work_items")
            
            return {
                'success': True, 
                'message': f'Đã xóa gói "{package_name}" và {items_deleted} mảnh thành công',
                'deleted_items': items_deleted
            }
            
        except Exception as e:
            conn.rollback()
            print(f"❌ Lỗi xóa package {package_id}: {e}")
            raise
        finally:
            conn.close()

    def delete_package(self, package_id, user_id):
        """Wrapper cho delete_package_safely để tương thích với code cũ"""
        return self.delete_package_safely(package_id, user_id)

    def get_cooperators_for_customer(self, customer_id):
        """Lấy danh sách cooperators mà customer quản lý"""
        conn = self.db.get_connection()
        cursor = conn.cursor()
        
        cursor.execute('''
            SELECT u.id, u.full_name, u.username, ur.created_at
            FROM users u
            JOIN user_relationships ur ON u.id = ur.cooperator_id
            WHERE ur.customer_id = ? AND u.status = 'active' AND u.role = 'cooperator'
            ORDER BY u.full_name
        ''', (customer_id,))
        
        cooperators = []
        for row in cursor.fetchall():
            cooperators.append({
                'id': row[0],
                'full_name': row[1],
                'username': row[2],
                'assigned_date': row[3].isoformat() if row[3] else None
            })
        
        conn.close()
        return cooperators

    # Private helper methods
    def _check_customer_cooperator_relationship(self, customer_id, cooperator_id):
        """Kiểm tra quan hệ customer-cooperator"""
        conn = self.db.get_connection()
        cursor = conn.cursor()
        
        cursor.execute('''
            SELECT COUNT(*) FROM user_relationships 
            WHERE customer_id = ? AND cooperator_id = ?
        ''', (customer_id, cooperator_id))
        
        count = cursor.fetchone()[0]
        conn.close()
        return count > 0

    def _update_document_kie_data(self, item_id, kie_data):
        """Cập nhật KIE data vào document gốc"""
        conn = self.db.get_connection()
        cursor = conn.cursor()
        
        # Lấy thông tin work item
        cursor.execute('SELECT doc_id, section_type FROM work_items WHERE id = ?', (item_id,))
        result = cursor.fetchone()
        
        if result:
            doc_id, section_type = result
            
            # Cập nhật KIE data tương ứng
            if section_type == 'header':
                self.db.update_document(doc_id, kie_header_data=kie_data)
            else:  # content
                self.db.update_document(doc_id, kie_content_data=kie_data)
        
        conn.close()

    def _update_package_status_if_needed(self, item_id):
        """Cập nhật status của package dựa trên trạng thái các items"""
        conn = self.db.get_connection()
        cursor = conn.cursor()
        
        # Lấy package_id từ item_id
        cursor.execute('SELECT package_id FROM work_items WHERE id = ?', (item_id,))
        result = cursor.fetchone()
        
        if not result:
            conn.close()
            return
            
        package_id = result[0]
        
        # Đếm trạng thái các items trong package
        cursor.execute('''
            SELECT 
                COUNT(CASE WHEN status = 'pending' THEN 1 END) as pending_count,
                COUNT(CASE WHEN status = 'completed' THEN 1 END) as completed_count,
                COUNT(CASE WHEN status = 'recompleted' THEN 1 END) as recompleted_count,
                COUNT(CASE WHEN status = 'rejected' THEN 1 END) as rejected_count,
                COUNT(CASE WHEN status = 'approved' THEN 1 END) as approved_count,
                COUNT(*) as total_count
            FROM work_items WHERE package_id = ?
        ''', (package_id,))
        
        counts = cursor.fetchone()
        pending, completed, recompleted, rejected, approved, total = counts
        
        # ✅ SỬA LOGIC: Xác định package status
        if approved == total:
            # Tất cả đã được approved
            new_status = 'completed'
        elif rejected > 0:
            # ✅ CÓ MẢNH BỊ TỪ CHỐI = ĐANG XỬ LÝ (có từ chối)
            new_status = 'has_rejected'
        elif (completed > 0 or recompleted > 0) and pending == 0 and rejected == 0:
            # Tất cả đã completed/recompleted, chờ review
            new_status = 'under_review'
        elif pending > 0:
            # Còn mảnh chưa làm
            new_status = 'in_progress'
        else:
            # Default
            new_status = 'in_progress'
        
        # Cập nhật package status
        cursor.execute('UPDATE work_packages SET status = ? WHERE id = ?', (new_status, package_id))
        
        conn.commit()
        conn.close()
        
        print(f"✅ Updated package {package_id} status to {new_status}")
        print(f"   Stats: {approved}/{total} approved, {rejected} rejected, {completed+recompleted} completed, {pending} pending")

    def get_package_detail(self, package_id, user_id, user_role):
        """Lấy chi tiết package với quyền truy cập"""
        conn = self.db.get_connection()
        cursor = conn.cursor()
        
        cursor.execute('''
            SELECT wp.id, wp.name, wp.customer_id, wp.cooperator_id, wp.total_items,
                   wp.status, wp.created_at, c.full_name as customer_name, co.full_name as cooperator_name
            FROM work_packages wp
            JOIN users c ON wp.customer_id = c.id
            JOIN users co ON wp.cooperator_id = co.id
            WHERE wp.id = ?
        ''', (package_id,))
        
        result = cursor.fetchone()
        
        if not result:
            conn.close()
            raise ValueError('Package not found')
        
        # Kiểm tra quyền truy cập
        if user_role == 'customer' and result[2] != user_id:
            conn.close()
            raise PermissionError('You can only access your own packages')
        elif user_role == 'cooperator':
            # Kiểm tra qua junction table
            cursor.execute('''
                SELECT COUNT(*) FROM work_package_cooperators 
                WHERE package_id = ? AND cooperator_id = ?
            ''', (package_id, user_id))
            has_access = cursor.fetchone()[0] > 0
            # Fallback: check cột cooperator_id cũ
            if not has_access and result[3] != user_id:
                conn.close()
                raise PermissionError('You can only access packages assigned to you')
        
        # Lấy tất cả cooperators từ junction table
        cursor.execute('''
            SELECT u.id, u.full_name 
            FROM work_package_cooperators wpc
            JOIN users u ON wpc.cooperator_id = u.id
            WHERE wpc.package_id = ?
            ORDER BY u.full_name
        ''', (package_id,))
        coop_rows = cursor.fetchall()
        
        if coop_rows:
            cooperator_names = [r[1] for r in coop_rows]
            cooperator_ids_list = [r[0] for r in coop_rows]
        else:
            cooperator_names = [result[8]] if result[8] else []
            cooperator_ids_list = [result[3]] if result[3] else []
        
        # Lấy status counts
        cursor.execute('''
            SELECT 
                COUNT(CASE WHEN status = 'pending' THEN 1 END) as pending_count,
                COUNT(CASE WHEN status = 'completed' THEN 1 END) as completed_count,
                COUNT(CASE WHEN status = 'recompleted' THEN 1 END) as recompleted_count,
                COUNT(CASE WHEN status = 'rejected' THEN 1 END) as rejected_count,
                COUNT(CASE WHEN status = 'approved' THEN 1 END) as approved_count
            FROM work_items WHERE package_id = ?
        ''', (package_id,))
        
        counts = cursor.fetchone()
        conn.close()
        
        return {
            'id': result[0],
            'name': result[1],
            'customer_id': result[2],
            'cooperator_id': result[3],
            'cooperator_ids': cooperator_ids_list,
            'total_items': result[4],
            'status': result[5],
            'created_at': result[6].isoformat() if result[6] else None,
            'customer_name': result[7],
            'cooperator_name': ', '.join(cooperator_names),
            'cooperator_names': cooperator_names,
            'status_counts': {
                'pending': counts[0],
                'completed': counts[1],
                'recompleted': counts[2],
                'rejected': counts[3],
                'approved': counts[4]
            }
        }
    def get_customer_batches_with_available_count(self, customer_id):
        """Lấy danh sách batch của customer với số mảnh khả dụng"""
        return self.db.get_customer_batches_with_available_count(customer_id)

    def get_available_pieces_count_by_batches(self, customer_id, batch_ids):
        """Đếm mảnh khả dụng theo batch IDs"""
        return self.db.get_available_pieces_count_by_batches(customer_id, batch_ids)

    def create_package_with_batch_filter(self, name, customer_id, cooperator_ids, item_count, batch_filter=None, is_fragmented=True):
        """Tạo gói công việc với filter batch và chế độ phân mảnh"""
        
        # Validate inputs
        if not name or not name.strip():
            raise ValueError('Tên gói không được để trống')
        
        # Hỗ trợ cả string và list
        if isinstance(cooperator_ids, str):
            cooperator_ids = [cooperator_ids]
        
        if not cooperator_ids:
            raise ValueError('Phải chọn ít nhất 1 cooperator')
        
        # Kiểm tra relationship cho tất cả cooperators
        for coop_id in cooperator_ids:
            if not self._check_customer_cooperator_relationship(customer_id, coop_id):
                raise PermissionError('Bạn không có quyền gán công việc cho cooperator này')
        
        # ✅ Auto pick pieces VỚI BATCH FILTER
        pieces = self.auto_pick_pieces_with_batch_filter(customer_id, item_count, batch_filter)
        
        # Tạo package với chế độ phân mảnh
        package_id = str(uuid.uuid4())
        self.db.create_work_package(package_id, name.strip(), customer_id, cooperator_ids, item_count, is_fragmented)
        
        # ✅ TẠO WORK ITEMS TÙY THEO CHẾ ĐỘ PHÂN MẢNH
        items_data = []
        
        if is_fragmented:
            # Chế độ phân mảnh: Tạo items theo pieces (header/content)
            for piece in pieces:
                item_id = str(uuid.uuid4())
                items_data.append({
                    'id': item_id,
                    'package_id': package_id,
                    'doc_id': piece['doc_id'],
                    'section_type': piece['section_type']
                })
        else:
            # ✅ Chế độ không phân mảnh: Tạo 1 item cho mỗi document (full)
            seen_docs = set()
            for piece in pieces:
                doc_id = piece['doc_id']
                if doc_id not in seen_docs:
                    item_id = str(uuid.uuid4())
                    items_data.append({
                        'id': item_id,
                        'package_id': package_id,
                        'doc_id': doc_id,
                        'section_type': 'full'
                    })
                    seen_docs.add(doc_id)
        
        self.db.create_work_items(items_data)
        
        mode_text = "phân mảnh (Header + Content)" if is_fragmented else "không phân mảnh (Full PDF)"
        return {
            'success': True,
            'package_id': package_id,
            'name': name.strip(),
            'total_items': len(items_data),
            'is_fragmented': is_fragmented,
            'message': f'Đã tạo gói "{name}" với {len(items_data)} mảnh - Chế độ: {mode_text}'
        }

    def auto_pick_pieces_with_batch_filter(self, customer_id, requested_count, batch_filter=None):
        """Tự động chọn mảnh với filter batch"""
        if batch_filter:
            # Đếm available với batch filter
            available_count = self.db.get_available_pieces_count_by_batches(
                customer_id, batch_filter
            )
        else:
            # Đếm tất cả
            available_count = self.db.get_available_pieces_count(customer_id)
        
        if requested_count > available_count:
            raise ValueError(f'Chỉ có {available_count} mảnh khả dụng, không thể tạo gói {requested_count} mảnh')
        
        # Lấy pieces với batch filter
        pieces = self.get_available_pieces_with_batch_filter(customer_id, batch_filter, limit=requested_count)
        
        if len(pieces) < requested_count:
            raise ValueError(f'Không đủ mảnh khả dụng. Yêu cầu: {requested_count}, Có: {len(pieces)}')
            
        return pieces

    def get_available_pieces_with_batch_filter(self, customer_id, batch_filter=None, limit=None):
        """Lấy mảnh khả dụng với filter batch"""
        conn = self.db.get_connection()
        cursor = conn.cursor()
        
        try:
            # Build query với batch filter
            if batch_filter and len(batch_filter) > 0:
                placeholders = ','.join(['?' for _ in batch_filter])
                batch_condition = f'AND b.id IN ({placeholders})'
                params = [customer_id] + batch_filter
            else:
                batch_condition = ''
                params = [customer_id]
            
            # Lấy documents với batch filter
            cursor.execute(f'''
                SELECT 
                    d.id, d.filename, d.image_header_path, d.image_content_path,
                    d.kie_header_data, d.kie_content_data, d.document_type,
                    b.name as batch_name
                FROM documents d
                JOIN batches b ON d.batch_id = b.id
                WHERE b.created_by = ? {batch_condition}
            ''', params)
            
            documents = cursor.fetchall()
            
            # Lấy assigned pieces
            cursor.execute('''
                SELECT doc_id, section_type 
                FROM work_items 
                WHERE package_id IN (
                    SELECT id FROM work_packages WHERE customer_id = ?
                )
            ''', (customer_id,))
            
            # ✅ FIX: Build assigned map with section types
            assigned_pieces = set()
            assigned_full_docs = set()  # Documents với section_type='full'
            
            for row in cursor.fetchall():
                doc_id, section_type = row[0], row[1]
                assigned_pieces.add((doc_id, section_type))
                
                # ✅ Nếu document đã được gán 'full', đánh dấu
                if section_type == 'full':
                    assigned_full_docs.add(doc_id)
            
            # Filter available pieces
            available_pieces = []
            
            for doc in documents:
                doc_id, filename, header_path, content_path, kie_header_data, kie_content_data, doc_type, batch_name = doc
                
                # ✅ FIX: Bỏ qua document đã được gán 'full'
                if doc_id in assigned_full_docs:
                    continue
                
                # Header piece
                if header_path and kie_header_data:
                    piece_key = (doc_id, 'header')
                    if (piece_key not in assigned_pieces and 
                        self.db._has_valid_kie_data(kie_header_data, 'header')):
                        available_pieces.append({
                            'doc_id': doc_id,
                            'filename': filename,
                            'section_type': 'header',
                            'image_path': header_path,
                            'kie_data': kie_header_data,
                            'document_type': doc_type,
                            'batch_name': batch_name
                        })
                
                # Content piece
                if content_path and kie_content_data:
                    piece_key = (doc_id, 'content')
                    if (piece_key not in assigned_pieces and 
                        self.db._has_valid_kie_data(kie_content_data, 'content')):
                        available_pieces.append({
                            'doc_id': doc_id,
                            'filename': filename,
                            'section_type': 'content',
                            'image_path': content_path,
                            'kie_data': kie_content_data,
                            'document_type': doc_type,
                            'batch_name': batch_name
                        })
            
            # Apply limit
            if limit and len(available_pieces) > limit:
                import random
                available_pieces = random.sample(available_pieces, limit)
            
            return available_pieces
            
        except Exception as e:
            print(f"❌ Lỗi get_available_pieces_with_batch_filter: {e}")
            return []
        finally:
            conn.close()