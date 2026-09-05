import os
import json
import time
import uuid
import threading
from datetime import datetime
from concurrent.futures import ThreadPoolExecutor, as_completed
from typing import List, Dict, Any, Optional
from dataclasses import dataclass
from enum import Enum

from main import DocumentKIE
from utils import copy_images_to_static, cleanup_after_processing

class JobStatus(Enum):
    PENDING = "pending"
    PROCESSING = "processing"
    COMPLETED = "completed"
    PAUSED = "paused"
    ERROR = "error"
    CANCELLED = "cancelled"

@dataclass
class WorkerInfo:
    worker_id: int
    status: str = "idle"
    current_file: Optional[str] = None
    processed_count: int = 0
    error_count: int = 0
    start_time: Optional[float] = None

@dataclass
class JobInfo:
    job_id: str
    batch_ids: List[str]
    status: JobStatus
    total_files: int = 0
    completed_files: int = 0
    error_files: int = 0
    processing_files: int = 0
    created_at: datetime = None
    started_at: Optional[datetime] = None
    completed_at: Optional[datetime] = None
    error_message: Optional[str] = None
    max_workers: int = 3
    timeout: int = 300

class ParallelKIEProcessor:
    """VLLM-only parallel processor"""
    
    def __init__(self, db_manager):
        self.db_manager = db_manager
        self.jobs: Dict[str, JobInfo] = {}
        self.workers: Dict[str, List[WorkerInfo]] = {}
        self.executors: Dict[str, ThreadPoolExecutor] = {}
        self.lock = threading.Lock()
        
        # Load VLLM config
        self.max_workers = int(os.getenv('VLLM_MAX_CONCURRENT', '10'))
        self.default_timeout = int(os.getenv('VLLM_TIMEOUT', '300'))
        self.retry_attempts = int(os.getenv('RETRY_ATTEMPTS', '2'))
        
        print(f"✅ VLLM ParallelKIEProcessor initialized")
        print(f"⚙️ Max workers: {self.max_workers}, Timeout: {self.default_timeout}s, Retries: {self.retry_attempts}")

    def create_job(self, job_id: str = None, batch_ids: List[str] = None, max_workers: int = None, timeout: int = None, created_by: str = None) -> str:
        """Tạo job mới cho VLLM"""
        if job_id is None:
            job_id = str(uuid.uuid4())
        
        if batch_ids is None:
            batch_ids = []
        
        # Optimize for VLLM
        if max_workers is None:
            max_workers = self.max_workers
        else:
            max_workers = min(max_workers, self.max_workers)
            
        if timeout is None:
            timeout = self.default_timeout
        
        # Count total files
        total_files = 0
        for batch_id in batch_ids:
            documents = self.db_manager.get_documents_for_processing(batch_id)
            total_files += len(documents)
        
        job = JobInfo(
            job_id=job_id,
            batch_ids=batch_ids,
            status=JobStatus.PENDING,
            total_files=total_files,
            created_at=datetime.now(),
            max_workers=max_workers,
            timeout=timeout
        )
        
        with self.lock:
            self.jobs[job_id] = job
            self.workers[job_id] = []
        
        print(f"✅ Created VLLM job {job_id}")
        print(f"📊 {total_files} files from {len(batch_ids)} batches, {max_workers} workers")
        return job_id

    def start_job(self, job_id: str, api_keys: List[str] = None, batch_ids: List[str] = None, force_restart: bool = True) -> bool:
        """Start VLLM processing job (api_keys ignored)"""
        with self.lock:
            if job_id not in self.jobs:
                print(f"❌ Job {job_id} not found")
                return False
            
            job = self.jobs[job_id]
            
            # Force restart if needed
            if force_restart and batch_ids:
                try:
                    print(f"🔄 Force restarting batches: {batch_ids}")
                    self.db_manager.reset_documents_for_restart(batch_ids)
                    
                    # Kill existing executor
                    if job_id in self.executors:
                        print(f"🛑 Shutting down existing executor for job {job_id}")
                        self.executors[job_id].shutdown(wait=False)
                        del self.executors[job_id]
                        
                except Exception as e:
                    print(f"❌ Error in force restart: {e}")
                    job.status = JobStatus.ERROR
                    job.error_message = f"Force restart failed: {str(e)}"
                    return False
            
            try:
                print(f"🚀 Setting up VLLM concurrent processing...")
                
                # Setup executor with optimal workers
                self.executors[job_id] = ThreadPoolExecutor(max_workers=job.max_workers)
                
                # Initialize workers
                self.workers[job_id] = [
                    WorkerInfo(worker_id=i) for i in range(job.max_workers)
                ]
                
                # Start processing
                job.status = JobStatus.PROCESSING
                job.started_at = datetime.now()
                job.error_message = None
                
                future = self.executors[job_id].submit(self._process_job, job_id)
                job.future = future
                
                print(f"✅ VLLM job {job_id} started with {job.max_workers} workers")
                return True
                
            except Exception as e:
                print(f"❌ Error starting VLLM job {job_id}: {e}")
                job.status = JobStatus.ERROR
                job.error_message = str(e)
                
                # Cleanup on error
                if job_id in self.executors:
                    self.executors[job_id].shutdown(wait=False)
                    del self.executors[job_id]
                if job_id in self.workers:
                    del self.workers[job_id]
                    
                return False

    def _process_job(self, job_id: str):
        """Process job với VLLM"""
        try:
            job = self.jobs[job_id]
            executor = self.executors[job_id]
            
            if job.status == JobStatus.CANCELLED:
                print(f"🛑 Job {job_id} was cancelled before processing")
                return
            
            # Update batch status
            for batch_id in job.batch_ids:
                try:
                    self.db_manager.update_batch_status(batch_id, 'processing')
                except Exception as e:
                    print(f"⚠️ Warning: Could not update batch status: {e}")
            
            # Get all documents
            all_documents = []
            for batch_id in job.batch_ids:
                if job.status == JobStatus.CANCELLED:
                    print(f"🛑 Job {job_id} cancelled during document discovery")
                    return
                    
                documents = self.db_manager.get_documents_for_processing(batch_id)
                all_documents.extend([(batch_id, doc_id, file_path, filename) for doc_id, file_path, filename in documents])
            
            print(f"🚀 Processing {len(all_documents)} documents with {job.max_workers} VLLM workers")
            
            if job.status == JobStatus.CANCELLED:
                print(f"🛑 Job {job_id} cancelled before submitting tasks")
                return
            
            # Submit tasks
            futures = []
            for i, (batch_id, doc_id, file_path, filename) in enumerate(all_documents):
                if job.status == JobStatus.CANCELLED:
                    print(f"🛑 Job {job_id} cancelled during task submission")
                    for future in futures:
                        future.cancel()
                    return
                
                worker_id = i % job.max_workers
                future = executor.submit(
                    self._process_single_document,
                    job_id, worker_id, batch_id, doc_id, file_path, filename
                )
                futures.append(future)
            
            job.futures = futures
            
            # Wait for completion
            completed_count = 0
            error_count = 0
            
            for future in as_completed(futures):
                if job.status in [JobStatus.CANCELLED, JobStatus.PAUSED]:
                    print(f"🛑 Job {job_id} stopping - cancelling remaining futures")
                    for remaining_future in futures:
                        if not remaining_future.done():
                            remaining_future.cancel()
                    break
                
                try:
                    result = future.result(timeout=job.timeout)
                    if result:
                        completed_count += 1
                        job.completed_files += 1
                    else:
                        error_count += 1
                        job.error_files += 1
                        
                    # Update progress
                    if (completed_count + error_count) % 5 == 0:
                        if job.status == JobStatus.CANCELLED:
                            break
                        self._update_batch_progress(job.batch_ids)
                        
                except Exception as e:
                    print(f"❌ Task failed: {e}")
                    error_count += 1
                    job.error_files += 1
            
            # Final completion
            with self.lock:
                if job.status == JobStatus.CANCELLED:
                    print(f"🛑 Job {job_id} was cancelled")
                    for batch_id in job.batch_ids:
                        try:
                            self.db_manager.update_batch_status(batch_id, 'uploaded')
                        except:
                            pass
                else:
                    job.completed_at = datetime.now()
                    self._update_batch_progress(job.batch_ids)
                    
                    total_processed = completed_count + error_count
                    if total_processed == 0:
                        job.status = JobStatus.ERROR
                        job.error_message = "No files were processed"
                        final_batch_status = 'error'
                    elif error_count == 0:
                        job.status = JobStatus.COMPLETED
                        final_batch_status = 'completed'
                    elif completed_count == 0:
                        job.status = JobStatus.ERROR
                        job.error_message = f"All {error_count} files failed"
                        final_batch_status = 'error'
                    else:
                        job.status = JobStatus.COMPLETED
                        job.error_message = f"Completed with {error_count} errors"
                        final_batch_status = 'completed'
                    
                    for batch_id in job.batch_ids:
                        try:
                            self.db_manager.update_batch_status(batch_id, final_batch_status)
                            print(f"✅ Updated batch {batch_id} status to {final_batch_status}")
                        except Exception as e:
                            print(f"❌ Error updating final batch status: {e}")
                    
                    print(f"🎉 VLLM Job {job_id} completed!")
                    print(f"📊 Results: {completed_count} successful, {error_count} errors")
                    print(f"⏱️ Duration: {(job.completed_at - job.started_at).total_seconds():.1f}s")
                    
                    if job_id in self.workers:
                        for worker in self.workers[job_id]:
                            worker.status = "completed"
                            worker.current_file = None
                    
                    job._needs_cleanup = True
                    print(f"📝 Job {job_id} marked for cleanup")
                        
        except Exception as e:
            print(f"❌ VLLM Job {job_id} failed: {e}")
            import traceback
            traceback.print_exc()
            
            with self.lock:
                job.status = JobStatus.ERROR
                job.error_message = str(e)
                job.completed_at = datetime.now()
                
                for batch_id in job.batch_ids:
                    try:
                        self.db_manager.update_batch_status(batch_id, 'error')
                    except:
                        pass
                
                job._needs_cleanup = True
        
    def _process_single_document(self, job_id: str, worker_id: int, batch_id: str, 
                    doc_id: str, file_path: str, filename: str) -> bool:
        """Process single document với VLLM only"""
        
        job = self.jobs.get(job_id)
        if not job or job.status == JobStatus.CANCELLED:
            print(f"🛑 Worker {worker_id}: Job cancelled, skipping {filename}")
            return False
        
        worker = self.workers[job_id][worker_id]
        worker.status = "processing"
        worker.current_file = filename
        worker.start_time = time.time()
        
        for attempt in range(self.retry_attempts):
            if job.status == JobStatus.CANCELLED:
                print(f"🛑 Worker {worker_id}: Job cancelled during retry {attempt}")
                worker.status = "cancelled"
                return False
            
            try:
                if job.status == JobStatus.CANCELLED:
                    print(f"🛑 Worker {worker_id}: Job cancelled before processing {filename}")
                    worker.status = "cancelled"
                    return False
                
                # Update document status
                self.db_manager.update_document(doc_id, status='processing')
                
                # Process with VLLM (no API key needed)
                kie = DocumentKIE()
                
                print(f"🔄 Worker {worker_id} processing: {filename} (attempt {attempt + 1}/{self.retry_attempts}) with VLLM")
                results = kie.process_document(file_path, doc_id)
                
                if job.status == JobStatus.CANCELLED:
                    print(f"🛑 Worker {worker_id}: Job cancelled after processing {filename}")
                    worker.status = "cancelled"
                    cleanup_after_processing(doc_id, results)
                    return False
                
                # Copy images to static
                static_paths = copy_images_to_static(doc_id, results['image_paths'])
                
                # Extract document type
                document_type = "N/A"
                kie_header_data = results.get('kie_header_data', {})
                if isinstance(kie_header_data, dict):
                    document_type = kie_header_data.get('document_type', 'N/A')
                    if not isinstance(document_type, str):
                        if isinstance(document_type, dict) and 'value' in document_type:
                            document_type = document_type['value']
                        else:
                            document_type = str(document_type) if document_type else 'N/A'
                
                print(f"✅ Worker {worker_id} extracted document_type: {document_type} with VLLM")
                
                # Get VLLM stats
                api_stats = kie.get_api_stats() if hasattr(kie, 'get_api_stats') else {}
                
                # Save to database
                self.db_manager.update_document(
                    doc_id,
                    kie_header_data=results.get('kie_header_data', {}),
                    kie_content_data=results.get('kie_content_data', {}),
                    cut_point=results.get('cut_point', 0),
                    image_full_path=static_paths.get('full'),
                    image_header_path=static_paths.get('header'),
                    image_content_path=static_paths.get('content'),
                    header_detection=results.get('header_detection', {}),
                    total_pages=results.get('total_pages', 1),
                    status='completed',
                    processed_at=datetime.now(),
                    document_type=document_type
                )
                
                # Cleanup
                cleanup_after_processing(doc_id, results)
                
                worker.processed_count += 1
                worker.status = "idle"
                worker.current_file = None
                
                print(f"✅ Worker {worker_id} completed: {filename} with VLLM")
                if api_stats:
                    print(f"📊 VLLM stats: {api_stats.get('successful_requests', 0)}/{api_stats.get('total_requests', 0)} success")
                
                return True
                
            except Exception as e:
                if job.status == JobStatus.CANCELLED:
                    print(f"🛑 Worker {worker_id}: Job cancelled during error handling")
                    worker.status = "cancelled"
                    return False
            
                error_type = self._classify_processing_error(str(e))
                print(f"❌ Worker {worker_id} VLLM error ({error_type}): {filename} - {e}")
                
                # VLLM retry logic
                if error_type in ["rate_limit", "service_unavailable", "timeout"]:
                    wait_time = 30 + (attempt * 15)
                    print(f"⏳ VLLM error, waiting {wait_time}s...")
                    if attempt < self.retry_attempts - 1:
                        time.sleep(wait_time)
                        continue
                
                if attempt == self.retry_attempts - 1:
                    try:
                        error_details = {
                            'error': str(e),
                            'error_type': error_type,
                            'attempts': self.retry_attempts,
                            'api_provider': 'vllm',
                            'worker_id': worker_id
                        }
                        
                        self.db_manager.update_document(
                            doc_id,
                            status='error',
                            kie_header_data={'error_details': error_details},
                            kie_content_data={'error_details': error_details}
                        )
                    except:
                        pass
                    
                    cleanup_after_processing(doc_id, None, keep_original=True)
                    worker.error_count += 1
                    worker.status = "error"
                    worker.current_file = None
                    return False
        
        return False

    def _classify_processing_error(self, error_str: str) -> str:
        """Classify VLLM processing errors"""
        error_lower = error_str.lower()
        
        if "rate limit" in error_lower or "429" in error_str:
            return "rate_limit"
        elif "service unavailable" in error_lower or "503" in error_str:
            return "service_unavailable"
        elif "timeout" in error_lower:
            return "timeout"
        elif "connection" in error_lower:
            return "connection_error"
        elif "500" in error_str:
            return "server_error"
        else:
            return "unknown"

    def _update_batch_progress(self, batch_ids: List[str]):
        """Update batch progress"""
        for batch_id in batch_ids:
            try:
                batch_documents = self.db_manager.get_documents_by_batch(batch_id)
                total_docs = len(batch_documents)
                completed_docs = len([d for d in batch_documents if d['status'] == 'completed'])
                error_docs = len([d for d in batch_documents if d['status'] == 'error'])
                processing_docs = len([d for d in batch_documents if d['status'] == 'processing'])
                
                self.db_manager.update_batch_processed_count(batch_id, completed_docs)
                
                processed_total = completed_docs + error_docs
                if processed_total >= total_docs and processing_docs == 0:
                    if error_docs == 0:
                        self.db_manager.update_batch_status(batch_id, 'completed')
                        print(f"✅ Auto-completed batch {batch_id}: {completed_docs}/{total_docs} successful")
                    elif completed_docs == 0:
                        self.db_manager.update_batch_status(batch_id, 'error')
                        print(f"❌ Auto-errored batch {batch_id}: {error_docs}/{total_docs} failed")
                    else:
                        self.db_manager.update_batch_status(batch_id, 'completed')
                        print(f"⚠️ Auto-completed batch {batch_id}: {completed_docs} success, {error_docs} errors")
                else:
                    print(f"📊 Batch {batch_id}: {completed_docs}/{total_docs} completed, {processing_docs} processing")
                
            except Exception as e:
                print(f"❌ Error updating batch progress: {e}")
    
    def get_job_status(self, job_id: str) -> Optional[Dict[str, Any]]:
        """Get job status (no API key stats)"""
        with self.lock:
            if job_id not in self.jobs:
                return None
            
            job = self.jobs[job_id]
            workers = self.workers.get(job_id, [])
            
            processing_count = sum(1 for w in workers if w.status == "processing")
            
            return {
                'job_id': job_id,
                'status': job.status.value,
                'total': job.total_files,
                'completed': job.completed_files,
                'errors': job.error_files,
                'processing': processing_count,
                'remaining': job.total_files - job.completed_files - job.error_files,
                'progress_percentage': round((job.completed_files / job.total_files * 100) if job.total_files > 0 else 0, 1),
                'created_at': job.created_at.isoformat() if job.created_at else None,
                'started_at': job.started_at.isoformat() if job.started_at else None,
                'completed_at': job.completed_at.isoformat() if job.completed_at else None,
                'error_message': job.error_message,
                'max_workers': job.max_workers,
                'timeout': job.timeout,
                'workers': [
                    {
                        'worker_id': w.worker_id,
                        'status': w.status,
                        'current_file': w.current_file,
                        'processed_count': w.processed_count,
                        'error_count': w.error_count,
                        'processing_time': round(time.time() - w.start_time) if w.start_time else 0
                    }
                    for w in workers
                ],
                'api_provider': 'vllm'
            }
    
    def pause_job(self, job_id: str) -> bool:
        """Pause job"""
        with self.lock:
            if job_id in self.jobs and self.jobs[job_id].status == JobStatus.PROCESSING:
                self.jobs[job_id].status = JobStatus.PAUSED
                print(f"⏸️ Job {job_id} paused")
                return True
            return False
    
    def stop_job(self, job_id: str) -> bool:
        """Stop job - KHÔNG RESET BATCH (đã reset từ trước)"""
        with self.lock:
            if job_id not in self.jobs:
                print(f"❌ Job {job_id} not found")
                return False
            
            job = self.jobs[job_id]
            
            if job.status in [JobStatus.COMPLETED, JobStatus.CANCELLED]:
                print(f"⚠️ Job {job_id} already {job.status.value}")
                return False
            
            print(f"🛑 Stopping job {job_id} (status: {job.status.value})")
            
            # Set status to cancelled
            job.status = JobStatus.CANCELLED
            job.completed_at = datetime.now()
            
            # Cancel all futures
            if hasattr(job, 'futures'):
                cancelled_count = 0
                for future in job.futures:
                    if not future.done():
                        future.cancel()
                        cancelled_count += 1
                print(f"🔴 Cancelled {cancelled_count} pending futures")
            
            # Cancel main future if exists
            if hasattr(job, 'future') and not job.future.done():
                job.future.cancel()
            
            # Update workers
            if job_id in self.workers:
                for worker in self.workers[job_id]:
                    worker.status = "cancelled"
                    worker.current_file = None
            
            # Schedule executor cleanup
            if job_id in self.executors:
                threading.Thread(
                    target=self._cleanup_job_executor,
                    args=(job_id,),
                    daemon=True
                ).start()
            
            # ✅ KHÔNG CẦN RESET BATCH Ở ĐÂY - ĐÃ RESET TỪ TRƯỚC
            print(f"✅ Job {job_id} stopped (batch reset was done before)")
            return True
    
    def _cleanup_job_executor(self, job_id: str):
        """Cleanup job executor"""
        try:
            if job_id in self.executors:
                print(f"🧹 Cleaning up executor for job {job_id}")
                executor = self.executors[job_id]
                executor.shutdown(wait=False)
                del self.executors[job_id]
                print(f"✅ Executor cleanup completed for job {job_id}")
        except Exception as e:
            print(f"❌ Error cleaning up executor for job {job_id}: {e}")

    def cleanup_completed_jobs(self, max_age_hours: int = 24):
        """Cleanup old jobs"""
        with self.lock:
            current_time = datetime.now()
            to_remove = []
            to_cleanup_executors = []
            
            for job_id, job in self.jobs.items():
                if hasattr(job, '_needs_cleanup') and job._needs_cleanup:
                    to_cleanup_executors.append(job_id)
                    job._needs_cleanup = False
                
                if (job.status in [JobStatus.COMPLETED, JobStatus.ERROR, JobStatus.CANCELLED] and
                    job.completed_at and 
                    (current_time - job.completed_at).total_seconds() > max_age_hours * 3600):
                    to_remove.append(job_id)
            
            for job_id in to_cleanup_executors:
                threading.Thread(
                    target=self._cleanup_job_executor, 
                    args=(job_id,),
                    daemon=True
                ).start()
            
            for job_id in to_remove:
                if job_id in self.jobs:
                    del self.jobs[job_id]
                if job_id in self.workers:
                    del self.workers[job_id]
                
                if job_id in self.executors:
                    threading.Thread(
                        target=self._cleanup_job_executor,
                        args=(job_id,),
                        daemon=True
                    ).start()
                    
                print(f"🗑️ Cleaned up old VLLM job: {job_id}")

# Global processor instance
_parallel_processor = None

def get_parallel_processor(db_manager):
    """Get global VLLM parallel processor instance"""
    global _parallel_processor
    if _parallel_processor is None:
        _parallel_processor = ParallelKIEProcessor(db_manager)
    return _parallel_processor