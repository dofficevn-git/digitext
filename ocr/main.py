from email.mime import base
import json
import random
import threading
import time
import cv2
import os
import uuid
from google import genai
from google.genai import types
import fitz  # PyMuPDF for PDF processing
from PIL import Image
import numpy as np
from dotenv import load_dotenv
from ultralytics import YOLO
import torch
from document_config import DocumentTypeConfig
import base64
import requests
import re
# Load environment variables từ .env file
load_dotenv()
class YOLOModelManager:
    _instance = None
    _model = None
    _lock = threading.Lock()
    
    @classmethod
    def get_model(cls, model_path="best.pt"):
        """Thread-safe YOLO model singleton"""
        if cls._instance is None:
            with cls._lock:
                if cls._instance is None:
                    try:
                        from ultralytics import YOLO
                        cls._model = YOLO(model_path)
                        cls._instance = cls
                        print(f"✅ Loaded shared YOLO model: {model_path}")
                    except Exception as e:
                        print(f"❌ Failed to load YOLO model: {e}")
                        cls._model = None
        return cls._model
class GoogleAPIRateLimiter:
    """Smart rate limiter for Google AI Studio API"""
    
    def __init__(self):
        self.request_times = []
        self.rpm_limit = int(os.getenv('API_RPM_LIMIT', 15))  # Conservative for free tier
        self.min_interval = 60 / self.rpm_limit  # Seconds between requests
        self.lock = threading.Lock()
        
        print(f"⚙️ Google API Rate Limiter initialized: {self.rpm_limit} RPM ({self.min_interval:.1f}s interval)")
    
    def wait_if_needed(self):
        """Wait if needed to respect rate limits"""
        with self.lock:
            now = time.time()
            
            # Remove old requests (older than 1 minute)
            self.request_times = [t for t in self.request_times if now - t < 60]
            
            # Check if we're at RPM limit
            if len(self.request_times) >= self.rpm_limit:
                oldest_request = min(self.request_times)
                wait_time = 60 - (now - oldest_request) + 1  # +1 second buffer
                if wait_time > 0:
                    print(f"⏳ RPM limit protection: waiting {wait_time:.1f}s")
                    time.sleep(wait_time)
                    now = time.time()  # Update time after wait
            
            # Ensure minimum interval between requests
            if self.request_times:
                time_since_last = now - max(self.request_times)
                if time_since_last < self.min_interval:
                    wait_time = self.min_interval - time_since_last
                    print(f"⏱️ Interval protection: waiting {wait_time:.1f}s")
                    time.sleep(wait_time)
                    now = time.time()
            
            # Record this request
            self.request_times.append(now)
            print(f"📊 Request recorded. Current RPM: {len(self.request_times)}/{self.rpm_limit}")

