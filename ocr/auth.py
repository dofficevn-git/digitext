from functools import wraps
from flask import session, request, jsonify, redirect, url_for

def login_required(f):
    """Decorator yêu cầu đăng nhập"""
    @wraps(f)
    def decorated_function(*args, **kwargs):
        if 'user_id' not in session:
            # Nếu là AJAX request thì trả JSON error
            if request.is_json or request.headers.get('X-Requested-With') == 'XMLHttpRequest':
                return jsonify({'error': 'Authentication required', 'redirect': '/login'}), 401
            # Nếu là request thường thì redirect
            return redirect('/login')
        return f(*args, **kwargs)
    return decorated_function

def role_required(*allowed_roles):
    """Decorator yêu cầu role cụ thể"""
    def decorator(f):
        @wraps(f)
        def decorated_function(*args, **kwargs):
            if 'user_role' not in session or session['user_role'] not in allowed_roles:
                if request.is_json:
                    return jsonify({'error': 'Insufficient permissions'}), 403
                return jsonify({'error': 'Access denied'}), 403
            return f(*args, **kwargs)
        return decorated_function
    return decorator

def admin_required(f):
    """Decorator chỉ cho phép admin"""
    return role_required('admin')(f)

def get_current_user():
    """Lấy thông tin user hiện tại từ session"""
    if 'user_id' in session:
        return {
            'id': session['user_id'],
            'username': session['username'],
            'full_name': session['full_name'],
            'role': session['user_role'],
            'email': session.get('email', '')
        }
    return None

def can_access_batch(db_manager, user_id, user_role, batch_id):
    """Kiểm tra user có quyền truy cập batch không"""
    if user_role == 'admin':
        return True
    
    # Lấy thông tin batch
    conn = db_manager.get_connection()
    cursor = conn.cursor()
    
    if user_role == 'customer':
        # Customer có thể xem batch của mình và cooperator được gán
        cursor.execute('''
            SELECT COUNT(*) FROM batches b
            LEFT JOIN user_relationships ur ON b.created_by = ur.cooperator_id
            WHERE b.id = ? AND (b.created_by = ? OR ur.customer_id = ?)
        ''', (batch_id, user_id, user_id))
    else:  # cooperator
        # Cooperator chỉ xem batch của mình
        cursor.execute('''
            SELECT COUNT(*) FROM batches b
            WHERE b.id = ? AND b.created_by = ?
        ''', (batch_id, user_id))
    
    count = cursor.fetchone()[0]
    conn.close()
    
    return count > 0