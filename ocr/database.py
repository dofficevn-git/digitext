import pyodbc
import json
from datetime import datetime
import os

class DatabaseManager:
    def __init__(self, config):
        self.config = config
    
    def get_connection(self):
        """Tạo kết nối đến SQL Server"""
        if 'trusted_connection' in self.config:
            # Windows Authentication
            conn_str = f"""
                DRIVER={self.config['driver']};
                SERVER={self.config['server']};
                DATABASE={self.config['database']};
                Trusted_Connection=yes;
            """
        else:
            # SQL Authentication
            conn_str = f"""
                DRIVER={self.config['driver']};
                SERVER={self.config['server']};
                DATABASE={self.config['database']};
                UID={self.config['username']};
                PWD={self.config['password']};
            """
        
        conn = pyodbc.connect(conn_str)
    
        # ✅ SET COMPATIBILITY OPTIONS
        datetime_as_string = os.getenv('DB_DATETIME_AS_STRING', 'false').lower() == 'true'
        if datetime_as_string:
            # Force datetime columns to return as strings
            conn.add_output_converter(pyodbc.SQL_TYPE_TIMESTAMP, str)
            conn.add_output_converter(pyodbc.SQL_TYPE_DATE, str)
        
        return conn
    
    def init_database(self):
        """Khởi tạo database và tables"""
        try:
            conn = self.get_connection()
            cursor = conn.cursor()
            
            # Tạo bảng batches TRƯỚC
            cursor.execute('''
                IF NOT EXISTS (SELECT * FROM sysobjects WHERE name='batches' AND xtype='U')
                CREATE TABLE batches (
                    id NVARCHAR(50) PRIMARY KEY,
                    name NVARCHAR(255) NOT NULL,
                    created_at DATETIME2 DEFAULT GETDATE(),
                    total_files INT DEFAULT 0,
                    processed_files INT DEFAULT 0,
                    status NVARCHAR(50) DEFAULT 'pending'
                )
            ''')
            
            # Tạo bảng documents
            cursor.execute('''
                IF NOT EXISTS (SELECT * FROM sysobjects WHERE name='documents' AND xtype='U')
                CREATE TABLE documents (
                    id NVARCHAR(50) PRIMARY KEY,
                    batch_id NVARCHAR(50),
                    filename NVARCHAR(255) NOT NULL,
                    file_path NVARCHAR(500) NOT NULL,
                    document_type NVARCHAR(100),
                    status NVARCHAR(50) DEFAULT 'pending',
                    created_at DATETIME2 DEFAULT GETDATE(),
                    processed_at DATETIME2,
                    kie_data NTEXT,
                    cut_point INT DEFAULT 0,
                    image_full_path NVARCHAR(500),
                    image_header_path NVARCHAR(500),
                    image_content_path NVARCHAR(500),
                    header_detection NTEXT,
                    total_pages INT DEFAULT 1,
                    FOREIGN KEY (batch_id) REFERENCES batches(id)
                )
            ''')
            
            # Tạo bảng field_definitions
            cursor.execute('''
                IF NOT EXISTS (SELECT * FROM sysobjects WHERE name='field_definitions' AND xtype='U')
                CREATE TABLE field_definitions (
                    id INT IDENTITY(1,1) PRIMARY KEY,
                    field_key NVARCHAR(100) UNIQUE NOT NULL,
                    field_name NVARCHAR(255) NOT NULL,
                    field_type NVARCHAR(50) DEFAULT 'text',
                    default_value NTEXT,
                    required BIT DEFAULT 0,
                    order_index INT DEFAULT 0
                )
            ''')
            
            # Tạo bảng users
            cursor.execute('''
                IF NOT EXISTS (SELECT * FROM sysobjects WHERE name='users' AND xtype='U')
                CREATE TABLE users (
                    id NVARCHAR(50) PRIMARY KEY,
                    username NVARCHAR(100) UNIQUE NOT NULL,
                    password_hash NVARCHAR(255) NOT NULL,
                    full_name NVARCHAR(255) NOT NULL,
                    email NVARCHAR(255),
                    role NVARCHAR(50) NOT NULL DEFAULT 'cooperator',
                    status NVARCHAR(50) NOT NULL DEFAULT 'active',
                    created_at DATETIME2 DEFAULT GETDATE(),
                    updated_at DATETIME2 DEFAULT GETDATE(),
                    created_by NVARCHAR(50),
                    last_login DATETIME2
                )
            ''')
            
            # Tạo bảng user_relationships (customer quản lý cooperator)
            cursor.execute('''
                IF NOT EXISTS (SELECT * FROM sysobjects WHERE name='user_relationships' AND xtype='U')
                CREATE TABLE user_relationships (
                    id INT IDENTITY(1,1) PRIMARY KEY,
                    customer_id NVARCHAR(50) NOT NULL,
                    cooperator_id NVARCHAR(50) NOT NULL,
                    created_at DATETIME2 DEFAULT GETDATE(),
                    created_by NVARCHAR(50),
                    FOREIGN KEY (customer_id) REFERENCES users(id),
                    FOREIGN KEY (cooperator_id) REFERENCES users(id),
                    UNIQUE(customer_id, cooperator_id)
                )
            ''')
            # ✅ Tạo bảng work_packages
            cursor.execute('''
                IF NOT EXISTS (SELECT * FROM sysobjects WHERE name='work_packages' AND xtype='U')
                CREATE TABLE work_packages (
                    id NVARCHAR(50) PRIMARY KEY,
                    name NVARCHAR(255) NOT NULL,
                    customer_id NVARCHAR(50) NOT NULL,
                    cooperator_id NVARCHAR(50) NOT NULL,
                    total_items INT DEFAULT 0,
                    status NVARCHAR(50) DEFAULT 'in_progress',
                    is_fragmented BIT DEFAULT 1,
                    created_at DATETIME2 DEFAULT GETDATE(),
                    FOREIGN KEY (customer_id) REFERENCES users(id),
                    FOREIGN KEY (cooperator_id) REFERENCES users(id)
                )
            ''')
            
            # ✅ Tạo bảng work_package_cooperators (nhiều CTV cho 1 gói)
            cursor.execute('''
                IF NOT EXISTS (SELECT * FROM sysobjects WHERE name='work_package_cooperators' AND xtype='U')
                CREATE TABLE work_package_cooperators (
                    id NVARCHAR(50) PRIMARY KEY,
                    package_id NVARCHAR(50) NOT NULL,
                    cooperator_id NVARCHAR(50) NOT NULL,
                    created_at DATETIME2 DEFAULT GETDATE(),
                    FOREIGN KEY (package_id) REFERENCES work_packages(id) ON DELETE CASCADE,
                    FOREIGN KEY (cooperator_id) REFERENCES users(id),
                    UNIQUE(package_id, cooperator_id)
                )
            ''')

            # ✅ Tạo bảng work_items
            cursor.execute('''
                IF NOT EXISTS (SELECT * FROM sysobjects WHERE name='work_items' AND xtype='U')
                CREATE TABLE work_items (
                    id NVARCHAR(50) PRIMARY KEY,
                    package_id NVARCHAR(50) NOT NULL,
                    doc_id NVARCHAR(50) NOT NULL,
                    section_type NVARCHAR(20) NOT NULL,
                    status NVARCHAR(50) DEFAULT 'pending',
                    review_comment NTEXT,
                    reviewed_by NVARCHAR(50),
                    reviewed_at DATETIME2,
                    completed_at DATETIME2,
                    created_at DATETIME2 DEFAULT GETDATE(),
                    FOREIGN KEY (package_id) REFERENCES work_packages(id) ON DELETE CASCADE,
                    FOREIGN KEY (doc_id) REFERENCES documents(id),
                    FOREIGN KEY (reviewed_by) REFERENCES users(id),
                    CHECK (section_type IN ('header', 'content')),
                    CHECK (status IN ('pending', 'completed', 'recompleted', 'rejected', 'approved'))
                )
            ''')
            
            # Kiểm tra và thêm các cột mới nếu chưa có
            self._add_missing_columns(cursor)
            
            # Insert default field definitions nếu chưa có
            self._insert_default_fields(cursor)
            
            # Insert default admin user nếu chưa có
            self._insert_default_admin(cursor)
            
            conn.commit()
            conn.close()
            print("✅ Database initialized successfully")
            
        except Exception as e:
            print(f"❌ Error initializing database: {e}")
            raise


    def _insert_default_admin(self, cursor):
        """Insert default admin user nếu chưa có"""
        cursor.execute('SELECT COUNT(*) FROM users WHERE role = ?', ('admin',))
        count = cursor.fetchone()[0]
        
        if count == 0:
            import hashlib
            import uuid
            
            admin_id = str(uuid.uuid4())
            # Default password: admin123 (nên thay đổi sau khi login lần đầu)
            password_hash = hashlib.sha256('admin123'.encode()).hexdigest()
            
            cursor.execute('''
                INSERT INTO users (id, username, password_hash, full_name, email, role, status)
                VALUES (?, ?, ?, ?, ?, ?, ?)
            ''', (admin_id, 'admin', password_hash, 'System Administrator', 'admin@system.com', 'admin', 'active'))
            
            print("✅ Default admin user created (username: admin, password: admin123)")

    # User management methods
    def authenticate_user(self, username, password):
        """Xác thực user"""
        import hashlib
        
        conn = self.get_connection()
        cursor = conn.cursor()
        
        password_hash = hashlib.sha256(password.encode()).hexdigest()
        
        cursor.execute('''
            SELECT id, username, full_name, email, role, status 
            FROM users 
            WHERE username = ? AND password_hash = ? AND status = 'active'
        ''', (username, password_hash))
        
        user = cursor.fetchone()
        
        if user:
            # Update last login
            cursor.execute('UPDATE users SET last_login = GETDATE() WHERE id = ?', (user[0],))
            conn.commit()
            
            user_data = {
                'id': user[0],
                'username': user[1],
                'full_name': user[2],
                'email': user[3],
                'role': user[4],
                'status': user[5]
            }
        else:
            user_data = None
        
        conn.close()
        return user_data

    def get_users(self, role_filter=None, created_by=None):
        """Lấy danh sách users với safe datetime handling"""
        conn = self.get_connection()
        cursor = conn.cursor()
        
        query = '''
            SELECT u.id, u.username, u.full_name, u.email, u.role, u.status, 
                u.created_at, u.last_login, creator.full_name as created_by_name
            FROM users u
            LEFT JOIN users creator ON u.created_by = creator.id
            WHERE 1=1
        '''
        params = []
        
        if role_filter:
            query += ' AND u.role = ?'
            params.append(role_filter)
        
        if created_by:
            query += ' AND u.created_by = ?'
            params.append(created_by)
        
        query += ' ORDER BY u.created_at DESC'
        
        cursor.execute(query, params)
        
        users = []
        for row in cursor.fetchall():
            # ✅ SAFE DATETIME CONVERSION
            created_at = self._safe_datetime_convert(row[6])
            last_login = self._safe_datetime_convert(row[7])
            
            users.append({
                'id': row[0],
                'username': row[1],
                'full_name': row[2],
                'email': row[3],
                'role': row[4],
                'status': row[5],
                'created_at': created_at,
                'last_login': last_login,
                'created_by_name': row[8]
            })
        
        conn.close()
        return users

    def _safe_datetime_convert(self, dt_value):
        """Safely convert datetime value to ISO string"""
        if dt_value is None:
            return None
        
        # ✅ NẾU ĐÃ LÀ STRING - TRẢ VỀ LUÔN
        if isinstance(dt_value, str):
            return dt_value
        
        # ✅ NẾU LÀ DATETIME OBJECT - CONVERT TO ISO
        if hasattr(dt_value, 'isoformat'):
            return dt_value.isoformat()
        
        # ✅ FALLBACK - CONVERT TO STRING
        return str(dt_value)

    def create_user(self, username, password, full_name, email, role, created_by):
        """Tạo user mới với validation tốt hơn"""
        conn = self.get_connection()
        cursor = conn.cursor()
        
        try:
            # ✅ VALIDATE INPUT
            if not username or not username.strip():
                raise ValueError('Username is required')
            if not password or not password.strip():
                raise ValueError('Password is required')
            if not full_name or not full_name.strip():
                raise ValueError('Full name is required')
            if not role or role not in ['admin', 'customer', 'cooperator']:
                raise ValueError('Invalid role')
            
            # Check username unique
            cursor.execute('SELECT COUNT(*) FROM users WHERE username = ?', (username.strip(),))
            if cursor.fetchone()[0] > 0:
                raise ValueError('Username already exists')
            
            import hashlib
            import uuid
            
            user_id = str(uuid.uuid4())
            password_hash = hashlib.sha256(password.encode()).hexdigest()
            
            cursor.execute('''
                INSERT INTO users (id, username, password_hash, full_name, email, role, created_by)
                VALUES (?, ?, ?, ?, ?, ?, ?)
            ''', (user_id, username.strip(), password_hash, full_name.strip(), 
                email.strip() if email else '', role, created_by))
            
            conn.commit()
            print(f"✅ Created user {username} successfully")
            return user_id
            
        except Exception as e:
            conn.rollback()
            print(f"❌ Error creating user {username}: {e}")
            raise
        finally:
            conn.close()

    def update_user(self, user_id, **kwargs):
        """Cập nhật thông tin user với validation tốt hơn"""
        conn = self.get_connection()
        cursor = conn.cursor()
        
        try:
            set_clauses = []
            values = []
            
            for key, value in kwargs.items():
                if key == 'password' and value:
                    import hashlib
                    set_clauses.append('password_hash = ?')
                    values.append(hashlib.sha256(value.encode()).hexdigest())
                elif value is not None:  # ✅ Chỉ update nếu value không None
                    set_clauses.append(f'{key} = ?')
                    values.append(value)
            
            if set_clauses:  # ✅ Chỉ update nếu có thay đổi
                set_clauses.append('updated_at = GETDATE()')
                values.append(user_id)
                
                query = f"UPDATE users SET {', '.join(set_clauses)} WHERE id = ?"
                cursor.execute(query, values)
                
                if cursor.rowcount == 0:
                    raise ValueError('User not found')
            
            conn.commit()
            print(f"✅ Updated user {user_id} successfully")
            
        except Exception as e:
            conn.rollback()
            print(f"❌ Error updating user {user_id}: {e}")
            raise
        finally:
            conn.close()

    def delete_user(self, user_id):
        """Xóa user (soft delete)"""
        conn = self.get_connection()
        cursor = conn.cursor()
        
        cursor.execute('UPDATE users SET status = ?, updated_at = GETDATE() WHERE id = ?', 
                    ('deleted', user_id))
        
        conn.commit()
        conn.close()

    def get_user_relationships(self, customer_id=None):
        """Lấy quan hệ customer-cooperator"""
        conn = self.get_connection()
        cursor = conn.cursor()
        
        query = '''
            SELECT ur.id, c.id as customer_id, c.full_name as customer_name,
                co.id as cooperator_id, co.full_name as cooperator_name,
                ur.created_at
            FROM user_relationships ur
            JOIN users c ON ur.customer_id = c.id
            JOIN users co ON ur.cooperator_id = co.id
            WHERE 1=1
        '''
        params = []
        
        if customer_id:
            query += ' AND ur.customer_id = ?'
            params.append(customer_id)
        
        query += ' ORDER BY ur.created_at DESC'
        
        cursor.execute(query, params)
        
        relationships = []
        for row in cursor.fetchall():
            relationships.append({
                'id': row[0],
                'customer_id': row[1],
                'customer_name': row[2],
                'cooperator_id': row[3],
                'cooperator_name': row[4],
                'created_at': self._safe_datetime_convert(row[5])  # ✅ SAFE CONVERT
            })
        
        conn.close()
        return relationships

    def assign_cooperator_to_customer(self, customer_id, cooperator_id, created_by):
        """Gán cooperator cho customer"""
        conn = self.get_connection()
        cursor = conn.cursor()
        
        # Check if relationship exists
        cursor.execute('''
            SELECT COUNT(*) FROM user_relationships 
            WHERE customer_id = ? AND cooperator_id = ?
        ''', (customer_id, cooperator_id))
        
        if cursor.fetchone()[0] > 0:
            conn.close()
            raise ValueError('Relationship already exists')
        
        cursor.execute('''
            INSERT INTO user_relationships (customer_id, cooperator_id, created_by)
            VALUES (?, ?, ?)
        ''', (customer_id, cooperator_id, created_by))
        
        conn.commit()
        conn.close()

    def remove_cooperator_from_customer(self, customer_id, cooperator_id):
        """Xóa cooperator khỏi customer"""
        conn = self.get_connection()
        cursor = conn.cursor()
        
        cursor.execute('''
            DELETE FROM user_relationships 
            WHERE customer_id = ? AND cooperator_id = ?
        ''', (customer_id, cooperator_id))
        
        conn.commit()
        conn.close()

    def get_accessible_batches_paginated(self, user_id, user_role, page=1, page_size=10, search='', status_filter=''):
        """Lấy batches với pagination và filtering"""
        conn = self.get_connection()
        cursor = conn.cursor()
        
        try:
            # Build WHERE clause dựa trên role
            if user_role == 'admin':
                where_conditions = ['1=1']  # Admin xem tất cả
                params = []
            elif user_role == 'customer':
                where_conditions = ['b.created_by = ?']
                params = [user_id]
            elif user_role == 'cooperator':
                where_conditions = ['''
                    b.created_by IN (
                        SELECT cr.customer_id FROM user_relationships cr 
                        WHERE cr.cooperator_id = ?
                    )
                ''']
                params = [user_id]
            else:
                where_conditions = ['1=0']  # Không có quyền
                params = []
            
            # Add search condition
            if search:
                where_conditions.append('b.name LIKE ?')
                params.append(f'%{search}%')
            
            # Add status filter
            if status_filter:
                where_conditions.append('b.status = ?')
                params.append(status_filter)
            
            where_clause = ' AND '.join(where_conditions)
            
            # ✅ GET TOTAL COUNT
            count_query = f'''
                SELECT COUNT(*) 
                FROM batches b
                WHERE {where_clause}
            '''
            cursor.execute(count_query, params)
            total_count = cursor.fetchone()[0]
            
            # Calculate pagination
            total_pages = (total_count + page_size - 1) // page_size
            offset = (page - 1) * page_size
            
            # ✅ FIX: SELECT chỉ những columns thực sự tồn tại trong schema
            batches_query = f'''
                SELECT b.id, b.name, b.status, b.total_files, b.processed_files, 
                    b.created_at, b.created_by,
                    u.full_name as created_by_name
                FROM batches b
                LEFT JOIN users u ON b.created_by = u.id
                WHERE {where_clause}
                ORDER BY b.created_at DESC
                OFFSET ? ROWS FETCH NEXT ? ROWS ONLY
            '''
            
            cursor.execute(batches_query, params + [offset, page_size])
            batches = []
            
            for row in cursor.fetchall():
                batch = {
                    'id': row[0],
                    'name': row[1],
                    'status': row[2],
                    'total_files': row[3] or 0,
                    'processed_files': row[4] or 0,
                    'created_at': self._safe_datetime_convert(row[5]),
                    'created_by': row[6],
                    'created_by_name': row[7] or 'Unknown'
                    # ✅ BỎ processed_at vì column này không tồn tại
                }
                batches.append(batch)
            
            return {
                'batches': batches,
                'total_count': total_count,
                'total_pages': total_pages,
                'current_page': page,
                'page_size': page_size
            }
            
        except Exception as e:
            print(f"❌ Error getting paginated batches: {e}")
            raise
        finally:
            conn.close()

    def _add_missing_columns(self, cursor):
        """Thêm các cột mới nếu chưa có"""
        try:
            # Thêm cột created_by vào bảng batches nếu chưa có
            cursor.execute("""
                IF NOT EXISTS (SELECT * FROM INFORMATION_SCHEMA.COLUMNS 
                            WHERE TABLE_NAME = 'batches' AND COLUMN_NAME = 'created_by')
                ALTER TABLE batches ADD created_by NVARCHAR(50)
            """)
            
            # Thêm cột created_by vào bảng documents nếu chưa có
            cursor.execute("""
                IF NOT EXISTS (SELECT * FROM INFORMATION_SCHEMA.COLUMNS 
                            WHERE TABLE_NAME = 'documents' AND COLUMN_NAME = 'created_by')
                ALTER TABLE documents ADD created_by NVARCHAR(50)
            """)
            
            # Kiểm tra cột cut_point
            cursor.execute("""
                IF NOT EXISTS (SELECT * FROM INFORMATION_SCHEMA.COLUMNS 
                            WHERE TABLE_NAME = 'documents' AND COLUMN_NAME = 'cut_point')
                ALTER TABLE documents ADD cut_point INT DEFAULT 0
            """)
            
            # Kiểm tra các cột image paths
            cursor.execute("""
                IF NOT EXISTS (SELECT * FROM INFORMATION_SCHEMA.COLUMNS 
                            WHERE TABLE_NAME = 'documents' AND COLUMN_NAME = 'image_full_path')
                ALTER TABLE documents ADD image_full_path NVARCHAR(500)
            """)
            
            cursor.execute("""
                IF NOT EXISTS (SELECT * FROM INFORMATION_SCHEMA.COLUMNS 
                            WHERE TABLE_NAME = 'documents' AND COLUMN_NAME = 'image_header_path')
                ALTER TABLE documents ADD image_header_path NVARCHAR(500)
            """)
            
            cursor.execute("""
                IF NOT EXISTS (SELECT * FROM INFORMATION_SCHEMA.COLUMNS 
                            WHERE TABLE_NAME = 'documents' AND COLUMN_NAME = 'image_content_path')
                ALTER TABLE documents ADD image_content_path NVARCHAR(500)
            """)
            
            cursor.execute("""
                IF NOT EXISTS (SELECT * FROM INFORMATION_SCHEMA.COLUMNS 
                            WHERE TABLE_NAME = 'documents' AND COLUMN_NAME = 'header_detection')
                ALTER TABLE documents ADD header_detection NTEXT
            """)
            
            cursor.execute("""
                IF NOT EXISTS (SELECT * FROM INFORMATION_SCHEMA.COLUMNS 
                            WHERE TABLE_NAME = 'documents' AND COLUMN_NAME = 'total_pages')
                ALTER TABLE documents ADD total_pages INT DEFAULT 1
            """)
            cursor.execute("""
                IF NOT EXISTS (SELECT * FROM INFORMATION_SCHEMA.COLUMNS 
                            WHERE TABLE_NAME = 'documents' AND COLUMN_NAME = 'kie_header_data')
                ALTER TABLE documents ADD kie_header_data NTEXT
            """)
            
            # Thêm cột kie_content_data để lưu KIE từ content (tóm tắt)
            cursor.execute("""
                IF NOT EXISTS (SELECT * FROM INFORMATION_SCHEMA.COLUMNS 
                            WHERE TABLE_NAME = 'documents' AND COLUMN_NAME = 'kie_content_data')
                ALTER TABLE documents ADD kie_content_data NTEXT
            """)
            cursor.execute("""
                IF NOT EXISTS (SELECT * FROM INFORMATION_SCHEMA.COLUMNS 
                            WHERE TABLE_NAME = 'documents' AND COLUMN_NAME = 'vanBanId')
                ALTER TABLE documents ADD vanBanId NVARCHAR(100)
            """)
            cursor.execute("""
                IF NOT EXISTS (SELECT * FROM INFORMATION_SCHEMA.COLUMNS 
                            WHERE TABLE_NAME = 'batches' AND COLUMN_NAME = 'batch_type')
                ALTER TABLE batches ADD batch_type NVARCHAR(20) DEFAULT 'local'
            """)
            cursor.execute("""
                IF NOT EXISTS (
                    SELECT * FROM sys.columns 
                    WHERE object_id = OBJECT_ID('documents') 
                    AND name = 'document_original_type'
                )
                BEGIN
                    ALTER TABLE documents 
                    ADD document_original_type NVARCHAR(50) DEFAULT N'Bản chính';
                    PRINT '✅ Added column: document_original_type';
                END
            """)
            
            # Check và thêm document_note
            cursor.execute("""
                IF NOT EXISTS (
                    SELECT * FROM sys.columns 
                    WHERE object_id = OBJECT_ID('documents') 
                    AND name = 'document_note'
                )
                BEGIN
                    ALTER TABLE documents 
                    ADD document_note NVARCHAR(MAX) NULL;
                    PRINT '✅ Added column: document_note';
                END
            """)
            
            # ✅ Check và thêm is_fragmented vào work_packages
            cursor.execute("""
                IF NOT EXISTS (
                    SELECT * FROM sys.columns 
                    WHERE object_id = OBJECT_ID('work_packages') 
                    AND name = 'is_fragmented'
                )
                BEGIN
                    ALTER TABLE work_packages 
                    ADD is_fragmented BIT DEFAULT 1;
                    PRINT '✅ Added column: is_fragmented to work_packages';
                END
            """)
            
            # ✅ Cập nhật CHECK constraint cho work_items.section_type để cho phép 'full'
            cursor.execute("""
                -- Tìm và drop constraint cũ
                DECLARE @ConstraintName nvarchar(200)
                SELECT @ConstraintName = Name 
                FROM sys.check_constraints 
                WHERE parent_object_id = OBJECT_ID('work_items') 
                AND parent_column_id = (
                    SELECT column_id 
                    FROM sys.columns 
                    WHERE object_id = OBJECT_ID('work_items') 
                    AND name = 'section_type'
                )
                
                IF @ConstraintName IS NOT NULL
                BEGIN
                    DECLARE @SQL nvarchar(500)
                    SET @SQL = 'ALTER TABLE work_items DROP CONSTRAINT ' + @ConstraintName
                    EXEC sp_executesql @SQL
                    PRINT '✅ Dropped old CHECK constraint: ' + @ConstraintName
                END
                
                -- Tạo constraint mới với 3 giá trị: header, content, full
                IF NOT EXISTS (
                    SELECT * FROM sys.check_constraints 
                    WHERE name = 'CK_work_items_section_type_full' 
                    AND parent_object_id = OBJECT_ID('work_items')
                )
                BEGIN
                    ALTER TABLE work_items 
                    ADD CONSTRAINT CK_work_items_section_type_full 
                    CHECK (section_type IN ('header', 'content', 'full'))
                    PRINT '✅ Added new CHECK constraint allowing section_type: header, content, full'
                END
            """)
            
            print("✅ All missing columns added successfully")

            # ✅ Thêm cột completed_by vào work_items để ghi nhận ai xác nhận
            cursor.execute("""
                IF NOT EXISTS (
                    SELECT * FROM sys.columns 
                    WHERE object_id = OBJECT_ID('work_items') 
                    AND name = 'completed_by'
                )
                BEGIN
                    ALTER TABLE work_items 
                    ADD completed_by NVARCHAR(50) NULL;
                    PRINT '✅ Added column: completed_by to work_items';
                END
            """)

            # ✅ Thêm cột locked_by, locked_at vào work_items để lock mảnh cho CTV
            cursor.execute("""
                IF NOT EXISTS (
                    SELECT * FROM sys.columns 
                    WHERE object_id = OBJECT_ID('work_items') 
                    AND name = 'locked_by'
                )
                BEGIN
                    ALTER TABLE work_items 
                    ADD locked_by NVARCHAR(50) NULL,
                        locked_at DATETIME NULL;
                    PRINT '✅ Added columns: locked_by, locked_at to work_items';
                END
            """)
            
        except Exception as e:
            print(f"Info: Some columns may already exist: {e}")


    def _insert_default_fields(self, cursor):
        """Insert default field definitions nếu chưa có"""
        cursor.execute('SELECT COUNT(*) FROM field_definitions')
        count = cursor.fetchone()[0]
        
        if count == 0:
            default_fields = [
                ('document_type', 'Loại tài liệu', 'select', '', 1, 1),
                ('document_number', 'Số tài liệu', 'text', '', 1, 2),
                ('document_symbol', 'Ký hiệu', 'text', '', 0, 3),
                ('handwritten_number', 'Số viết tay', 'text', '', 0, 4),  # ✅ NEW FIELD
                ('issue_date', 'Ngày ban hành', 'date', '', 1, 4),
                ('issuing_authority', 'Cơ quan ban hành', 'text', '', 1, 5),
                ('summary', 'Trích yếu', 'textarea', '', 1, 6)
            ]
            
            for field in default_fields:
                cursor.execute('''
                    INSERT INTO field_definitions 
                    (field_key, field_name, field_type, default_value, required, order_index) 
                    VALUES (?, ?, ?, ?, ?, ?)
                ''', field)

    # Batch operations
    def get_batches(self):
        """Lấy danh sách batch"""
        conn = self.get_connection()
        cursor = conn.cursor()
        cursor.execute('''
            SELECT id, name, created_at, total_files, processed_files, status 
            FROM batches ORDER BY created_at DESC
        ''')
        
        batches = []
        for row in cursor.fetchall():
            batches.append({
                'id': row[0],
                'name': row[1],
                'created_at': row[2].isoformat() if row[2] else None,
                'total_files': row[3],
                'processed_files': row[4],
                'status': row[5]
            })
        
        conn.close()
        return batches

    def create_batch(self, batch_id, batch_name, status='uploading', created_by=None, batch_type='local'):
        """Tạo batch mới với thông tin user"""
        conn = self.get_connection()
        cursor = conn.cursor()
        
        cursor.execute('''
            INSERT INTO batches (id, name, status, created_by, batch_type)
            VALUES (?, ?, ?, ?, ?)
        ''', (batch_id, batch_name, status, created_by, batch_type))
    
        
        conn.commit()
        conn.close()
        print(f"✅ Created {batch_type} batch: {batch_name} (ID: {batch_id})")
    def reset_processing_documents_to_uploaded(self, batch_id):
        """Reset documents đang processing về uploaded"""
        conn = self.get_connection()  # ✅ TẠO CONNECTION MỚI
        cursor = conn.cursor()        # ✅ TẠO CURSOR MỚI
        
        try:
            query = """
            UPDATE documents 
            SET status = 'uploaded', 
                processed_at = NULL,
                kie_header_data = NULL,
                kie_content_data = NULL
            WHERE batch_id = ? AND status = 'processing'
            """
            
            cursor.execute(query, (batch_id,))
            conn.commit()  # ✅ SỬ DỤNG conn.commit()
            
            affected_rows = cursor.rowcount
            print(f"✅ Reset {affected_rows} processing documents to uploaded for batch {batch_id}")
            
            return affected_rows
            
        except Exception as e:
            print(f"❌ Error resetting processing documents: {e}")
            conn.rollback()  # ✅ ROLLBACK NẾU CÓ LỖI
            return 0
        finally:
            conn.close()  # ✅ ĐÓNG CONNECTION
    def update_batch(self, batch_id, **kwargs):
        """Cập nhật thông tin batch"""
        conn = self.get_connection()
        cursor = conn.cursor()
        
        set_clauses = []
        values = []
        for key, value in kwargs.items():
            set_clauses.append(f"{key} = ?")
            values.append(value)
        
        values.append(batch_id)
        query = f"UPDATE batches SET {', '.join(set_clauses)} WHERE id = ?"
        
        cursor.execute(query, values)
        conn.commit()
        conn.close()

    # Thêm vào class DatabaseManager trong database.py

    def delete_work_items_by_doc_ids(self, doc_ids):
        """Xóa tất cả work_items liên quan đến các doc_ids"""
        if not doc_ids:
            return
        
        conn = self.get_connection()
        cursor = conn.cursor()
        
        try:
            # Tạo placeholders cho IN clause
            placeholders = ','.join(['?' for _ in doc_ids])
            
            # Xóa work_items liên quan
            delete_query = f'''
                DELETE FROM work_items 
                WHERE doc_id IN ({placeholders})
            '''
            
            cursor.execute(delete_query, doc_ids)
            deleted_count = cursor.rowcount
            
            conn.commit()
            print(f"✅ Đã xóa {deleted_count} work_items liên quan đến documents")
            
        except Exception as e:
            print(f"❌ Lỗi khi xóa work_items: {e}")
            conn.rollback()
            raise
        finally:
            conn.close()

    def delete_batch(self, batch_id):
        """Xóa batch và tất cả documents liên quan (bao gồm work_items)"""
        conn = self.get_connection()
        cursor = conn.cursor()
        
        try:
            # ✅ BƯỚC 1: Lấy thông tin documents trước khi xóa
            cursor.execute('''
                SELECT id, filename, file_path, image_full_path, image_header_path, image_content_path
                FROM documents 
                WHERE batch_id = ?
            ''', (batch_id,))
            
            documents = cursor.fetchall()
            document_data = []
            doc_ids = []
            
            for doc in documents:
                doc_ids.append(doc[0])  # doc_id
                # ✅ FIX: Return as dict format thay vì tuple
                document_data.append({
                    'id': doc[0],
                    'filename': doc[1],
                    'file_path': doc[2],
                    'image_full_path': doc[3],
                    'image_header_path': doc[4],
                    'image_content_path': doc[5]
                })
            
            print(f"📋 Tìm thấy {len(documents)} documents trong batch {batch_id}")
            
            # ✅ BƯỚC 2: Xóa work_items liên quan đến documents trong batch này
            if doc_ids:
                print(f"🗑️ Đang xóa work_items cho {len(doc_ids)} documents...")
                
                placeholders = ','.join(['?' for _ in doc_ids])
                delete_work_items_query = f'''
                    DELETE FROM work_items 
                    WHERE doc_id IN ({placeholders})
                '''
                
                cursor.execute(delete_work_items_query, doc_ids)
                work_items_deleted = cursor.rowcount
                print(f"✅ Đã xóa {work_items_deleted} work_items")
            
            # ✅ BƯỚC 3: Xóa documents
            cursor.execute('DELETE FROM documents WHERE batch_id = ?', (batch_id,))
            docs_deleted = cursor.rowcount
            print(f"✅ Đã xóa {docs_deleted} documents")
            
            # ✅ BƯỚC 4: Xóa batch
            cursor.execute('DELETE FROM batches WHERE id = ?', (batch_id,))
            batch_deleted = cursor.rowcount
            
            if batch_deleted == 0:
                raise ValueError('Batch not found')
            
            conn.commit()
            print(f"✅ Đã xóa batch {batch_id} thành công")
            
            return document_data
            
        except Exception as e:
            print(f"❌ Lỗi khi xóa batch {batch_id}: {e}")
            conn.rollback()
            raise
        finally:
            conn.close()

    def get_batch_status(self, batch_id):
        """Lấy trạng thái batch"""
        conn = self.get_connection()
        cursor = conn.cursor()
        cursor.execute('''
            SELECT total_files, processed_files, status FROM batches WHERE id = ?
        ''', (batch_id,))
        
        row = cursor.fetchone()
        conn.close()
        
        if not row:
            return None
            
        return {
            'total_files': row[0],
            'processed_files': row[1],
            'status': row[2],
            'progress_percent': (row[1] / row[0] * 100) if row[0] > 0 else 0
        }

    # Document operations
    def get_batch_documents(self, batch_id):
        """Lấy danh sách documents trong batch"""
        conn = self.get_connection()
        cursor = conn.cursor()
        cursor.execute('''
            SELECT id, filename, document_type, status, created_at, processed_at
            FROM documents WHERE batch_id = ? ORDER BY filename
        ''', (batch_id,))
        
        documents = []
        for row in cursor.fetchall():
            documents.append({
                'id': row[0],
                'filename': row[1],
                'document_type': row[2],
                'status': row[3],
                'created_at': row[4].isoformat() if row[4] else None,
                'processed_at': row[5].isoformat() if row[5] else None
            })
        
        conn.close()
        return documents
    def get_batch_info(self, batch_id):
        """Lấy thông tin đầy đủ của batch bao gồm batch_type"""
        conn = self.get_connection()
        cursor = conn.cursor()
        
        cursor.execute('''
            SELECT b.id, b.name, b.created_at, b.total_files, b.processed_files, 
                b.status, u.full_name as created_by_name, u.id as created_by_id, b.batch_type
            FROM batches b
            LEFT JOIN users u ON b.created_by = u.id
            WHERE b.id = ?
        ''', (batch_id,))
        
        row = cursor.fetchone()
        conn.close()
        
        if not row:
            return None
            
        return {
            'id': row[0],
            'name': row[1],
            'created_at': self._safe_datetime_convert(row[2]),
            'total_files': row[3],
            'processed_files': row[4],
            'status': row[5],
            'created_by_name': row[6],
            'created_by_id': row[7],
            'batch_type': row[8] or 'local'  # ✅ THÊM batch_type
        }
    def create_document(self, doc_id, batch_id, filename, file_path, created_by=None, van_ban_id=None):
        """Tạo document mới với thông tin user"""
        conn = self.get_connection()
        cursor = conn.cursor()
        
        cursor.execute('''
            INSERT INTO documents (id, batch_id, filename, file_path, status, created_by, vanBanId)
            VALUES (?, ?, ?, ?, ?, ?, ?)
        ''', (doc_id, batch_id, filename, file_path, 'pending', created_by, van_ban_id))
        
        
        conn.commit()
        conn.close()
        print(f"📄 Created document: {filename} (ID: {doc_id})")


    def get_document(self, doc_id):
        """Lấy thông tin chi tiết document với KIE data tách riêng"""
        conn = self.get_connection()
        cursor = conn.cursor()
        cursor.execute('''
            SELECT filename, file_path, kie_header_data, kie_content_data, cut_point, 
                image_full_path, image_header_path, image_content_path,
                header_detection, status, total_pages, vanBanId,
                document_original_type, document_note
            FROM documents WHERE id = ?
        ''', (doc_id,))
        
        row = cursor.fetchone()
        conn.close()
        
        if not row:
            return None
        
        # Parse JSON fields safely
        try:
            kie_header_data = json.loads(row[2]) if row[2] else {}
        except:
            kie_header_data = {}
            
        try:
            kie_content_data = json.loads(row[3]) if row[3] else {}
        except:
            kie_content_data = {}
            
        try:
            header_detection = json.loads(row[8]) if row[8] else {}
        except:
            header_detection = {}
        
        return {
            'id': doc_id,
            'filename': row[0],
            'file_path': row[1],
            'kie_header_data': kie_header_data,
            'kie_content_data': kie_content_data,
            'cut_point': row[4] or 0,
            'image_paths': {
                'full': row[5],
                'header': row[6],
                'content': row[7]
            },
            'header_detection': header_detection,
            'status': row[9],
            'total_pages': row[10] or 1,
            'vanBanId': row[11],
            'document_original_type': row[12] or 'Bản chính',  # ✅ MỚI
            'document_note': row[13] or ''  # ✅ MỚI
        }
    def update_document(self, doc_id, **kwargs):
        """Cập nhật thông tin document với KIE data tách riêng"""
        conn = self.get_connection()
        cursor = conn.cursor()
        
        set_clauses = []
        values = []
        for key, value in kwargs.items():
            if key == 'kie_header_data' or key == 'kie_content_data' or key == 'header_detection':
                set_clauses.append(f"{key} = ?")
                values.append(json.dumps(value, ensure_ascii=False) if value else None)
            elif key == 'processed_at':
                set_clauses.append(f"{key} = ?")
                values.append(value)
            # ✅ THÊM 2 FIELDS MỚI
            elif key == 'document_original_type':
                set_clauses.append(f"{key} = ?")
                values.append(value or 'Bản chính')
            elif key == 'document_note':
                set_clauses.append(f"{key} = ?")
                values.append(value or '')
            else:
                set_clauses.append(f"{key} = ?")
                values.append(value)
        
        values.append(doc_id)
        query = f"UPDATE documents SET {', '.join(set_clauses)} WHERE id = ?"
        
        # ✅ DEBUG LOG
        print(f"🔍 UPDATE DOCUMENT DEBUG:")
        print(f"   Query: {query}")
        print(f"   Values: {values}")
        print(f"   Keys: {list(kwargs.keys())}")
        
        try:
            cursor.execute(query, values)
            conn.commit()
        except Exception as e:
            print(f"❌ Error updating document: {e}")
            print(f"   Failed query: {query}")
            print(f"   Failed values: {values}")
            conn.rollback()
            raise
        finally:
            conn.close()


    def get_document_status(self, doc_id):
        """Lấy trạng thái document"""
        conn = self.get_connection()
        cursor = conn.cursor()
        cursor.execute('SELECT status, filename, processed_at FROM documents WHERE id = ?', (doc_id,))
        
        result = cursor.fetchone()
        conn.close()
        
        if not result:
            return None
            
        return {
            'doc_id': doc_id,
            'filename': result[0],
            'status': result[1],
            'processed_at': result[2].isoformat() if result[2] else None
        }

    def get_documents_for_processing(self, batch_id):
        """Lấy danh sách documents cần xử lý"""
        conn = self.get_connection()
        cursor = conn.cursor()
        cursor.execute('SELECT id, file_path, filename FROM documents WHERE batch_id = ? AND status != ?', 
                      (batch_id, 'completed'))
        documents = cursor.fetchall()
        conn.close()
        return documents

    # Field definitions
    def get_field_definitions(self):
        """Lấy định nghĩa các trường"""
        conn = self.get_connection()
        cursor = conn.cursor()
        cursor.execute('''
            SELECT field_key, field_name, field_type, default_value, required, order_index
            FROM field_definitions ORDER BY order_index
        ''')
        
        fields = []
        for row in cursor.fetchall():
            fields.append({
                'key': row[0],
                'name': row[1],
                'type': row[2],
                'default_value': row[3],
                'required': bool(row[4]),
                'order': row[5]
            })
        
        conn.close()
        return fields
    
    def _has_valid_kie_data(self, kie_data_json, data_type='header'):
        if not kie_data_json:
            return False
        
        try:
            if isinstance(kie_data_json, str):
                data = json.loads(kie_data_json)
            elif isinstance(kie_data_json, dict):
                data = kie_data_json
            else:
                return False
            
            # ✅ FLAT FORMAT VALIDATION
            if data_type == 'header':
                # Kiểm tra ít nhất có document_type
                return bool(data.get('document_type', '').strip())
            else:
                # Content: kiểm tra có summary
                return bool(data.get('summary', '').strip())
                
        except Exception as e:
            print(f"❌ Error validating KIE data: {e}")
            return False

    def get_available_pieces_count(self, customer_id=None):
        """Đếm số mảnh khả dụng (có KIE hợp lệ + chưa được gán) theo customer"""
        conn = self.get_connection()
        cursor = conn.cursor()
        
        try:
            # Lấy documents của customer
            cursor.execute('''
                SELECT 
                    d.id,
                    d.image_header_path,
                    d.image_content_path,
                    d.kie_header_data,
                    d.kie_content_data
                FROM documents d
                JOIN batches b ON d.batch_id = b.id
                WHERE b.created_by = ?
            ''', (customer_id,))
            
            documents = cursor.fetchall()
            
            # ✅ THÊM: Lấy danh sách mảnh đã được gán
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
            
            available_count = 0
            
            for doc in documents:
                doc_id, header_path, content_path, kie_header_data, kie_content_data = doc
                
                # ✅ FIX: Bỏ qua document đã được gán 'full'
                if doc_id in assigned_full_docs:
                    continue
                
                # ✅ Kiểm tra header piece: có KIE hợp lệ + chưa được gán
                if header_path and kie_header_data:
                    piece_key = (doc_id, 'header')
                    if (piece_key not in assigned_pieces and 
                        self._has_valid_kie_data(kie_header_data, 'header')):
                        available_count += 1
                
                # ✅ Kiểm tra content piece: có KIE hợp lệ + chưa được gán  
                if content_path and kie_content_data:
                    piece_key = (doc_id, 'content')
                    if (piece_key not in assigned_pieces and 
                        self._has_valid_kie_data(kie_content_data, 'content')):
                        available_count += 1
            
            return available_count
            
        except Exception as e:
            print(f"❌ Lỗi đếm available pieces: {e}")
            return 0
        finally:
            conn.close()

    def get_available_pieces_detailed(self, customer_id=None):
        """Lấy danh sách chi tiết các mảnh khả dụng với thông tin KIE"""
        conn = self.get_connection()
        cursor = conn.cursor()
        
        try:
            base_query = '''
                SELECT 
                    d.id,
                    d.filename,
                    d.image_header_path,
                    d.image_content_path,
                    d.kie_header_data,
                    d.kie_content_data,
                    d.document_type,
                    b.name as batch_name
                FROM documents d
                JOIN batches b ON d.batch_id = b.id
            '''
            
            params = []
            if customer_id:
                # ✅ SỬA: Dùng created_by thay vì customer_id
                base_query += ' WHERE b.created_by = ?'
                params.append(customer_id)
            
            cursor.execute(base_query, params)
            documents = cursor.fetchall()
            
            available_pieces = []
            
            for doc in documents:
                doc_id, filename, header_path, content_path, kie_header_data, kie_content_data, doc_type, batch_name = doc
                
                # Kiểm tra header piece
                if header_path and kie_header_data:
                    if self._has_valid_kie_data(kie_header_data, 'header'):
                        available_pieces.append({
                            'doc_id': doc_id,
                            'filename': filename,
                            'piece_type': 'header',
                            'image_path': header_path,
                            'kie_data': kie_header_data,
                            'document_type': doc_type,
                            'batch_name': batch_name,
                            'piece_description': f'Header - {filename}'
                        })
                
                # Kiểm tra content piece
                if content_path and kie_content_data:
                    if self._has_valid_kie_data(kie_content_data, 'content'):
                        available_pieces.append({
                            'doc_id': doc_id,
                            'filename': filename,
                            'piece_type': 'content',
                            'image_path': content_path,
                            'kie_data': kie_content_data,
                            'document_type': doc_type,
                            'batch_name': batch_name,
                            'piece_description': f'Content - {filename}'
                        })
            
            return available_pieces
            
        except Exception as e:
            print(f"❌ Lỗi lấy available pieces detailed: {e}")
            return []
        finally:
            conn.close()

    def create_work_package(self, package_id, name, customer_id, cooperator_ids, total_items, is_fragmented=True):
        """Tạo work package mới với nhiều cooperators"""
        conn = self.get_connection()
        cursor = conn.cursor()
        
        # Hỗ trợ cả string (1 CTV) và list (nhiều CTV)
        if isinstance(cooperator_ids, str):
            cooperator_ids = [cooperator_ids]
        
        # Lưu cooperator đầu tiên vào cột cooperator_id (backward compatible)
        primary_cooperator = cooperator_ids[0]
        
        cursor.execute('''
            INSERT INTO work_packages (id, name, customer_id, cooperator_id, total_items, is_fragmented)
            VALUES (?, ?, ?, ?, ?, ?)
        ''', (package_id, name, customer_id, primary_cooperator, total_items, is_fragmented))
        
        # Lưu tất cả cooperators vào bảng junction
        for coop_id in cooperator_ids:
            import uuid as _uuid
            row_id = str(_uuid.uuid4())
            cursor.execute('''
                INSERT INTO work_package_cooperators (id, package_id, cooperator_id)
                VALUES (?, ?, ?)
            ''', (row_id, package_id, coop_id))
        
        conn.commit()
        conn.close()
        fragmented_mode = "phân mảnh" if is_fragmented else "không phân mảnh"
        print(f"✅ Created work package: {name} (ID: {package_id}) - {len(cooperator_ids)} CTV - Chế độ: {fragmented_mode}")

    def create_work_items(self, items_data):
        """Tạo nhiều work items cùng lúc"""
        conn = self.get_connection()
        cursor = conn.cursor()
        
        for item in items_data:
            cursor.execute('''
                INSERT INTO work_items (id, package_id, doc_id, section_type)
                VALUES (?, ?, ?, ?)
            ''', (item['id'], item['package_id'], item['doc_id'], item['section_type']))
        
        conn.commit()
        conn.close()
        print(f"✅ Created {len(items_data)} work items")

    def get_packages_paginated(self, customer_id=None, cooperator_id=None, search_term='', page=1, page_size=10, status_filter=''):
        """Lấy danh sách packages với phân trang, search và ĐÚNG status filter"""
        conn = self.get_connection()
        cursor = conn.cursor()
        
        offset = (page - 1) * page_size
        
        base_query = '''
            FROM work_packages wp
            JOIN users c ON wp.customer_id = c.id
            JOIN users co ON wp.cooperator_id = co.id
            WHERE 1=1
        '''
        
        params = []
        
        if customer_id:
            base_query += ' AND wp.customer_id = ?'
            params.append(customer_id)
            
        if cooperator_id:
            base_query += ' AND wp.id IN (SELECT package_id FROM work_package_cooperators WHERE cooperator_id = ?)'
            params.append(cooperator_id)
            
        if search_term:
            base_query += ' AND wp.name LIKE ?'
            params.append(f'%{search_term}%')
        
        # ✅ SỬA STATUS FILTER - HANDLE MULTIPLE STATUSES
        if status_filter:
            if ',' in status_filter:
                # Multiple statuses: "in_progress,has_rejected"
                status_list = status_filter.split(',')
                placeholders = ','.join(['?' for _ in status_list])
                base_query += f' AND wp.status IN ({placeholders})'
                params.extend(status_list)
                
                print(f"🔍 Multi-status filter: {status_list}")
            else:
                # Single status
                base_query += ' AND wp.status = ?'
                params.append(status_filter)
                
                print(f"🔍 Single status filter: {status_filter}")
        
        # Count total với filter
        count_query = f'SELECT COUNT(*) {base_query}'
        cursor.execute(count_query, params)
        total_count = cursor.fetchone()[0]
        
        # Get paginated results with status counts
        data_query = f'''
            SELECT wp.id, wp.name, wp.customer_id, wp.cooperator_id, wp.total_items, 
                wp.status, wp.is_fragmented, wp.created_at, c.full_name as customer_name, co.full_name as cooperator_name,
                (SELECT COUNT(*) FROM work_items wi WHERE wi.package_id = wp.id AND wi.status = 'pending') as pending_count,
                (SELECT COUNT(*) FROM work_items wi WHERE wi.package_id = wp.id AND wi.status = 'completed') as completed_count,
                (SELECT COUNT(*) FROM work_items wi WHERE wi.package_id = wp.id AND wi.status = 'recompleted') as recompleted_count,
                (SELECT COUNT(*) FROM work_items wi WHERE wi.package_id = wp.id AND wi.status = 'rejected') as rejected_count,
                (SELECT COUNT(*) FROM work_items wi WHERE wi.package_id = wp.id AND wi.status = 'approved') as approved_count
            {base_query}
            ORDER BY wp.created_at DESC
            OFFSET ? ROWS FETCH NEXT ? ROWS ONLY
        '''
        
        params.extend([offset, page_size])
        cursor.execute(data_query, params)
        
        packages = []
        for row in cursor.fetchall():
            pkg_id = row[0]
            
            # Lấy tất cả cooperators của package từ junction table
            cursor.execute('''
                SELECT u.id, u.full_name 
                FROM work_package_cooperators wpc
                JOIN users u ON wpc.cooperator_id = u.id
                WHERE wpc.package_id = ?
                ORDER BY u.full_name
            ''', (pkg_id,))
            coop_rows = cursor.fetchall()
            
            if coop_rows:
                cooperator_names = [r[1] for r in coop_rows]
                cooperator_ids_list = [r[0] for r in coop_rows]
            else:
                # Fallback: dùng cooperator_id cũ
                cooperator_names = [row[9]] if row[9] else []
                cooperator_ids_list = [row[3]] if row[3] else []
            
            packages.append({
                'id': pkg_id,
                'name': row[1],
                'customer_id': row[2],
                'cooperator_id': row[3],
                'cooperator_ids': cooperator_ids_list,
                'total_items': row[4],
                'status': row[5],
                'is_fragmented': bool(row[6]),
                'created_at': self._safe_datetime_convert(row[7]),
                'customer_name': row[8],
                'cooperator_name': ', '.join(cooperator_names),
                'cooperator_names': cooperator_names,
                'status_counts': {
                    'pending': row[10],
                    'completed': row[11],
                    'recompleted': row[12],
                    'rejected': row[13],
                    'approved': row[14]
                }
            })
        
        conn.close()
        
        print(f"📊 Filter result: {len(packages)} packages with status_filter='{status_filter}'")
        
        return {
            'packages': packages,
            'total_count': total_count,
            'total_pages': (total_count + page_size - 1) // page_size,
            'current_page': page,
            'page_size': page_size,
            'status_filter': status_filter
        }

    def get_work_items_paginated(self, package_id=None, cooperator_id=None, status_filter=None, search_term='', page=1, page_size=10):
        """Lấy danh sách work items với phân trang"""
        conn = self.get_connection()
        cursor = conn.cursor()
        
        offset = (page - 1) * page_size
        
        # Base query với JOIN để lấy thông tin document và package
        base_query = '''
            FROM work_items wi
            JOIN documents d ON wi.doc_id = d.id
            JOIN work_packages wp ON wi.package_id = wp.id
            WHERE 1=1
        '''
        
        params = []
        
        if package_id:
            base_query += ' AND wi.package_id = ?'
            params.append(package_id)
            
        if cooperator_id:
            base_query += ' AND wp.id IN (SELECT package_id FROM work_package_cooperators WHERE cooperator_id = ?)'
            params.append(cooperator_id)
            
        if status_filter:
            base_query += ' AND wi.status = ?'
            params.append(status_filter)
            
        if search_term:
            base_query += ' AND (d.filename LIKE ? OR wp.name LIKE ?)'
            params.extend([f'%{search_term}%', f'%{search_term}%'])
        
        # Count total
        count_query = f'SELECT COUNT(*) {base_query}'
        cursor.execute(count_query, params)
        total_count = cursor.fetchone()[0]
        
        # Get paginated results
        data_query = f'''
            SELECT wi.id, wi.package_id, wi.doc_id, wi.section_type, wi.status,
                wi.review_comment, wi.reviewed_by, wi.reviewed_at, wi.completed_at,
                d.filename, wp.name as package_name,
                CASE WHEN wi.section_type = 'header' THEN d.image_header_path 
                     ELSE d.image_content_path END as image_path
            {base_query}
            ORDER BY wi.created_at DESC
            OFFSET ? ROWS FETCH NEXT ? ROWS ONLY
        '''
        
        params.extend([offset, page_size])
        cursor.execute(data_query, params)
        
        items = []
        for row in cursor.fetchall():
            items.append({
                'id': row[0],
                'package_id': row[1],
                'doc_id': row[2],
                'section_type': row[3],
                'status': row[4],
                'review_comment': row[5],
                'reviewed_by': row[6],
                'reviewed_at': self._safe_datetime_convert(row[7]),     # ✅ SAFE CONVERT
                'completed_at': self._safe_datetime_convert(row[8]),    # ✅ SAFE CONVERT
                'filename': row[9],
                'package_name': row[10],
                'image_path': row[11]
            })
        
        conn.close()
        
        return {
            'items': items,
            'total_count': total_count,
            'total_pages': (total_count + page_size - 1) // page_size,
            'current_page': page,
            'page_size': page_size
        }

    def update_work_item_status(self, item_id, status, reviewed_by=None, review_comment=None, completed_by=None):
        """Cập nhật status của work item"""
        conn = self.get_connection()
        cursor = conn.cursor()
        
        if reviewed_by:
            # ✅ UPDATE VỚI REVIEW INFO
            cursor.execute('''
                UPDATE work_items 
                SET status = ?, reviewed_by = ?, review_comment = ?, reviewed_at = GETDATE(),
                    locked_by = NULL, locked_at = NULL
                WHERE id = ?
            ''', (status, reviewed_by, review_comment, item_id))
        else:
            # ✅ UPDATE STATUS THÔNG THƯỜNG + completed_by + unlock
            cursor.execute('''
                UPDATE work_items 
                SET status = ?, 
                    completed_at = CASE WHEN ? IN ('completed', 'recompleted') THEN GETDATE() ELSE completed_at END,
                    completed_by = CASE WHEN ? IN ('completed', 'recompleted') THEN ? ELSE completed_by END,
                    locked_by = NULL, locked_at = NULL
                WHERE id = ?
            ''', (status, status, status, completed_by, item_id))
        
        conn.commit()
        conn.close()

    def delete_work_package(self, package_id, user_id):
        """Xóa work package và các work items liên quan"""
        conn = self.get_connection()
        cursor = conn.cursor()
        
        # Kiểm tra quyền: chỉ customer tạo gói mới được xóa
        cursor.execute('SELECT customer_id FROM work_packages WHERE id = ?', (package_id,))
        result = cursor.fetchone()
        
        if not result:
            conn.close()
            raise ValueError('Package not found')
            
        if result[0] != user_id:
            conn.close()
            raise PermissionError('You can only delete your own packages')
        
        # Xóa work_items trước (do foreign key constraint)
        cursor.execute('DELETE FROM work_items WHERE package_id = ?', (package_id,))
        
        # Xóa work_package
        cursor.execute('DELETE FROM work_packages WHERE id = ?', (package_id,))
        
        conn.commit()
        conn.close()
        print(f"✅ Deleted work package: {package_id}")

    def get_customer_batches_with_available_count(self, customer_id):
        """Lấy danh sách batch của customer kèm số mảnh khả dụng"""
        conn = self.get_connection()
        cursor = conn.cursor()
        
        try:
            # Lấy batches của customer
            cursor.execute('''
                SELECT id, name, created_at, total_files, processed_files, status
                FROM batches 
                WHERE created_by = ?
                ORDER BY created_at DESC
            ''', (customer_id,))
            
            batches = cursor.fetchall()
            result = []
            
            for batch in batches:
                batch_id, name, created_at, total_files, processed_files, status = batch
                
                # Đếm mảnh khả dụng cho batch này
                available_count = self.get_available_pieces_count_by_batches(
                    customer_id, [batch_id]
                )
                
                result.append({
                    'id': batch_id,
                    'name': name,
                    'created_at': created_at.isoformat() if created_at else None,
                    'total_files': total_files,
                    'processed_files': processed_files,
                    'status': status,
                    'available_pieces': available_count
                })
            
            return result
            
        except Exception as e:
            print(f"❌ Lỗi lấy customer batches: {e}")
            return []
        finally:
            conn.close()

    def get_available_pieces_count_by_batches(self, customer_id, batch_ids=None):
        """Đếm số mảnh khả dụng theo batch IDs cụ thể"""
        conn = self.get_connection()
        cursor = conn.cursor()
        
        try:
            # Base query
            if batch_ids and len(batch_ids) > 0:
                # Filter theo batch IDs
                placeholders = ','.join(['?' for _ in batch_ids])
                batch_filter = f'AND b.id IN ({placeholders})'
                params = [customer_id] + batch_ids
            else:
                # Tất cả batch
                batch_filter = ''
                params = [customer_id]
            
            # Lấy documents theo batch filter
            cursor.execute(f'''
                SELECT 
                    d.id,
                    d.image_header_path,
                    d.image_content_path,
                    d.kie_header_data,
                    d.kie_content_data
                FROM documents d
                JOIN batches b ON d.batch_id = b.id
                WHERE b.created_by = ? {batch_filter}
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
            
            # Đếm available pieces
            available_count = 0
            
            for doc in documents:
                doc_id, header_path, content_path, kie_header_data, kie_content_data = doc
                
                # ✅ FIX: Bỏ qua document đã được gán 'full'
                if doc_id in assigned_full_docs:
                    continue
                
                # Header piece
                if header_path and kie_header_data:
                    piece_key = (doc_id, 'header')
                    if (piece_key not in assigned_pieces and 
                        self._has_valid_kie_data(kie_header_data, 'header')):
                        available_count += 1
                
                # Content piece
                if content_path and kie_content_data:
                    piece_key = (doc_id, 'content')
                    if (piece_key not in assigned_pieces and 
                        self._has_valid_kie_data(kie_content_data, 'content')):
                        available_count += 1
            
            return available_count
            
        except Exception as e:
            print(f"❌ Lỗi đếm pieces by batches: {e}")
            return 0
        finally:
            conn.close()

    def update_batch_status(self, batch_id: str, status: str):
        """Update batch status"""
        try:
            conn = self.get_connection()
            cursor = conn.cursor()
            
            cursor.execute("""
                UPDATE batches 
                SET status = ?
                WHERE id = ?
            """, (status, batch_id))
            
            conn.commit()
            conn.close()
            
            print(f"✅ Updated batch {batch_id} status to {status}")
            
        except Exception as e:
            print(f"❌ Error updating batch status: {e}")

    def update_batch_processed_count(self, batch_id: str, processed_count: int):
        """Update batch processed files count"""
        try:
            conn = self.get_connection()
            cursor = conn.cursor()
            
            cursor.execute("""
                UPDATE batches 
                SET processed_files = ?
                WHERE id = ?
            """, (processed_count, batch_id))
            
            conn.commit()
            conn.close()
            
        except Exception as e:
            print(f"❌ Error updating batch processed count: {e}")

    def get_documents_by_batch(self, batch_id: str):
        """Get all documents in a batch with their status"""
        try:
            conn = self.get_connection()
            cursor = conn.cursor()
            
            cursor.execute("""
                SELECT id, filename, status, document_type, processed_at
                FROM documents 
                WHERE batch_id = ?
                ORDER BY filename
            """, (batch_id,))
            
            documents = []
            for row in cursor.fetchall():
                documents.append({
                    'id': row[0],
                    'filename': row[1], 
                    'status': row[2],
                    'document_type': row[3],
                    'processed_at': row[4]
                })
            
            conn.close()
            return documents
            
        except Exception as e:
            print(f"❌ Error getting documents by batch: {e}")
            return []


    def get_batch_documents_paginated(self, batch_id, page=1, page_size=10, search='', status_filter=''):
        """Lấy documents của batch với pagination và filtering"""
        conn = self.get_connection()
        cursor = conn.cursor()
        
        try:
            # Build WHERE clause
            where_conditions = ['batch_id = ?']
            params = [batch_id]
            
            # Add search condition
            if search:
                where_conditions.append('filename LIKE ?')
                params.append(f'%{search}%')
            
            # Add status filter
            if status_filter:
                where_conditions.append('status = ?')
                params.append(status_filter)
            
            where_clause = ' AND '.join(where_conditions)
            
            # ✅ GET TOTAL COUNT FIRST
            count_query = f'''
                SELECT COUNT(*) 
                FROM documents 
                WHERE {where_clause}
            '''
            cursor.execute(count_query, params)
            total_count = cursor.fetchone()[0]
            
            # Calculate pagination
            total_pages = (total_count + page_size - 1) // page_size  # Ceiling division
            offset = (page - 1) * page_size
            
            # ✅ FIX: SELECT đúng columns tồn tại trong schema
            documents_query = f'''
                SELECT id, filename, status, document_type, processed_at, created_at,
                    kie_header_data, kie_content_data, 
                    image_full_path, image_header_path, image_content_path, total_pages
                FROM documents 
                WHERE {where_clause}
                ORDER BY created_at DESC
                OFFSET ? ROWS FETCH NEXT ? ROWS ONLY
            '''
            
            cursor.execute(documents_query, params + [offset, page_size])
            documents = []
            
            for row in cursor.fetchall():
                doc = {
                    'id': row[0],
                    'filename': row[1], 
                    'status': row[2],
                    'document_type': row[3],
                    'processed_at': self._safe_datetime_convert(row[4]),   # ✅ SAFE CONVERT
                    'created_at': self._safe_datetime_convert(row[5]),     # ✅ SAFE CONVERT
                     'total_pages': row[11] or 1
                }
                
                # Parse JSON fields safely
                try:
                    doc['kie_header_data'] = json.loads(row[6]) if row[6] else {}
                    doc['kie_content_data'] = json.loads(row[7]) if row[7] else {}
                except:
                    doc['kie_header_data'] = {}
                    doc['kie_content_data'] = {}
                
                # ✅ BUILD image_paths object từ các columns riêng biệt
                doc['image_paths'] = {
                    'full': row[8],      # image_full_path
                    'header': row[9],    # image_header_path
                    'content': row[10]   # image_content_path
                }
                
                documents.append(doc)
            
            return {
                'documents': documents,
                'total_count': total_count,
                'total_pages': total_pages,
                'current_page': page,
                'page_size': page_size
            }
            
        except Exception as e:
            print(f"❌ Error getting paginated documents: {e}")
            raise
        finally:
            conn.close()

    def delete_work_items_by_doc_ids(self, doc_ids):
        """Xóa tất cả work_items liên quan đến các doc_ids"""
        if not doc_ids:
            return
        
        conn = self.get_connection()
        cursor = conn.cursor()
        
        try:
            # Tạo placeholders cho IN clause
            placeholders = ','.join(['?' for _ in doc_ids])
            
            # Xóa work_items liên quan
            delete_query = f'''
                DELETE FROM work_items 
                WHERE doc_id IN ({placeholders})
            '''
            
            cursor.execute(delete_query, doc_ids)
            deleted_count = cursor.rowcount
            
            conn.commit()
            print(f"✅ Đã xóa {deleted_count} work_items liên quan đến documents")
            
        except Exception as e:
            print(f"❌ Lỗi khi xóa work_items: {e}")
            conn.rollback()
            raise
        finally:
            conn.close()
    def reset_documents_for_restart(self, batch_ids):
        """Reset tất cả documents trong batches về pending để restart được - SAFE VERSION"""
        if not batch_ids:
            return
        
        conn = self.get_connection()
        cursor = conn.cursor()
        
        try:
            # Tạo placeholders cho IN clause
            placeholders = ','.join(['?' for _ in batch_ids])
            
            # ✅ SAFE VERSION: Chỉ reset những columns chắc chắn có
            cursor.execute(f'''
                UPDATE documents 
                SET status = 'pending', 
                    processed_at = NULL
                WHERE batch_id IN ({placeholders})
            ''', batch_ids)
            
            documents_updated = cursor.rowcount
            
            # Reset batch status
            cursor.execute(f'''
                UPDATE batches 
                SET status = 'pending', 
                    processed_files = 0
                WHERE id IN ({placeholders})
            ''', batch_ids)
            
            batches_updated = cursor.rowcount
            
            conn.commit()
            print(f"✅ Reset {documents_updated} documents and {batches_updated} batches for restart")
            
        except Exception as e:
            conn.rollback()
            print(f"❌ Error resetting documents for restart: {e}")
            raise
        finally:
            conn.close()

    def reset_batch_documents_for_stop(self, batch_id):
        """Reset documents đang processing hoặc error về uploaded"""
        conn = self.get_connection()
        cursor = conn.cursor()
        
        try:
            # ✅ Reset documents có status = processing hoặc error
            cursor.execute("""
                UPDATE documents 
                SET status = 'uploaded', 
                    processed_at = NULL
                WHERE batch_id = ? 
                AND status IN ('processing', 'error')
            """, (batch_id,))
            
            documents_reset = cursor.rowcount
            
            conn.commit()
            print(f"✅ Reset {documents_reset} documents in batch {batch_id} to uploaded")
            
            return documents_reset
            
        except Exception as e:
            print(f"❌ Error resetting batch documents: {e}")
            conn.rollback()
            raise
        finally:
            conn.close()
