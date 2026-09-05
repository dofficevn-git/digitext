from datetime import datetime
import os
from flask import Blueprint, request, jsonify, render_template

from auth import login_required, get_current_user
from parallel_processor import JobStatus, get_parallel_processor

# Tạo blueprint
parallel_bp = Blueprint('parallel', __name__)

# ===== SERVE HTML PAGE =====
@parallel_bp.route('/parallel-extraction-page')
@login_required
def serve_parallel_extraction():
    """Serve trang trích xuất song song với VLLM"""
    return render_template('parallel-extraction.html')

# ===== API ENDPOINTS =====

@parallel_bp.route('/api/parallel/server-status', methods=['GET'])
@login_required
def get_vllm_server_status():
    """API kiểm tra trạng thái VLLM server"""
    try:
        import requests
        
        vllm_base_url = os.getenv("VLLM_BASE_URL", "http://localhost:3000")
        vllm_model = os.getenv("VLLM_MODEL", "google/gemma-2-9b-it")
        vllm_max_concurrent = int(os.getenv("VLLM_MAX_CONCURRENT", "10"))
        
        # Test connection to VLLM server
        try:
            response = requests.get(f"{vllm_base_url}/health", timeout=10)
            server_status = 'online' if response.status_code == 200 else 'error'
        except:
            server_status = 'offline'
        
        return jsonify({
            'success': True,
            'vllm_info': {
                'provider': 'vllm',
                'server_url': vllm_base_url,
                'model': vllm_model,
                'max_concurrent': vllm_max_concurrent,
                'server_status': server_status,
                'timeout': int(os.getenv("VLLM_TIMEOUT", "300")),
                'temperature': float(os.getenv("VLLM_TEMPERATURE", "0.1"))
            }
        })
        
    except Exception as e:
        return jsonify({'success': False, 'error': str(e)}), 500

@parallel_bp.route('/api/parallel/start-extraction', methods=['POST'])
@login_required
def start_parallel_extraction():
    """API bắt đầu trích xuất song song với VLLM (no API keys needed)"""
    try:
        from app import db_manager
        
        user = get_current_user()
        data = request.get_json()
        
        batch_ids = data.get('batch_ids', [])
        max_workers = data.get('max_workers', 10)
        timeout = data.get('timeout', 300)
        
        if not batch_ids:
            return jsonify({
                'success': False,
                'error': 'Vui lòng chọn ít nhất một batch'
            }), 400
        
        # VLLM setup - no API keys needed
        vllm_max_concurrent = int(os.getenv("VLLM_MAX_CONCURRENT", "10"))
        if max_workers > vllm_max_concurrent:
            max_workers = vllm_max_concurrent
            print(f"🔧 Adjusted workers to VLLM limit: {max_workers}")
        
        # Tạo processor và job
        processor = get_parallel_processor(db_manager)
        job_id = f"job_{datetime.now().strftime('%Y%m%d_%H%M%S')}_{user['id']}"
        
        job_created_id = processor.create_job(
            job_id=job_id,
            batch_ids=batch_ids,
            max_workers=max_workers,
            timeout=timeout,
            created_by=user['id']
        )
        
        if not job_created_id:
            return jsonify({
                'success': False,
                'error': 'Không thể tạo job'
            }), 500
        
        # Start job with VLLM (no API keys needed)
        job_started = processor.start_job(
            job_id=job_created_id, 
            batch_ids=batch_ids,
            force_restart=True
        )
        
        if not job_started:
            return jsonify({
                'success': False,
                'error': 'Không thể bắt đầu job'
            }), 500
        
        return jsonify({
            'success': True,
            'job_id': job_created_id,
            'message': f'Bắt đầu xử lý {len(batch_ids)} batches với VLLM server',
            'max_workers': max_workers,
            'force_restart': True,
            'api_info': {
                'provider': 'vllm',
                'server_url': os.getenv("VLLM_BASE_URL", "http://localhost:3000"),
                'model': os.getenv("VLLM_MODEL", "google/gemma-2-9b-it"),
                'max_concurrent': vllm_max_concurrent
            }
        })
        
    except Exception as e:
        print(f"❌ Error in start_parallel_extraction: {e}")
        import traceback
        traceback.print_exc()
        return jsonify({
            'success': False,
            'error': f'Lỗi server: {str(e)}'
        }), 500

@parallel_bp.route('/api/parallel/job-status/<job_id>', methods=['GET'])
@login_required
def get_job_status(job_id):
    """API lấy trạng thái job"""
    try:
        from app import db_manager
        
        processor = get_parallel_processor(db_manager)
        status = processor.get_job_status(job_id)
        
        if status:
            return jsonify({'success': True, 'status': status})
        else:
            return jsonify({'success': False, 'error': 'Job not found'}), 404
            
    except Exception as e:
        return jsonify({'success': False, 'error': str(e)}), 500

@parallel_bp.route('/api/parallel/pause-job/<job_id>', methods=['POST'])
@login_required
def pause_job(job_id):
    """API tạm dừng job"""
    try:
        from app import db_manager
        
        processor = get_parallel_processor(db_manager)
        success = processor.pause_job(job_id)
        
        if success:
            return jsonify({'success': True, 'message': 'Job đã được tạm dừng'})
        else:
            return jsonify({'success': False, 'error': 'Không thể tạm dừng job'}), 400
            
    except Exception as e:
        return jsonify({'success': False, 'error': str(e)}), 500