class DocumentKIE:
    def __init__(self, yolo_model_path="best.pt"):
        # Khởi tạo YOLO model
        try:
            self.yolo_model = YOLOModelManager.get_model(yolo_model_path)
            print(f"Đã load YOLO model: {yolo_model_path}")
        except Exception as e:
            print(f"Lỗi khi load YOLO model: {e}")
            self.yolo_model = None
        
        # ✅ INITIALIZE COMMON ATTRIBUTES FIRST
        self.api_provider = os.getenv("API_PROVIDER", "vllm").lower()
        self.vllm_enabled = os.getenv("VLLM_ENABLED", "false").lower() == "true"
        self.stats_lock = threading.Lock()
        # ✅ INITIALIZE REQUEST STATS FOR ALL PROVIDERS
        self.request_stats = {
            'total_requests': 0,
            'successful_requests': 0,
            'failed_requests': 0,
            'rate_limited_requests': 0,
            'quota_exceeded_requests': 0
        }
        
        print(f"🤖 API Provider: {self.api_provider}")
        print(f"🔌 VLLM Enabled: {self.vllm_enabled}")
        
        # ✅ INITIALIZE BASED ON PROVIDER
        if self.api_provider == "vllm" and self.vllm_enabled:
            self.init_vllm_client()
       
        
        import sys
        
        if getattr(sys, 'frozen', False):
            # ✅ KHI CHẠY EXE - DÙNG WORKING DIRECTORY
            self.base_dir = os.getcwd()
            print(f"🚀 Running as EXE - using working directory: {self.base_dir}")
        else:
            # ✅ KHI CHẠY PYTHON SCRIPT - DÙNG SOURCE FOLDER
            self.base_dir = os.path.dirname(os.path.abspath(__file__))
            print(f"🐍 Running as script - using source folder: {self.base_dir}")
        
        self.temp_dir = os.path.join(self.base_dir, "temp_processing")
        self.output_dir = os.path.join(self.base_dir, "output")
        
        # ✅ ENSURE FOLDERS EXIST
        os.makedirs(self.temp_dir, exist_ok=True)
        os.makedirs(self.output_dir, exist_ok=True)
        os.makedirs(os.path.join(self.base_dir, "static", "processed_images"), exist_ok=True)
        
        print(f"📁 Temp directory: {self.temp_dir}")
        print(f"📁 Output directory: {self.output_dir}")
    def _update_stats(self, stat_name: str, increment: int = 1):
        """Thread-safe stats update"""
        with self.stats_lock:
            if stat_name in self.request_stats:
                self.request_stats[stat_name] += increment
            else:
                print(f"⚠️ Unknown stat name: {stat_name}")

    def init_vllm_client(self):
        """Initialize VLLM client configuration"""
        self.vllm_base_url = os.getenv("VLLM_BASE_URL", "http://localhost:3000")
        self.vllm_model = os.getenv("VLLM_MODEL", "google/gemma-3-12b-it")
        self.vllm_max_tokens = int(os.getenv("VLLM_MAX_TOKENS", "8192"))
        self.vllm_temperature = float(os.getenv("VLLM_TEMPERATURE", "0.1"))
        self.vllm_timeout = int(os.getenv("VLLM_TIMEOUT", "300"))
        self.vllm_max_concurrent = int(os.getenv("VLLM_MAX_CONCURRENT", "10"))
        
        # ✅ NO RATE LIMITER FOR VLLM (local server)
        self.rate_limiter = None
        
        print(f"🚀 VLLM Configuration:")
        print(f"   📡 Base URL: {self.vllm_base_url}")
        print(f"   🤖 Model: {self.vllm_model}")
        print(f"   🎯 Max tokens: {self.vllm_max_tokens}")
        print(f"   🌡️ Temperature: {self.vllm_temperature}")
        print(f"   ⏱️ Timeout: {self.vllm_timeout}s")
        
        # Test connection
        try:
            response = requests.get(f"{self.vllm_base_url}/health", timeout=10)
            if response.status_code == 200:
                print("✅ VLLM server connection successful")
            else:
                print(f"⚠️ VLLM server responded with status: {response.status_code}")
        except Exception as e:
            print(f"❌ Cannot connect to VLLM server: {e}")
    
    def image_to_base64(self, image_path):
        """Convert image file to base64 string"""
        try:
            with open(image_path, 'rb') as image_file:
                encoded_string = base64.b64encode(image_file.read()).decode('utf-8')
                return encoded_string
        except Exception as e:
            print(f"❌ Error converting image to base64: {e}")
            return None
    def call_vllm_api_with_retry(self, image_path, prompt, max_retries=3):
        """Call VLLM API với retry logic và thread-safe stats"""
        for attempt in range(max_retries):
            try:
                # ✅ THREAD-SAFE STATS UPDATE
                self._update_stats('total_requests')
                
                # ✅ CONVERT IMAGE TO BASE64
                base64_image = self.image_to_base64(image_path)
                if not base64_image:
                    print(f"❌ Cannot convert image to base64: {image_path}")
                    continue
                
                # ✅ VLLM REQUEST PAYLOAD
                payload = {
                    "model": self.vllm_model,
                    "messages": [
                        {
                            "role": "user",
                            "content": [
                                {
                                    "type": "text",
                                    "text": prompt
                                },
                                {
                                    "type": "image_url",
                                    "image_url": {
                                        "url": f"data:image/png;base64,{base64_image}"
                                    }
                                }
                            ]
                        }
                    ],
                    "max_tokens": self.vllm_max_tokens,
                    "temperature": self.vllm_temperature,
                    "top_p": 0.9,
                    "stop": None
                }
                
                # ✅ SEND REQUEST TO VLLM SERVER
                print(f"🚀 VLLM API call (attempt {attempt + 1}): {self.vllm_base_url}")
                
                response = requests.post(
                    f"{self.vllm_base_url}/v1/chat/completions",
                    json=payload,
                    timeout=self.vllm_timeout,
                    headers={
                        "Content-Type": "application/json",
                        "Accept": "application/json"
                    }
                )
                
                print(f"📡 VLLM response status: {response.status_code}")
                
                # ✅ HANDLE SUCCESS RESPONSE
                if response.status_code == 200:
                    try:
                        response_data = response.json()
                        
                        # ✅ EXTRACT CONTENT FROM RESPONSE
                        if 'choices' in response_data and len(response_data['choices']) > 0:
                            content = response_data['choices'][0]['message']['content']
                            
                            # ✅ THREAD-SAFE SUCCESS STATS
                            self._update_stats('successful_requests')
                            
                            print(f"✅ VLLM API success: {len(content)} chars")
                            return content.strip()
                        else:
                            print(f"❌ VLLM response missing choices: {response_data}")
                            raise Exception("No choices in VLLM response")
                            
                    except json.JSONDecodeError as e:
                        print(f"❌ VLLM JSON decode error: {e}")
                        print(f"Raw response: {response.text[:200]}...")
                        raise Exception(f"Invalid JSON response from VLLM: {e}")
                
                # ✅ HANDLE ERROR RESPONSES
                else:
                    # ✅ THREAD-SAFE ERROR STATS
                    self._update_stats('failed_requests')
                    
                    try:
                        error_data = response.json()
                        error_message = error_data.get('error', {}).get('message', 'Unknown error')
                    except:
                        error_message = response.text[:200] if response.text else 'No error message'
                    
                    print(f"❌ VLLM API error {response.status_code}: {error_message}")
                    
                    # ✅ HANDLE SPECIFIC ERROR CODES
                    if response.status_code == 429:
                        # Rate limit
                        self._update_stats('rate_limited_requests')
                        wait_time = 30 + attempt * 10
                        print(f"🕐 VLLM rate limited, waiting {wait_time}s...")
                        time.sleep(wait_time)
                        continue
                        
                    elif response.status_code == 503:
                        # Service unavailable
                        wait_time = 60 + attempt * 20
                        print(f"🕐 VLLM service unavailable, waiting {wait_time}s...")
                        time.sleep(wait_time)
                        continue
                        
                    elif response.status_code == 400:
                        # Bad request - không retry
                        print(f"❌ VLLM bad request (400): {error_message}")
                        raise Exception(f"VLLM bad request: {error_message}")
                        
                    elif response.status_code == 500:
                        # Server error - retry
                        wait_time = 45 + attempt * 15
                        print(f"🕐 VLLM server error, waiting {wait_time}s...")
                        time.sleep(wait_time)
                        continue
                        
                    else:
                        # Other errors
                        print(f"❌ VLLM unexpected error {response.status_code}: {error_message}")
                        if attempt < max_retries - 1:
                            time.sleep(20 + attempt * 10)
                            continue
                        else:
                            raise Exception(f"VLLM API error {response.status_code}: {error_message}")
            
            except requests.exceptions.Timeout:
                # ✅ TIMEOUT HANDLING
                self._update_stats('failed_requests')
                print(f"❌ VLLM API timeout (attempt {attempt + 1})")
                if attempt < max_retries - 1:
                    wait_time = 30 + attempt * 15
                    print(f"🕐 Retrying after timeout in {wait_time}s...")
                    time.sleep(wait_time)
                    continue
                else:
                    raise Exception("VLLM API timeout after all retries")
            
            except requests.exceptions.ConnectionError:
                # ✅ CONNECTION ERROR HANDLING
                self._update_stats('failed_requests')
                print(f"❌ VLLM connection error (attempt {attempt + 1})")
                if attempt < max_retries - 1:
                    wait_time = 60 + attempt * 30
                    print(f"🕐 Retrying after connection error in {wait_time}s...")
                    time.sleep(wait_time)
                    continue
                else:
                    raise Exception("VLLM connection failed after all retries")
            
            except Exception as e:
                # ✅ GENERAL EXCEPTION HANDLING
                self._update_stats('failed_requests')
                print(f"❌ VLLM API unexpected error (attempt {attempt + 1}): {e}")
                
                if attempt < max_retries - 1:
                    wait_time = 30 + attempt * 10
                    print(f"🕐 Retrying after error in {wait_time}s...")
                    time.sleep(wait_time)
                    continue
                else:
                    print(f"❌ VLLM API failed after {max_retries} attempts")
                    raise Exception(f"VLLM API failed: {str(e)}")
        
        # ✅ FALLBACK IF ALL RETRIES FAILED
        self._update_stats('failed_requests')
        raise Exception(f"VLLM API failed after {max_retries} attempts")
    def classify_google_error(self, error):
        """Classify Google AI Studio specific errors"""
        error_str = str(error).lower()
        error_msg = str(error)
        
        # HTTP status codes
        if "429" in error_msg:
            if "requests per minute" in error_str:
                return "rpm_limit"
            elif "requests per day" in error_str:
                return "daily_quota"
            else:
                return "rate_limit"
        elif "503" in error_msg:
            return "service_unavailable"
        elif "400" in error_msg:
            return "invalid_request"
        elif "401" in error_msg:
            return "invalid_key"
        elif "403" in error_msg:
            return "permission_denied"
        
        # Google-specific messages
        if "quota exceeded" in error_str:
            if "requests per minute" in error_str:
                return "rpm_limit"
            elif "requests per day" in error_str:
                return "daily_quota"
            else:
                return "quota_exceeded"
        elif "resource_exhausted" in error_str:
            return "resource_exhausted"
        elif "unavailable" in error_str:
            return "service_unavailable"
        elif "invalid_argument" in error_str:
            return "invalid_request"
        elif "rate limit" in error_str:
            return "rate_limit"
        
        return "unknown"
    
    def call_api_with_retry(self, image_path, prompt, max_retries=3):
        """Unified API call method - routes to appropriate provider"""
        if self.api_provider == "vllm" and self.vllm_enabled:
            return self.call_vllm_api_with_retry(image_path, prompt, max_retries)

    def create_full_document_image(self, image_paths, doc_id, max_pages=None):
        """Ghép tất cả trang thành 1 ảnh dài (full document) với giới hạn số trang"""
        if not image_paths:
            return None
        
        # ✅ Áp dụng giới hạn số trang
        if max_pages and max_pages > 0:
            pages_to_use = image_paths[:max_pages]
            if len(image_paths) > max_pages:
                print(f"⚠️ Giới hạn {max_pages} trang đầu tiên (từ {len(image_paths)} trang)")
        else:
            pages_to_use = image_paths
        
        doc_temp_dir = os.path.dirname(pages_to_use[0])
        full_image_path = os.path.join(doc_temp_dir, "full_document.png")
        
        images = []
        total_height = 0
        max_width = 0
        
        for img_path in pages_to_use:
            img = cv2.imread(img_path)
            if img is not None:
                images.append(img)
                h, w = img.shape[:2]
                total_height += h
                max_width = max(max_width, w)
        
        if not images:
            return None
        
        full_image = np.zeros((total_height, max_width, 3), dtype=np.uint8)
        full_image.fill(255)
        
        current_y = 0
        for img in images:
            h, w = img.shape[:2]
            x_offset = (max_width - w) // 2
            full_image[current_y:current_y + h, x_offset:x_offset + w] = img
            current_y += h
        
        cv2.imwrite(full_image_path, full_image)
        print(f"📄 Đã tạo full document: {full_image_path} ({max_width}x{total_height}) từ {len(pages_to_use)} trang")
        
        # ✅ Estimate file size
        file_size_mb = os.path.getsize(full_image_path) / (1024 * 1024)
        print(f"📊 Kích thước file full document: {file_size_mb:.1f} MB")
        
        return full_image_path


    def pdf_to_images(self, pdf_path, doc_id=None, max_pages=None):
        """Enhanced PDF to images với pdf2image + fallback PyMuPDF"""
        if doc_id is None:
            doc_id = str(uuid.uuid4())[:8]
        
        # ✅ Sử dụng default nếu không có max_pages
        if max_pages is None:
            max_pages = DocumentTypeConfig.DEFAULT_MAX_PAGES_FOR_FULL
        
        doc_temp_dir = os.path.join(self.temp_dir, f"doc_{doc_id}")
        os.makedirs(doc_temp_dir, exist_ok=True)
        
        print(f"📄 Enhanced PDF conversion: {pdf_path}")
        print(f"📁 Temp directory: {doc_temp_dir}")
        print(f"📊 Max pages limit: {max_pages}")
        
        # ✅ TRY pdf2image FIRST (high quality)
        try:
            return self._pdf_to_images_pdf2image(pdf_path, doc_temp_dir, max_pages)
        except Exception as e:
            print(f"⚠️ pdf2image failed: {e}")
            print(f"🔄 Falling back to PyMuPDF...")
            return self._pdf_to_images_pymupdf(pdf_path, doc_temp_dir, max_pages)

    def _pdf_to_images_pdf2image(self, pdf_path, doc_temp_dir, max_pages):
        """Primary method: pdf2image với high quality + transparency fix"""
        try:
            from pdf2image import convert_from_path
            from PIL import Image
            
            print(f"🎯 Using pdf2image (high quality method)")
            
            # ✅ HIGH QUALITY CONVERSION
            images = convert_from_path(
                pdf_path,
                dpi=196,                    # High DPI for quality
                first_page=1,
                last_page=max_pages,
                fmt='PNG',
                thread_count=1,
                transparent=True,           # Preserve transparency
            )
            
            # Get actual total pages from PDF
            doc = fitz.open(pdf_path)
            actual_total_pages = len(doc)
            doc.close()
            
            image_paths = []
            
            for i, image in enumerate(images):
                image_path = os.path.join(doc_temp_dir, f"page_{i + 1}.png")
                
                # ✅ FIX BLACK PDF TRANSPARENCY ISSUE
                if image.mode in ('RGBA', 'LA'):
                    # Create white background for transparent images
                    background = Image.new('RGB', image.size, 'white')
                    background.paste(image, mask=image.split()[-1])
                    background.save(image_path, "PNG")
                    print(f"📄 Page {i + 1}: Fixed transparency → white background")
                else:
                    # No transparency issue
                    image.save(image_path, "PNG")
                    print(f"📄 Page {i + 1}: Direct save (no transparency)")
                
                # Verify file size
                file_size_mb = os.path.getsize(image_path) / (1024 * 1024)
                image_paths.append(image_path)
                print(f"✅ Page {i + 1}: {image.size[0]}x{image.size[1]}, {file_size_mb:.1f}MB")
            
            if actual_total_pages > len(images):
                print(f"⚠️ Đã bỏ qua {actual_total_pages - len(images)} trang cuối")
            
            print(f"✅ pdf2image conversion completed: {len(image_paths)} pages")
            return image_paths, doc_temp_dir, actual_total_pages
            
        except ImportError:
            raise Exception("pdf2image library not installed. Run: pip install pdf2image")
        except Exception as e:
            raise Exception(f"pdf2image conversion failed: {e}")

    def _pdf_to_images_pymupdf(self, pdf_path, doc_temp_dir, max_pages):
        """Fallback method: PyMuPDF với enhanced quality"""
        print(f"🔄 Using PyMuPDF fallback (enhanced quality)")
        
        doc = fitz.open(pdf_path)
        actual_total_pages = len(doc)
        pages_to_convert = min(max_pages, actual_total_pages)
        
        print(f"📑 PDF có {actual_total_pages} trang thực tế")
        print(f"🔧 Sẽ convert {pages_to_convert} trang đầu tiên")
        
        image_paths = []
        
        for page_num in range(pages_to_convert):
            page = doc.load_page(page_num)
            
            # ✅ ENHANCED QUALITY - Match pdf2image DPI (~196)
            zoom_factor = 196 / 72  # Convert to ~196 DPI
            mat = fitz.Matrix(zoom_factor, zoom_factor)
            pix = page.get_pixmap(matrix=mat, alpha=False)  # No alpha for compatibility
            
            image_path = os.path.join(doc_temp_dir, f"page_{page_num + 1}.png")
            pix.save(image_path)
            
            # Log quality info
            file_size_mb = os.path.getsize(image_path) / (1024 * 1024)
            print(f"✅ PyMuPDF Page {page_num + 1}: {pix.width}x{pix.height}, {file_size_mb:.1f}MB")
            
            image_paths.append(image_path)
        
        doc.close()
        
        if actual_total_pages > pages_to_convert:
            print(f"⚠️ Đã bỏ qua {actual_total_pages - pages_to_convert} trang cuối")
        
        print(f"✅ PyMuPDF fallback completed: {len(image_paths)} pages")
        return image_paths, doc_temp_dir, actual_total_pages

    def create_kie_prompt(self):
        """Prompt KIE 1 bước: xác định loại + trích xuất các trường với gợi ý theo loại"""
        return DocumentTypeConfig.build_finetune_kie_prompt()

    def create_summary_prompt(self):
        """Tạo prompt để tóm tắt nội dung khi header không có trích yếu"""
        prompt = """
        Từ ảnh nội dung tài liệu được cung cấp, hãy tóm tắt nội dung chính của tài liệu thành trích yếu ngắn gọn (1-2 câu).
        
        Trả về JSON:
        {
            "summary": "..."
        }
        """
        return prompt

    def check_summary_validity(self, summary_value):
        """Kiểm tra xem trích yếu có hợp lệ không"""
        if not summary_value:
            return False
        
        normalized = summary_value.strip().lower()
        invalid_values = [
            '', 'n/a', 'không có', 'không xác định', 
            'chưa có', 'trống', 'null', 'none',
            'không tìm thấy', 'không rõ'
        ]
        if normalized in invalid_values:
            return False
        if len(normalized) < 10:
            return False
        return True

    def parse_document_number_and_symbol(self, number_and_symbol):
        """
        Parse số và ký hiệu thành document_number và document_symbol
        
        Logic đơn giản:
        - Bỏ từ "Số", "So" ở đầu trước khi parse
        - document_number: từ số đầu tiên đến số cuối cùng, bỏ dấu ".", ":", " "
        - document_symbol: từ chữ đầu tiên đến chữ cuối cùng, bỏ dấu ".", ":", " "
        """
        if not number_and_symbol or not number_and_symbol.strip():
            return "", ""
        
        original = number_and_symbol.strip()
        print(f"🔍 Parsing: '{original}'")
        cleaned = re.sub(r'\s*([/\-_])\s*', r'\1', original)
        # ✅ Bỏ từ "Số", "So" ở đầu (case insensitive)
        if re.match(r'^(số|so)\s*[-:.]?\s*', cleaned, re.IGNORECASE):
            cleaned = re.sub(r'^(số|so)\s*[-:.]?\s*', '', cleaned, flags=re.IGNORECASE).strip()
            print(f"  🧹 Cleaned: '{cleaned}' (removed prefix)")
        
        document_number = ""
        document_symbol = ""
        
        # ✅ Tìm vị trí số đầu tiên và số cuối cùng
        first_digit = None
        last_digit = None
        for i, char in enumerate(cleaned):
            if char.isdigit():
                if first_digit is None:
                    first_digit = i
                last_digit = i
        
        if first_digit is not None and last_digit is not None:
            document_number = cleaned[first_digit:last_digit+1]
            # ✅ Bỏ dấu ".", ":", " "
            document_number = document_number.replace(".", "").replace(":", "").replace(" ", "")
            print(f"  📊 Found number: '{document_number}'")
        
        # ✅ Tìm vị trí chữ đầu tiên và chữ cuối cùng  
        first_letter = None
        last_letter = None
        for i, char in enumerate(cleaned):
            if char.isalpha():
                if first_letter is None:
                    first_letter = i
                last_letter = i
        
        if first_letter is not None and last_letter is not None:
            document_symbol = cleaned[first_letter:last_letter+1]
            # ✅ Bỏ dấu ".", ":", " "
            document_symbol = document_symbol.replace(".", "").replace(":", "").replace(" ", "")
            print(f"  🔤 Found symbol: '{document_symbol}'")
        
        print(f"✅ Final result: number='{document_number}', symbol='{document_symbol}'")
        return document_number, document_symbol

    def extract_values_manually(self, json_text):
        """Manual extraction as fallback when JSON parsing fails - preserving original content"""
        result = {}
        
        # ✅ SMART REGEX PATTERNS - HANDLE ESCAPED QUOTES PROPERLY
        patterns = {
            'document_type': r'"document_type":\s*"([^"]*(?:\\.[^"]*)*)"',
            'document_number_and_symbol': r'"document_number_and_symbol":\s*"([^"]*(?:\\.[^"]*)*)"',
            'handwritten_number': r'"handwritten_number":\s*"([^"]*(?:\\.[^"]*)*)"',
            'issue_date': r'"issue_date":\s*"([^"]*(?:\\.[^"]*)*)"',
            'supervising_authority': r'"supervising_authority":\s*"([^"]*(?:\\.[^"]*)*)"',
            'issuing_authority': r'"issuing_authority":\s*"([^"]*(?:\\.[^"]*)*)"',
            'signer_name': r'"signer_name":\s*"([^"]*(?:\\.[^"]*)*)"',
            'summary': r'"summary":\s*"([^"]*(?:\\.[^"]*)*)"'
        }
        
        import re
        for key, pattern in patterns.items():
            match = re.search(pattern, json_text, re.DOTALL)
            if match:
                value = match.group(1)
                # ✅ ONLY UNESCAPE PROPERLY ESCAPED QUOTES, PRESERVE CONTENT
                value = value.replace('\\"', '"')  # Only unescape \" back to "
                result[key] = value
            else:
                result[key] = ""
        
        print(f"🔧 Manual extraction completed: {len(result)} fields extracted")
        return result

    def fix_json_quotes(self, json_text):
        """Smart JSON quote fixing that preserves content"""
        import re
        
        # ✅ STRATEGY: Properly escape unescaped quotes inside string values
        def fix_string_value(match):
            key = match.group(1)
            value = match.group(2)
            
            # Count quotes to see if they need escaping
            if '"' in value:
                # Escape internal quotes
                fixed_value = value.replace('"', '\\"')
                return f'"{key}": "{fixed_value}"'
            else:
                return match.group(0)  # No change needed
        
        # Pattern to match complete key-value pairs
        pattern = r'"([^"]+)":\s*"([^"]*(?:[^"\\]|\\.|"[^"]*")*[^"]*)"'
        
        try:
            fixed_json = re.sub(pattern, fix_string_value, json_text)
            return fixed_json
        except Exception as e:
            print(f"❌ Error in quote fixing: {e}")
            return json_text
    def normalize_text(self, text):
        """
        Normalize text: bỏ dấu tiếng Việt và chuyển về lowercase
        """
        if not text:
            return ""
        
        # ✅ BỎ DẤU TIẾNG VIỆT
        import unicodedata
        
        # Bỏ dấu bằng NFD normalization
        normalized = unicodedata.normalize('NFD', text)
        # Lọc bỏ các ký tự dấu
        normalized = ''.join(c for c in normalized if unicodedata.category(c) != 'Mn')
        
        # ✅ LOWERCASE
        normalized = normalized.lower()
        
        # ✅ BỎ KHOẢNG TRẮNG (như bạn yêu cầu)
        normalized = normalized.replace(' ', '')
        
        return normalized

    def filter_authority_text(self, text):
        """
        Lọc authority text đơn giản: nếu chứa từ cấm thì coi như rỗng
        """
        if not text or not text.strip():
            return ""
        
        text = text.strip()
        
        # ✅ DANH SÁCH TỪ CẤM
        forbidden_words = [
            "Cộng hòa xã hội chủ nghĩa Việt Nam",
            "Độc lập - Tự do - Hạnh phúc",
            "Mã QHNS",
            "Mã số",
            "www.",
            "http",
            "Email",
            "Tel:",
            "Fax:",
            "Số:",
            "So:",
            "Hà Nội, ngày",
            "TP. Hồ Chí Minh, ngày",
            "0","1","2","3","4","5","6","7","8","9",
        ]
        
        # ✅ NORMALIZE: BỎ DẤU + LOWERCASE + BỎ KHOẢNG TRẮNG
        text_normalized = self.normalize_text(text)
        
        # ✅ NORMALIZE FORBIDDEN WORDS
        for forbidden in forbidden_words:
            forbidden_normalized = self.normalize_text(forbidden)
            
            if forbidden_normalized in text_normalized:
                print(f"🚫 Authority filtered out: '{text}' (chứa '{forbidden}')")
                return ""
        
        # ✅ KHÔNG CÓ TỪ CẤM THÌ TRẢ VỀ NGUYÊN VẸN
        return text
    def analyze_header_kie(self, header_image_path, prompt_override=None):
        """Enhanced header KIE analysis với proper JSON handling"""
        print(f"🧠 Analyzing header KIE with {self.api_provider.upper()}: {os.path.basename(header_image_path)}")
        prompt = prompt_override or self.create_kie_prompt()
        
        try:
            # ✅ Use unified API call
            response_text = self.call_api_with_retry(header_image_path, prompt)
            print(f"📥 Raw response: {response_text[:200]}...")
            
            # ✅ IMPROVED JSON EXTRACTION
            json_text = None
            
            # Method 1: Look for ```json blocks
            if "```json" in response_text:
                json_start = response_text.find("```json") + 7
                json_end = response_text.find("```", json_start)
                if json_end != -1:
                    json_text = response_text[json_start:json_end].strip()
            
            # Method 2: Look for { } blocks with proper brace matching
            if not json_text and "{" in response_text and "}" in response_text:
                json_candidates = []
                start_pos = 0
                
                while True:
                    start = response_text.find("{", start_pos)
                    if start == -1:
                        break
                    
                    # Find matching closing brace
                    brace_count = 0
                    in_string = False
                    escaped = False
                    
                    for i in range(start, len(response_text)):
                        char = response_text[i]
                        
                        if escaped:
                            escaped = False
                            continue
                        
                        if char == '\\':
                            escaped = True
                            continue
                        
                        if char == '"' and not escaped:
                            in_string = not in_string
                            continue
                        
                        if not in_string:
                            if char == '{':
                                brace_count += 1
                            elif char == '}':
                                brace_count -= 1
                                if brace_count == 0:
                                    candidate = response_text[start:i+1]
                                    json_candidates.append(candidate)
                                    break
                    
                    start_pos = start + 1
                
                # Choose the longest valid JSON candidate
                if json_candidates:
                    json_text = max(json_candidates, key=len)
            
            if not json_text:
                raise Exception("No valid JSON structure found in response")
            
            print(f"📋 Extracted JSON: {json_text[:150]}...")
            
            # ✅ SAFE JSON PARSING
            try:
                result = json.loads(json_text)
            except json.JSONDecodeError as je:
                print(f"❌ JSON decode error: {je}")
                print(f"🔧 Attempting smart quote fixing...")
                
                # Try smart quote fixing
                fixed_json = self.fix_json_quotes(json_text)
                
                try:
                    result = json.loads(fixed_json)
                    print("✅ JSON fixed and parsed successfully")
                except json.JSONDecodeError as je2:
                    print(f"❌ Still cannot parse after smart fix: {je2}")
                    print(f"🔧 Using manual extraction...")
                    result = self.extract_values_manually(json_text)
            
            # ✅ PARSE NUMBER AND SYMBOL
            number_and_symbol = result.get('document_number_and_symbol', '')
            document_number, document_symbol = self.parse_document_number_and_symbol(number_and_symbol)
            raw_supervising_authority = result.get('supervising_authority', '')
            raw_issuing_authority = result.get('issuing_authority', '')

            filtered_supervising_authority = self.filter_authority_text(raw_supervising_authority)
            filtered_issuing_authority = self.filter_authority_text(raw_issuing_authority)
            # ✅ Build final result with preserved content
            final_result = {
                'document_type': result.get('document_type', ''),
                'document_number': document_number,
                'document_symbol': document_symbol,
                'document_number_and_symbol': number_and_symbol,
                'handwritten_number': result.get('handwritten_number', ''),
                'issue_date': result.get('issue_date', ''),
                'supervising_authority': filtered_supervising_authority,  # ✅ FILTERED
                'issuing_authority': filtered_supervising_authority + " " + filtered_issuing_authority,         # ✅ FILTERED
                'signer_name': result.get('signer_name', ''),
                'summary': result.get('summary', ''),
            }

            # ✅ Check summary validity
            summary_value = final_result.get('summary', '')
            has_summary = self.check_summary_validity(summary_value)
            final_result['has_summary'] = has_summary
            final_result['api_provider'] = self.api_provider
            
            print(f"✅ Header KIE analysis successful with {self.api_provider.upper()}")
            print(f"📝 Summary found: {has_summary}")
            print(f"📊 Document number: '{document_number}'")
            print(f"🔤 Document symbol: '{document_symbol}'")
            if summary_value:
                print(f"📄 Summary (preserved): {summary_value[:100]}...")
            
            return final_result
            
        except Exception as e:
            print(f"❌ Header KIE analysis failed with {self.api_provider.upper()}: {e}")
            
            # Return error response
            error_response = {
                "document_type": "Khác",
                "document_number": "",
                "document_symbol": "",
                "document_number_and_symbol": "",
                "handwritten_number": "",
                "issue_date": "",
                "supervising_authority": "",
                "issuing_authority": "",
                "signer_name": "",
                "summary": "",
                "has_summary": False,
                "error": str(e),
                "api_provider": self.api_provider,
                "error_source": f"{self.api_provider}_api"
            }
            
            return error_response

    def merge_kie_results(self, base, refined):
        """Gộp kết quả KIE với flat format và parsing số/ký hiệu"""
        if not isinstance(base, dict) or not isinstance(refined, dict):
            return base
        
        # ✅ CẬP NHẬT DANH SÁCH KEYS VỚI TRƯỜNG MỚI
        keys = ["document_type", "document_number_and_symbol", "handwritten_number", 
                "issue_date", "supervising_authority", "issuing_authority", 
                "signer_name", "summary"]  # ✅ THÊM 2 TRƯỜNG MỚI
        
        for k in keys:
            if k in refined and refined[k]:  # ✅ Direct value check
                base[k] = refined[k]
        
        # ✅ Re-parse number and symbol if updated
        if 'document_number_and_symbol' in refined and refined['document_number_and_symbol']:
            document_number, document_symbol = self.parse_document_number_and_symbol(
                refined['document_number_and_symbol']
            )
            base['document_number'] = document_number
            base['document_symbol'] = document_symbol
        
        # Cập nhật has_summary sau merge
        summary_value = base.get("summary", "")
        base["has_summary"] = self.check_summary_validity(summary_value)
        return base

    def summarize_content_with_config(self, content_image_path, document_type):
        """Enhanced content summarization với flat JSON format"""
        has_specific_config = DocumentTypeConfig.has_specific_config(document_type)
        prompt = DocumentTypeConfig.get_summary_prompt(document_type)
        
        config_type = "specific" if has_specific_config else "general"
        print(f"🎯 Summarizing with {config_type} config for '{document_type}' using {self.api_provider.upper()}")
        print(f"📄 Content image: {os.path.basename(content_image_path)}")
        
        try:
            # ✅ Use unified API call
            response_text = self.call_api_with_retry(content_image_path, prompt)
            
            # Parse JSON response
            if "```json" in response_text:
                json_text = response_text.split("```json")[1].split("```")[0].strip()
            elif "{" in response_text and "}" in response_text:
                start = response_text.find("{")
                end = response_text.rfind("}") + 1
                json_text = response_text[start:end]
            else:
                raise Exception("No valid JSON found in response")
            
            result = json.loads(json_text)
            # ✅ FLAT FORMAT - Direct access
            summary = result.get("summary", "Không thể tóm tắt nội dung")
            
            print(f"✅ Content summarization successful with {config_type} config using {self.api_provider.upper()}")
            print(f"📝 Summary: {summary[:100]}...")
            return summary
            
        except Exception as e:
            print(f"❌ Content summarization failed with {self.api_provider.upper()}: {e}")
            print(f"VLLM error details: {e}")
            
            return "Lỗi khi tóm tắt nội dung"
    def get_api_stats(self):
        """Get API usage statistics"""
        stats = self.request_stats.copy()
        if stats['total_requests'] > 0:
            stats['success_rate'] = round(stats['successful_requests'] / stats['total_requests'] * 100, 2)
        else:
            stats['success_rate'] = 0
        return stats
    
    def validate_and_normalize_document_type(self, document_type):
        """Validate và normalize loại tài liệu với flat format"""
        if not document_type or document_type.strip() == '':
            return "Khác"
        
        document_type = document_type.strip()
        
        # Kiểm tra xem có trong danh sách không
        if DocumentTypeConfig.is_valid_document_type(document_type):
            return document_type
        
        # Thử tìm match gần nhất (case-insensitive)
        document_type_lower = document_type.lower()
        for valid_type in DocumentTypeConfig.get_all_document_types():
            if valid_type.lower() == document_type_lower:
                print(f"✅ Normalized document type: '{document_type}' -> '{valid_type}'")
                return valid_type
        
        # Thử tìm match một phần
        for valid_type in DocumentTypeConfig.get_all_document_types():
            if document_type_lower in valid_type.lower() or valid_type.lower() in document_type_lower:
                print(f"✅ Partial match document type: '{document_type}' -> '{valid_type}'")
                return valid_type
        
        # Không tìm thấy, trả về "Khác"
        print(f"⚠️ Loại tài liệu '{document_type}' không có trong danh sách, sử dụng 'Khác'")
        return "Khác"
    def validate_and_enhance_document_type(self, kie_results, yolo_type_hint=None):
        """
        Enhanced document type validation với multi-source priority:
        1. YOLO type hint (header_congvan)
        2. CV symbol priority  
        3. Symbol-based detection
        4. AI detection validation
        """
        detected_type = kie_results.get('document_type', '').strip()
        document_symbol = kie_results.get('document_symbol', '').strip()
        
        print(f"🔍 Document type validation:")
        print(f"   AI detected: '{detected_type}'")
        print(f"   Document symbol: '{document_symbol}'")
        print(f"   YOLO type hint: {yolo_type_hint}")
        
        # ✅ PRIORITY 1: YOLO HEADER_CONGVAN OVERRIDE
        if yolo_type_hint == 'congvan':
            original_type = detected_type
            kie_results['document_type'] = 'Công văn'
            print(f"🎯 PRIORITY 1 - YOLO override: '{original_type}' → 'Công văn' (YOLO detected header_congvan)")
            return kie_results
        
        # ✅ PRIORITY 2: CV SYMBOL FORCE OVERRIDE
        if document_symbol and document_symbol.upper().startswith('CV'):
            if detected_type != "Công văn":
                print(f"🎯 PRIORITY 2 - CV symbol override: '{detected_type}' → 'Công văn' (symbol: '{document_symbol}')")
                kie_results['document_type'] = 'Công văn'
            else:
                print(f"✅ PRIORITY 2 - CV symbol confirmed: Already 'Công văn', symbol confirms: '{document_symbol}'")
            return kie_results
        
        # ✅ PRIORITY 3: CONDITIONAL SYMBOL-BASED FALLBACK
        # Chỉ áp dụng khi AI detect "Khác" hoặc invalid type
        if detected_type == "Khác" or not DocumentTypeConfig.is_valid_document_type(detected_type):
            print(f"🔍 PRIORITY 3 - AI detection '{detected_type}' cần symbol fallback")
            
            if document_symbol:
                symbol_based_type = DocumentTypeConfig.detect_document_type_from_symbol(document_symbol)
                
                if symbol_based_type != "Khác":
                    print(f"🔄 Symbol override: '{detected_type}' → '{symbol_based_type}' (from symbol: '{document_symbol}')")
                    kie_results['document_type'] = symbol_based_type
                    return kie_results
                else:
                    print(f"⚠️ No symbol match found for: '{document_symbol}'")
                    # Fallback to normalization
                    kie_results['document_type'] = self.validate_and_normalize_document_type(detected_type)
                    return kie_results
            else:
                print(f"⚠️ No document symbol for fallback")
                kie_results['document_type'] = self.validate_and_normalize_document_type(detected_type)
                return kie_results
        
        # ✅ PRIORITY 4: AI DETECTION IS VALID - NO CHANGE NEEDED
        else:
            print(f"✅ PRIORITY 4 - AI detection '{detected_type}' is valid, no override needed")
            kie_results['document_type'] = detected_type
            return kie_results
    def detect_header_and_signature_with_yolo(self, image_paths):
        """
        Enhanced YOLO detection với header_congvan priority - BỎ signature fallback
        """
        if self.yolo_model is None:
            print("YOLO model không khả dụng, sử dụng phương pháp fallback")
            return self.detect_sections_fallback_multipage(image_paths)
        
        try:
            best_header = None
            found_signature = None
            header_page_idx = None
            signature_page_idx = None
            best_header_conf = 0
            signature_threshold = 0.2
            header_threshold = 0.3
            header_confidence_threshold = 0.4
            
            # ✅ GIỚI HẠN HEADER CHỈ TRONG 2 TRANG ĐẦU TIÊN
            max_header_pages = min(2, len(image_paths))
            
            print(f"🔍 Enhanced detection trên {len(image_paths)} trang (header: {max_header_pages} trang đầu)")
            
            # ✅ Chạy detection trên từng trang THEO THỨ TỰ
            for page_idx, image_path in enumerate(image_paths):
                print(f"🔍 Detecting page {page_idx + 1}: {os.path.basename(image_path)}")
                results = self.yolo_model(image_path, conf=0.25, iou=0.45)
                
                for result in results:
                    boxes = result.boxes
                    if boxes is not None:
                        for box in boxes:
                            class_id = int(box.cls[0])
                            confidence = float(box.conf[0])
                            x1, y1, x2, y2 = box.xyxy[0].tolist()
                            
                            # ✅ CLASS 3: header_congvan - HIGHEST PRIORITY
                            if (class_id == 3 and 
                                page_idx < max_header_pages and 
                                confidence > 0.35 and 
                                best_header is None):
                                
                                best_header = {
                                    'bbox': [int(x1), int(y1), int(x2), int(y2)],
                                    'confidence': confidence,
                                    'bottom_y': int(y2),
                                    'top_y': int(y1),
                                    'page_index': page_idx,
                                    'image_path': image_path,
                                    'detection_method': 'yolo_header_congvan',
                                    'type_hint': 'congvan'
                                }
                                header_page_idx = page_idx
                                best_header_conf = confidence
                                print(f"🎯 PRIORITY: Header_congvan detected tại trang {page_idx + 1}, confidence: {confidence:.3f}")
                            
                            # ✅ CLASS 0: header - chỉ chọn nếu chưa có header_congvan
                            elif (class_id == 0 and 
                                page_idx < max_header_pages and  
                                confidence > best_header_conf and 
                                confidence > header_threshold and
                                best_header is None):
                                
                                best_header_conf = confidence
                                best_header = {
                                    'bbox': [int(x1), int(y1), int(x2), int(y2)],
                                    'confidence': confidence,
                                    'bottom_y': int(y2),
                                    'top_y': int(y1),
                                    'page_index': page_idx,
                                    'image_path': image_path,
                                    'detection_method': 'yolo_header_standard'
                                }
                                header_page_idx = page_idx
                                print(f"✅ Header standard detected tại trang {page_idx + 1}, conf={confidence:.3f}")
                            
                            # ✅ HEADER NGOÀI GIỚI HẠN - CHỈ LOG
                            elif class_id in [0, 3] and page_idx >= max_header_pages:
                                header_type = "header_congvan" if class_id == 3 else "header"
                                print(f"   📋 {header_type} bỏ qua ở page {page_idx + 1} (ngoài giới hạn), conf={confidence:.3f}")
                            
                            # ✅ SIGNATURE - CHỈ DETECT, KHÔNG FALLBACK
                            elif class_id == 1 and confidence > signature_threshold and found_signature is None:
                                found_signature = {
                                    'bbox': [int(x1), int(y1), int(x2), int(y2)],
                                    'confidence': confidence,
                                    'bottom_y': int(y2),
                                    'top_y': int(y1),
                                    'page_index': page_idx,
                                    'image_path': image_path,
                                    'detection_method': 'yolo'
                                }
                                signature_page_idx = page_idx
                                print(f"   ✍️ Signature found: page {page_idx + 1}, conf={confidence:.3f}")
                    
                    # ✅ EARLY STOP: Nếu đã có header_congvan
                    if best_header and best_header.get('type_hint') == 'congvan':
                        print(f"⚡ Early stop: header_congvan found, continuing for signature only...")
            
            # ✅ HEADER FALLBACK - GIỮ NGUYÊN
            if best_header is None or best_header_conf < header_confidence_threshold:
                if best_header is None:
                    print(f"⚠️ YOLO không detect được header nào trong {max_header_pages} trang đầu")
                else:
                    print(f"⚠️ Header confidence {best_header_conf:.3f} < {header_confidence_threshold}")
                
                print(f"🔄 Fallback: Lấy 1/4 trang đầu tiên làm header")
                first_page = image_paths[0]
                first_img = cv2.imread(first_page)
                
                if first_img is not None:
                    h, w = first_img.shape[:2]
                    fallback_header_bottom = h // 4
                    
                    best_header = {
                        'bbox': [0, 0, w, fallback_header_bottom],
                        'confidence': 0.5,
                        'bottom_y': fallback_header_bottom,
                        'top_y': 0,
                        'page_index': 0,
                        'image_path': first_page,
                        'detection_method': 'fallback_quarter_page'
                    }
                    header_page_idx = 0
                    best_header_conf = 0.5
                    
                    print(f"✅ Fallback header: trang 1, từ y=0 đến y={fallback_header_bottom}")
            
            # ✅ SIGNATURE - KHÔNG FALLBACK
            if found_signature is None:
                print(f"⚠️ Không detect được signature - THÔI, không fallback")
                found_signature = None
                signature_page_idx = None
            
            detection_result = {
                'header': best_header,
                'signature': found_signature,  # ✅ Có thể None
                'header_page_index': header_page_idx,
                'signature_page_index': signature_page_idx,  # ✅ Có thể None
                'method': 'enhanced_yolo_with_congvan_priority_no_signature_fallback',
                'header_search_range': f'first_{max_header_pages}_pages',
                'signature_search_range': f'all_{len(image_paths)}_pages_no_fallback'
            }
            
            # ✅ LOG KẾT QUẢ CUỐI CÙNG
            if best_header:
                method = best_header.get('detection_method', 'unknown')
                type_hint = best_header.get('type_hint', 'none')
                print(f"✅ FINAL Header: page {header_page_idx + 1}, confidence={best_header['confidence']:.3f}, method={method}, type_hint={type_hint}")
            
            if found_signature:
                method = found_signature.get('detection_method', 'unknown')
                print(f"✅ FINAL Signature: page {signature_page_idx + 1}, confidence={found_signature['confidence']:.3f}, method={method}")
            else:
                print(f"❌ FINAL Signature: None detected - no fallback")
            
            return detection_result
                
        except Exception as e:
            print(f"❌ Enhanced YOLO detection error: {e}")
            return self.detect_sections_fallback_multipage(image_paths)
    def detect_sections_fallback_multipage(self, image_paths):
        """Fallback detection cho multi-page - CHỈ header dùng 1/4 trang đầu"""
        print(f"🔄 Using complete fallback for {len(image_paths)} pages")
        
        first_page = image_paths[0]
        first_img = cv2.imread(first_page)
        
        if first_img is None:
            return None
        
        h1, w1 = first_img.shape[:2]
        
        # ✅ CHỈ HEADER DÙNG 1/4 TRANG ĐẦU
        header_bottom = h1 // 4
        
        print(f"📐 Fallback detection:")
        print(f"   📋 Header: trang 1, từ 0 đến {header_bottom} (1/4 trang = {header_bottom/h1*100:.1f}%)")
        print(f"   ✍️ Signature: Không có - bỏ qua")
        
        return {
            'header': {
                'bbox': [0, 0, w1, header_bottom],
                'confidence': 0.5,
                'bottom_y': header_bottom,
                'top_y': 0,
                'page_index': 0,
                'image_path': first_page,
                'detection_method': 'fallback_quarter_page'
            },
            'signature': None,  # ✅ KHÔNG CÓ SIGNATURE
            'header_page_index': 0,
            'signature_page_index': None,  # ✅ KHÔNG CÓ
            'method': 'complete_fallback'
        }
    
    def build_content_from_header_page_to_max_pages(self, image_paths, detection_result, max_pages_for_full):
        """
        ✅ LOGIC MỚI: Lấy content từ cuối header (trang có header) đến trang max_pages_for_full
        
        Args:
            image_paths: Danh sách tất cả trang
            detection_result: Kết quả detect header/signature  
            max_pages_for_full: Số trang tối đa theo config
        
        Returns:
            str: Đường dẫn ảnh content đã ghép
        """
        header_info = detection_result.get('header')
        if not header_info:
            print("❌ Không có thông tin header để tạo content")
            return None
        
        # ✅ Trang có header và vị trí cuối header
        header_page_index = header_info['page_index']
        header_bottom_y = header_info['bottom_y']
        header_page_path = header_info['image_path']
        
        print(f"📍 Header ở trang {header_page_index + 1}, cuối header tại y={header_bottom_y}")
        
        # ✅ Xác định range trang cho content: từ trang có header đến max_pages_for_full
        total_available_pages = len(image_paths)
        end_page_index = min(max_pages_for_full - 1, total_available_pages - 1)  # 0-indexed
        
        print(f"📄 Content range: từ trang {header_page_index + 1} đến trang {end_page_index + 1}")
        print(f"📊 Total available pages: {total_available_pages}, max_pages_for_full: {max_pages_for_full}")
        
        if header_page_index > end_page_index:
            print(f"⚠️ Header ở trang {header_page_index + 1} > end_page {end_page_index + 1}, không có content")
            return None
        
        content_images = []
        
        # ✅ TRANG ĐẦU TIÊN: Lấy phần dưới header
        header_img = cv2.imread(header_page_path)
        if header_img is not None:
            h, w = header_img.shape[:2]
            
            # Đảm bảo header_bottom_y hợp lệ
            if header_bottom_y < h - 20:  # Còn ít nhất 20px content
                first_page_content = header_img[header_bottom_y:h, 0:w]
                if first_page_content.shape[0] > 0:
                    content_images.append(first_page_content)
                    print(f"  ✅ Trang {header_page_index + 1}: lấy từ y={header_bottom_y} đến y={h} ({first_page_content.shape[0]}px)")
                else:
                    print(f"  ⚠️ Trang {header_page_index + 1}: không có content sau header")
            else:
                print(f"  ⚠️ Trang {header_page_index + 1}: header quá dài, không có content")
        
        # ✅ CÁC TRANG TIẾP THEO: Lấy nguyên trang
        for page_idx in range(header_page_index + 1, end_page_index + 1):
            if page_idx < len(image_paths):
                page_path = image_paths[page_idx]
                page_img = cv2.imread(page_path)
                if page_img is not None:
                    content_images.append(page_img)
                    print(f"  ✅ Trang {page_idx + 1}: lấy toàn bộ trang ({page_img.shape[0]}px)")
        
        if not content_images:
            print("❌ Không có content image nào để ghép")
            return None
        
        # ✅ GHÉP TẤT CẢ CONTENT IMAGES
        max_width = max(img.shape[1] for img in content_images)
        total_height = sum(img.shape[0] for img in content_images)
        
        combined_content = np.full((total_height, max_width, 3), 255, dtype=np.uint8)
        
        current_y = 0
        for i, img in enumerate(content_images):
            h, w = img.shape[:2]
            x_offset = (max_width - w) // 2
            combined_content[current_y:current_y + h, x_offset:x_offset + w] = img
            current_y += h
            print(f"    🔧 Ghép content {i+1}: y={current_y-h} đến y={current_y}")
        
        # ✅ LƯU FILE
        output_dir = os.path.dirname(image_paths[0])
        pages_range = f"page{header_page_index + 1}_to_page{end_page_index + 1}"
        content_path = os.path.join(output_dir, f"content_from_header_{pages_range}.png")
        
        success = cv2.imwrite(content_path, combined_content)
        if not success:
            print(f"❌ Lỗi lưu content: {content_path}")
            return None
        
        print(f"✅ Đã tạo content từ header: {content_path}")
        print(f"📏 Kích thước: {max_width}x{total_height}")
        print(f"📄 Từ {len(content_images)} phần (trang {header_page_index + 1} đến {end_page_index + 1})")
        
        return content_path
    def analyze_signature_date(self, signature_image_path):
        """Phân tích ngày tháng từ signature area"""
        print(f"📅 Analyzing signature date from: {os.path.basename(signature_image_path)}")
        
        prompt = DocumentTypeConfig.create_signature_prompt()
        
        try:
            # ✅ Use unified API call
            response_text = self.call_api_with_retry(signature_image_path, prompt)
            print(f"📥 Signature response: {response_text[:200]}...")
            
            # ✅ JSON extraction (same logic as header)
            json_text = None
            
            if "```json" in response_text:
                json_start = response_text.find("```json") + 7
                json_end = response_text.find("```", json_start)
                if json_end != -1:
                    json_text = response_text[json_start:json_end].strip()
            
            if not json_text and "{" in response_text and "}" in response_text:
                start = response_text.find("{")
                end = response_text.rfind("}") + 1
                json_text = response_text[start:end]
            
            if not json_text:
                raise Exception("No valid JSON structure found in signature response")
            
            # ✅ Parse JSON
            try:
                result = json.loads(json_text)
            except json.JSONDecodeError as je:
                print(f"❌ Signature JSON decode error: {je}")
                result = self.extract_signature_manually(json_text)
            
            signature_date = result.get('signature_date', '')
            
            final_result = {
                'signature_date': signature_date,
                'source': 'signature_area',
                'confidence': 'high' if signature_date else 'none',
                'api_provider': self.api_provider
            }
            
            if signature_date:
                print(f"✅ Signature date extracted: {signature_date}")
            else:
                print(f"⚠️ No signature date found")
            
            return final_result
            
        except Exception as e:
            print(f"❌ Signature analysis failed with {self.api_provider.upper()}: {e}")
            
            return {
                'signature_date': '',
                'source': 'signature_area',
                'confidence': 'none',
                'error': str(e),
                'api_provider': self.api_provider
            }

    def extract_signature_manually(self, json_text):
        """Manual extraction for signature analysis"""
        import re
        
        pattern = r'"signature_date":\s*"([^"]*(?:\\.[^"]*)*)"'
        match = re.search(pattern, json_text, re.DOTALL)
        
        if match:
            value = match.group(1).replace('\\"', '"')
            return {'signature_date': value}
        else:
            return {'signature_date': ''}

    def merge_with_signature_priority(self, header_results, signature_results):
        """Merge results với signature date priority"""
        final_results = header_results.copy()
        
        if signature_results and signature_results.get('signature_date'):
            signature_date = signature_results['signature_date'].strip()
            if signature_date:
                print(f"🎯 Priority merge: signature_date '{signature_date}' overrides header issue_date '{header_results.get('issue_date', '')}'")
                final_results['issue_date'] = signature_date
                final_results['date_source'] = 'signature'
            else:
                final_results['date_source'] = 'header' if header_results.get('issue_date') else 'none'
        else:
            final_results['date_source'] = 'header' if header_results.get('issue_date') else 'none'
        
        return final_results

    def crop_header_only(self, image_paths, detection_result):
        """Crop header với mở rộng xuống dưới thêm - CONFIGURABLE"""
        header_info = detection_result.get('header')
        if not header_info:
            return None
        
        header_page_path = header_info['image_path']
        header_img = cv2.imread(header_page_path)
        if header_img is None:
            return None
        
        h, w = header_img.shape[:2]
        header_bottom = header_info['bottom_y']
        
        # Validation
        if header_bottom <= 0:
            header_bottom = h // 4
        elif header_bottom >= h:
            header_bottom = h - 10
        
        # ✅ LOAD CONFIG TỪ .ENV
        extension_ratio = float(os.getenv('HEADER_EXTENSION_RATIO', '0.7'))  # Default 0.7 = 70%
        min_extension = int(os.getenv('HEADER_EXTENSION_MIN_PIXELS', '30'))  # Default 30px
        max_extension = int(os.getenv('HEADER_EXTENSION_MAX_PIXELS', '100')) # Default 100px
        
        # ✅ TÍNH TOÁN EXTENSION DỰA TRÊN CONFIG
        extension_pixels = int(h * extension_ratio)
        
        # Áp dụng giới hạn min/max
        actual_extension = max(min_extension, min(extension_pixels, max_extension))
        extended_bottom = header_bottom + actual_extension
        
        # Đảm bảo không vượt quá chiều cao ảnh
        if extended_bottom > h:
            extended_bottom = h
            actual_extension = extended_bottom - header_bottom
        
        print(f"📋 Header extension (configurable):")
        print(f"   Config: ratio={extension_ratio}, min={min_extension}px, max={max_extension}px")
        print(f"   Page height: {h}px")
        print(f"   Original bottom: {header_bottom}px")
        print(f"   Extension: {actual_extension}px ({actual_extension/h*100:.1f}%)")
        print(f"   Final bottom: {extended_bottom}px")
        
        # ✅ CROP VỚI VÙNG MỞ RỘNG
        header_section = header_img[0:extended_bottom, 0:w]
        
        if header_section.shape[0] > 0:
            # Save extended header
            output_dir = os.path.dirname(image_paths[0])
            header_only_path = os.path.join(output_dir, "header_only_for_analysis.png")
            
            success = cv2.imwrite(header_only_path, header_section)
            if success:
                detection_method = header_info.get('detection_method', 'unknown')
                print(f"📋 Extended header cropped: {header_section.shape[1]}x{header_section.shape[0]} (method: {detection_method})")
                return header_only_path
        
        return None

    def crop_signature_only(self, image_paths, detection_result):
        """Crop chỉ signature để phân tích riêng biệt"""
        signature_info = detection_result.get('signature')
        if not signature_info:
            return None
        
        signature_page_path = signature_info['image_path']
        signature_img = cv2.imread(signature_page_path)
        if signature_img is None:
            return None
        
        h, w = signature_img.shape[:2]
        signature_top = signature_info['top_y']
        
        # Validation
        if signature_top <= 0:
            signature_top = int(h * 0.8)
        elif signature_top >= h:
            signature_top = h - 50
        
        signature_section = signature_img[signature_top:h, 0:w]
        
        if signature_section.shape[0] > 0:
            # Save signature only
            output_dir = os.path.dirname(image_paths[0])
            signature_only_path = os.path.join(output_dir, "signature_only_for_analysis.png")
            
            success = cv2.imwrite(signature_only_path, signature_section)
            if success:
                detection_method = signature_info.get('detection_method', 'unknown')
                print(f"✍️ Signature only cropped: {signature_section.shape[1]}x{signature_section.shape[0]} (method: {detection_method})")
                return signature_only_path
        
        return None
    def combine_cropped_images_for_storage(self, header_only_path, signature_only_path=None):
        """Ghép các ảnh đã crop sẵn để lưu trữ - KHÔNG crop lại"""
        if not header_only_path or not os.path.exists(header_only_path):
            print("❌ Không có header image để combine")
            return None
        
        # ✅ LOAD HEADER IMAGE
        header_img = cv2.imread(header_only_path)
        if header_img is None:
            print("❌ Không thể load header image")
            return None
        
        sections = [header_img]
        section_names = ["header"]
        
        # ✅ LOAD SIGNATURE IMAGE (nếu có và cần thiết)
        if signature_only_path and os.path.exists(signature_only_path):
            signature_img = cv2.imread(signature_only_path)
            if signature_img is not None:
                sections.append(signature_img)
                section_names.append("signature")
                print(f"✅ Loaded signature for combining: {signature_img.shape[1]}x{signature_img.shape[0]}")
            else:
                print("⚠️ Signature path exists but cannot load image")
        else:
            print("📝 Không có signature để combine - chỉ dùng header")
        
        output_dir = os.path.dirname(header_only_path)
        
        # ✅ NẾU CHỈ CÓ HEADER - COPY VÀ RENAME
        if len(sections) == 1:
            result_path = os.path.join(output_dir, "header_for_storage.png")
            success = cv2.imwrite(result_path, header_img)
            
            if success:
                print(f"✅ Header-only saved for storage: {result_path}")
                return result_path
            else:
                print(f"❌ Failed to save header-only")
                return None
        
        # ✅ NẾU CÓ CẢ HEADER + SIGNATURE - GHÉP
        else:
            max_width = max(sec.shape[1] for sec in sections)
            spacing = 20
            total_height = sum(sec.shape[0] for sec in sections) + spacing * (len(sections) - 1)
            
            combined_image = np.full((total_height, max_width, 3), 255, dtype=np.uint8)
            
            current_y = 0
            for i, section in enumerate(sections):
                h, w = section.shape[:2]
                x_offset = (max_width - w) // 2
                combined_image[current_y:current_y + h, x_offset:x_offset + w] = section
                current_y += h + spacing
                print(f"✅ Combined {section_names[i]}: y={current_y-h-spacing} to y={current_y-spacing}")
            
            combined_path = os.path.join(output_dir, "header_signature_combined.png")
            success = cv2.imwrite(combined_path, combined_image)
            
            if success:
                print(f"✅ Header+Signature combined for storage: {combined_path}")
                print(f"📏 Combined size: {max_width}x{total_height}")
                return combined_path
            else:
                print(f"❌ Failed to save combined image")
                return None

    def process_document(self, file_path, doc_id=None):
        """Xử lý tài liệu với NEW FLOW: Header riêng → Enhanced validation → Check signature → Signature analysis → Merge → Combine images"""
        if not os.path.exists(file_path):
            raise FileNotFoundError(f"Không tìm thấy file: {file_path}")
        
        if doc_id is None:
            doc_id = str(uuid.uuid4())[:8]
            
        file_ext = os.path.splitext(file_path)[1].lower()
        print(f"🚀 Đang xử lý file với ENHANCED SIGNATURE FLOW: {file_path}")
        print(f"📂 Document ID: {doc_id}")
        
        if file_ext == '.pdf':
            
            # ✅ Bước 1: Tách PDF thành ảnh với giới hạn default
            max_pages_default = DocumentTypeConfig.DEFAULT_MAX_PAGES_FOR_FULL
            image_paths, doc_temp_dir, actual_total_pages = self.pdf_to_images(file_path, doc_id, max_pages_default)
            if not image_paths:
                raise Exception("Không thể tách PDF thành ảnh")
            
            # ✅ Bước 2: Enhanced YOLO Detection với header_congvan priority
            detection_result = self.detect_header_and_signature_with_yolo(image_paths)
            if not detection_result:
                raise Exception("Không thể detect header và signature")
            
            # 3.1: Crop header
            header_only_path = self.crop_header_only(image_paths, detection_result)
            if not header_only_path:
                raise Exception("Không thể crop header để phân tích")

            # 3.2: Crop signature (nếu có)
            signature_info = detection_result.get('signature')
            signature_only_path = None

            if signature_info:
                signature_only_path = self.crop_signature_only(image_paths, detection_result)
                if signature_only_path:
                    pass
                print("⚠️ Không thể crop signature")
            else:
                print("⚠️ Không detect được signature")

            # 3.3: Ghép header + signature (DÙNG HÀM CÓ SẴN)
            combined_for_analysis_path = self.combine_cropped_images_for_storage(
                header_only_path, signature_only_path
            )

            if not combined_for_analysis_path:
                print("⚠️ Không thể ghép, dùng header-only")
                combined_for_analysis_path = header_only_path

            # 3.4: Gửi COMBINED IMAGE cho AI để phân tích
            kie_header_results = self.analyze_header_kie(combined_for_analysis_path)
            print(f"📋 Raw Header+Signature KIE Results (bao gồm signer_name): {kie_header_results}")
            
            print("🧠 STEP 4: Enhanced document type validation...")

            # Get YOLO type hint
            yolo_type_hint = None
            header_info = detection_result.get('header')
            if header_info:
                yolo_type_hint = header_info.get('type_hint')

            # Apply enhanced validation với priority system
            final_kie_results = self.validate_and_enhance_document_type(
                kie_header_results, 
                yolo_type_hint=yolo_type_hint
            )
            document_type = final_kie_results.get('document_type', '')
            # ✅ DÙNG final_kie_results TRỰC TIẾP - KHÔNG CẦN MERGE
            merged_kie_results = final_kie_results  # ✅ ĐỔI TÊN CHO NHẤT QUÁN VỚI CODE CŨ
            
            # ✅ Bước 8: Tạo full document image
            full_image_path = None # self.create_full_document_image(image_paths, doc_id, max_pages_default)
            # if not full_image_path:
            #     raise Exception("Không thể tạo full document image")
            
            # ✅ Bước 10: Lấy config và check content analysis requirement
            config = DocumentTypeConfig.get_config_summary(document_type)
            max_pages_for_full = config['max_pages_for_full']
            has_specific_config = config['has_specific_config']
            
            # ✅ Bước 11: Tái tạo images nếu config khác default
            final_image_paths = image_paths
            if max_pages_for_full != max_pages_default:
                print(f"🔄 STEP 11: Config yêu cầu {max_pages_for_full} trang, default là {max_pages_default}")
                
                if max_pages_for_full > max_pages_default:
                    print(f"📄 Cần convert thêm trang từ PDF...")
                    additional_image_paths, _, _ = self.pdf_to_images(file_path, doc_id, max_pages_for_full)
                    final_image_paths = additional_image_paths
                    
                    # ✅ Re-detect trên image paths mới nếu cần
                    print(f"🔍 Re-detecting trên {len(final_image_paths)} trang mới...")
                    updated_detection_result = self.detect_header_and_signature_with_yolo(final_image_paths)
                    if updated_detection_result:
                        detection_result = updated_detection_result
                        print("✅ Updated detection results với pages mở rộng")
                else:
                    final_image_paths = image_paths[:max_pages_for_full]
                    print(f"✂️ Cắt xuống còn {len(final_image_paths)} trang")
                
                # # Remove old full image và tạo lại
                # if os.path.exists(full_image_path):
                #     os.remove(full_image_path)
                # full_image_path = self.create_full_document_image(final_image_paths, doc_id, max_pages_for_full)
            
            # ✅ Bước 12: Content analysis (theo config)
            should_analyze_content = has_specific_config
            print(f"📝 STEP 12: Content analysis decision: {should_analyze_content}")

            combined_content_path = None
            kie_content_results = {}
            
            if should_analyze_content:
                print(f"🎯 Loại đặc biệt '{document_type}' có trong DOCUMENT_CONFIGS - xử lý content")
                
                combined_content_path = self.build_content_from_header_page_to_max_pages(
                    final_image_paths, detection_result, max_pages_for_full
                )
                
                if combined_content_path:
                    summary_text = self.summarize_content_with_config(combined_content_path, document_type)
                    
                    header_page_index = detection_result.get('header', {}).get('page_index', 0)
                    end_page_index = min(max_pages_for_full - 1, len(final_image_paths) - 1)
                    actual_pages_analyzed = max(0, end_page_index - header_page_index + 1)
                    
                    kie_content_results = {
                        'summary': summary_text,
                        'source': f'content_analysis_from_header_page_to_max_pages',
                        'document_type_used': document_type,
                        'config_type': "specific",
                        'pages_analyzed': actual_pages_analyzed,
                        'header_page_index': header_page_index,
                        'content_start_page': header_page_index + 1,
                        'content_end_page': end_page_index + 1,
                        'content_image_path': combined_content_path,
                        'processing_method': f'ai_summarization_with_specific_config_only',
                        'has_specific_config': True
                    }
                    
                    # ✅ PRIORITY LOGIC - XÓA HEADER SUMMARY khi có content summary
                    print(f"🎯 Loại đặc biệt - ưu tiên content summary, xóa header summary")
                    merged_kie_results['summary'] = ''
                    merged_kie_results['has_summary'] = False
                    
                else:
                    kie_content_results = {
                        'summary': '',
                        'source': 'content_creation_failed',
                        'processing_method': 'error',
                        'has_specific_config': True,
                        'error': 'Cannot create content image'
                    }
            else:
                print(f"✅ Loại '{document_type}' không có trong DOCUMENT_CONFIGS - bỏ qua content analysis")
                
                kie_content_results = {
                    'summary': '',
                    'source': 'not_in_document_configs',
                    'processing_method': 'skipped_no_specific_config',
                    'document_type_checked': document_type,
                    'has_specific_config': has_specific_config
                }
            
            print("🧹 STEP 13: Cleanup temporary files...")

            # Clean up individual page files
            for page_path in final_image_paths:
                if os.path.exists(page_path):
                    try:
                        os.remove(page_path)
                    except Exception as e:
                        print(f"⚠️ Cannot remove {page_path}: {e}")

            # ✅ Clean up analysis-only images (header_only, signature_only)
            if 'header_only_path' in locals() and header_only_path and os.path.exists(header_only_path):
                try:
                    os.remove(header_only_path)
                except Exception as e:
                    print(f"⚠️ Cannot remove header_only: {e}")

            if 'signature_only_path' in locals() and signature_only_path and os.path.exists(signature_only_path):
                try:
                    os.remove(signature_only_path)
                except Exception as e:
                    print(f"⚠️ Cannot remove signature_only: {e}")
            # print("🧹 DELETE temp_processing folder immediately after processing...")
            # if doc_temp_dir and os.path.exists(doc_temp_dir):
            #     try:
            #         import shutil
            #         shutil.rmtree(doc_temp_dir)
            #         print(f"✅ Deleted entire temp_processing folder: {doc_temp_dir}")
            #     except Exception as e:
            #         print(f"⚠️ Failed to delete temp_processing folder: {e}")
            
            # ✅ Bước 14: Build final result
            print("📦 STEP 14: Building final result...")
            
            result = {
                "file_path": file_path,
                "kie_header_data": merged_kie_results,  # ✅ ĐÃ CÓ SIGNER_NAME
                "kie_content_data": kie_content_results,
                "document_type_validation": {
                    "ai_detected": kie_header_results.get('document_type', ''),
                    "yolo_type_hint": yolo_type_hint,
                    "final_type": document_type,
                    "validation_method": "enhanced_multi_source_priority"
                },
                "document_type_detected": document_type,
                "signer_name": final_kie_results.get('signer_name', ''),  # ✅ THÊM VÀO TOP LEVEL
                "config_used": config,
                "has_specific_config": has_specific_config,
                "max_pages_for_full": max_pages_for_full,
                "total_pages": actual_total_pages,
                "pages_used_for_full": len(final_image_paths),
                "image_paths": {
                    "full": full_image_path,
                    "header": combined_for_analysis_path,  # ✅ ĐÂY LÀ HEADER+SIGNATURE
                    "content": combined_content_path if should_analyze_content else None
                },
                "header_detection": detection_result,
                "document_type": "PDF",
                "temp_dir": doc_temp_dir,
                "flow_version": "enhanced_header_signature_combined_before_analysis",
                "api_provider": self.api_provider,
                "processing_stats": {
                    "total_steps": 13,  # ✅ GIẢM XUỐNG VÌ BỎ STEP 5,6,7
                    "enhanced_validation": True,
                    "signature_combined_before_analysis": True,
                    "content_analysis": should_analyze_content,
                    "api_stats": self.get_api_stats() if hasattr(self, 'get_api_stats') else None
                }
            }
                        
            return result
                            
        else:
            raise ValueError(f"Định dạng file không được hỗ trợ: {file_ext}")