@parallel_bp.route('/api/parallel/stop-job/<job_id>', methods=['POST'])
@login_required
def stop_job(job_id):
    """API dừng job"""
    try:
        from app import db_manager
        
        processor = get_parallel_processor(db_manager)
        success = processor.stop_job(job_id)
        
        if success:
            return jsonify({'success': True, 'message': 'Job đã được dừng'})
        else:
            return jsonify({'success': False, 'error': 'Không thể dừng job'}), 400
            
    except Exception as e:
        return jsonify({'success': False, 'error': str(e)}), 500

@parallel_bp.route('/api/parallel/jobs', methods=['GET'])
@login_required
def get_all_jobs():
    """API lấy danh sách tất cả jobs"""
    try:
        from app import db_manager
        
        processor = get_parallel_processor(db_manager)
        
        jobs_summary = []
        for job_id, job in processor.jobs.items():
            jobs_summary.append({
                'job_id': job_id,
                'status': job.status.value,
                'total_files': job.total_files,
                'completed_files': job.completed_files,
                'error_files': job.error_files,
                'created_at': job.created_at.isoformat() if job.created_at else None,
                'batch_count': len(job.batch_ids),
                'batch_ids': job.batch_ids  # ✅ ĐÚNG RỒI
            })
        
        return jsonify({'success': True, 'jobs': jobs_summary})
        
    except Exception as e:
        return jsonify({'success': False, 'error': str(e)}), 500

@parallel_bp.route('/api/parallel/cleanup-jobs', methods=['POST'])
@login_required
def cleanup_old_jobs():
    """API dọn dẹp jobs cũ"""
    try:
        from app import db_manager
        
        data = request.get_json()
        max_age_hours = data.get('max_age_hours', 24)
        
        processor = get_parallel_processor(db_manager)
        processor.cleanup_completed_jobs(max_age_hours)
        
        return jsonify({'success': True, 'message': f'Đã dọn dẹp jobs cũ hơn {max_age_hours} giờ'})
        
    except Exception as e:
        return jsonify({'success': False, 'error': str(e)}), 500

@parallel_bp.route('/api/batch/<batch_id>/reset-for-stop', methods=['POST'])
@login_required
def reset_batch_for_stop(batch_id):
    """Reset batch và documents về trạng thái ban đầu trước khi dừng job"""
    try:
        from app import db_manager
        
        # ✅ BƯỚC 1: Reset documents về uploaded
        # Chỉ reset những documents đang processing hoặc error
        documents_reset = db_manager.reset_batch_documents_for_stop(batch_id)
        
        # ✅ BƯỚC 2: Reset batch status về uploaded
        db_manager.update_batch_status(batch_id, 'uploaded')
        
        # ✅ BƯỚC 3: Reset processed count
        db_manager.update_batch_processed_count(batch_id, 0)
        
        print(f"✅ Reset batch {batch_id} for stop: {documents_reset} documents reset")
        
        return jsonify({
            'success': True,
            'message': f'Đã reset batch {batch_id}',
            'documents_reset': documents_reset,
            'batch_id': batch_id
        })
        
    except Exception as e:
        print(f"❌ Error resetting batch {batch_id} for stop: {e}")
        return jsonify({
            'success': False,
            'error': str(e)
        }), 500
@parallel_bp.route('/api/parallel/stop-all-jobs', methods=['POST'])
@login_required
def stop_all_jobs():
    """API dừng tất cả jobs đang chạy"""
    try:
        from app import db_manager
        
        data = request.get_json()
        reason = data.get('reason', 'user_requested')
        
        processor = get_parallel_processor(db_manager)
        
        # Stop all active jobs
        stopped_jobs = []
        for job_id, job in processor.jobs.items():
            if job.status == JobStatus.PROCESSING:
                success = processor.stop_job(job_id)
                if success:
                    stopped_jobs.append(job_id)
        
        # Reset batch status về uploaded
        for job_id, job in processor.jobs.items():
            if job_id in stopped_jobs:
                for batch_id in job.batch_ids:
                    try:
                        db_manager.update_batch_status(batch_id, 'uploaded')
                        # Reset processing documents về uploaded
                        db_manager.reset_processing_documents_to_uploaded(batch_id)
                    except Exception as e:
                        print(f"Error resetting batch {batch_id}: {e}")
        
        return jsonify({
            'success': True,
            'message': f'Đã dừng {len(stopped_jobs)} jobs',
            'stopped_jobs': stopped_jobs,
            'reason': reason
        })
        
    except Exception as e:
        return jsonify({'success': False, 'error': str(e)}), 500

@parallel_bp.route('/api/batch/<batch_id>/reset-status', methods=['POST'])
@login_required
def reset_batch_status(batch_id):
    """Reset batch status về uploaded"""
    try:
        from app import db_manager
        
        # Reset batch status
        db_manager.update_batch_status(batch_id, 'uploaded')
        
        # Reset documents status về uploaded
        db_manager.reset_processing_documents_to_uploaded(batch_id)
        
        return jsonify({
            'success': True,
            'message': f'Batch {batch_id} đã được reset về trạng thái uploaded'
        })
        
    except Exception as e:
        return jsonify({'success': False, 'error': str(e)}), 500

# ===== ERROR HANDLERS =====
@parallel_bp.errorhandler(404)
def handle_404(e):
    return jsonify({'success': False, 'error': 'Endpoint not found'}), 404

@parallel_bp.errorhandler(500)
def handle_500(e):
    return jsonify({'success': False, 'error': 'Internal server error'}), 500