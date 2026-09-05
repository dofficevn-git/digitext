# ✅ CẬP NHẬT external_api.py - AUTO RETRY KHI 401

import os
import requests
import json
from datetime import datetime
from typing import List, Dict, Optional

class ExternalAPIManager:
    """Manager for external project server API"""
    
    def __init__(self):
        self.base_url = os.getenv('EXTERNAL_SERVER_URL', 'http://103.154.62.17:8077')
        self.username = os.getenv('EXTERNAL_SERVER_USERNAME', 'thangnb')
        self.password = os.getenv('EXTERNAL_SERVER_PASSWORD', '')
        self.timeout = int(os.getenv('EXTERNAL_SERVER_TIMEOUT', '30'))
        self.token = None
        
        print(f"🔌 External API Manager initialized: {self.base_url}")
    
    def login(self) -> bool:
        """Login to external server and get token"""
        try:
            url = f"{self.base_url}/api/Account/LogIn"
            
            payload = {
                "userName": self.username,
                "password": self.password,
                "rememberMe": True
            }
            
            print(f"🔐 Logging in to external server: {url}")
            
            response = requests.post(
                url, 
                json=payload, 
                headers={'Content-Type': 'application/json'},
                timeout=self.timeout
            )
            
            if response.status_code == 200:
                data = response.json()
                self.token = data.get('token')
                
                if self.token:
                    print(f"✅ External login successful - Token: {self.token[:20]}...")
                    return True
                else:
                    print(f"❌ No token in response: {data}")
                    return False
            else:
                print(f"❌ External login failed: {response.status_code} - {response.text}")
                return False
                
        except Exception as e:
            print(f"❌ External login error: {e}")
            return False

    # ✅ THÊM METHOD _make_authenticated_request VỚI AUTO RETRY
    def _make_authenticated_request(self, method, url, **kwargs):
        """Make authenticated request với auto retry khi 401"""
        
        # ✅ ENSURE WE HAVE TOKEN
        if not self.token:
            print("🔐 No token found, logging in first...")
            if not self.login():
                raise Exception("Failed to authenticate with external server")
        
        # ✅ PREPARE HEADERS
        headers = kwargs.get('headers', {})
        headers.update({
            'Authorization': f'Bearer {self.token}',
            'Content-Type': 'application/json'
        })
        kwargs['headers'] = headers
        kwargs['timeout'] = self.timeout
        
        # ✅ FIRST ATTEMPT
        print(f"🌐 Making {method.upper()} request to: {url}")
        
        try:
            response = requests.request(method, url, **kwargs)
            
            # ✅ CHECK FOR 401 - TOKEN EXPIRED
            if response.status_code == 401:
                print("🔄 Got 401, token expired. Logging in again and retrying...")
                
                # ✅ CLEAR OLD TOKEN AND LOGIN AGAIN
                self.token = None
                if not self.login():
                    raise Exception("Re-authentication failed after 401")
                
                # ✅ UPDATE HEADERS WITH NEW TOKEN
                headers['Authorization'] = f'Bearer {self.token}'
                kwargs['headers'] = headers
                
                # ✅ RETRY REQUEST WITH NEW TOKEN
                print("🔄 Retrying request with new token...")
                response = requests.request(method, url, **kwargs)
                
                if response.status_code == 401:
                    raise Exception("Still getting 401 after re-authentication")
                    
            return response
            
        except requests.exceptions.RequestException as e:
            print(f"❌ Network error during request: {e}")
            raise
    
    def ensure_authenticated(self) -> bool:
        """Ensure we have valid authentication (legacy compatibility)"""
        if not self.token:
            return self.login()
        return True
    
    def get_projects(self) -> List[Dict]:
        """Get list of projects from external server với auto retry"""
        try:
            url = f"{self.base_url}/api/Duan/DanhSachDuAn"
            
            response = self._make_authenticated_request('GET', url)
            
            if response.status_code == 200:
                projects = response.json()
                print(f"✅ Retrieved {len(projects)} projects")
                return projects
            else:
                print(f"❌ Get projects failed: {response.status_code} - {response.text}")
                return []
                
        except Exception as e:
            print(f"❌ Get projects error: {e}")
            return []
    
    def get_project_files(self, project_id: str) -> List[Dict]:
        """Get files for a specific project với auto retry"""
        try:
            url = f"{self.base_url}/api/VanBan/ThongTinFileTheoDuAn"
            
            params = {'idDuAn': project_id}
            
            print(f"📁 Getting files for project {project_id} from VanBan API")
            
            response = self._make_authenticated_request('GET', url, params=params)
            
            if response.status_code == 200:
                data = response.json()
                
                # ✅ HANDLE RESPONSE STRUCTURE
                if data.get('success'):
                    files = data.get('data', [])
                else:
                    print(f"❌ API response success=false: {data}")
                    return []
                
                print(f"✅ Retrieved {len(files)} files for project {project_id}")
                
                # ✅ DEBUG FIRST FILE IF EXISTS
                if files:
                    first_file = files[0]
                    print(f"🔍 First file: {first_file.get('tenFile')} -> idVanBan: {first_file.get('idVanBan')}")
                
                return files
            else:
                print(f"❌ Get project files failed: {response.status_code}")
                print(f"   Response: {response.text}")
                return []
                
        except Exception as e:
            print(f"❌ Get project files error: {e}")
            import traceback
            traceback.print_exc()
            return []


    def update_document(self, van_ban_id: str, document_data: Dict) -> bool:
        """Update document với auto retry"""
        try:
            url = f"{self.base_url}/api/VanBan/CapNhatVanBan"
            
            # ✅ SMART SUMMARY EXTRACTION - ƯU TIÊN HEADER → CONTENT
            summary_value = ""
            
            # ✅ 1. ƯU TIÊN HEADER SUMMARY TRƯỚC
            header_summary = document_data.get('summary', '').strip()
            if header_summary and header_summary not in ['', 'N/A', 'Không có', 'null']:
                summary_value = header_summary
                print(f"📝 Using HEADER summary: {summary_value[:50]}...")
            else:
                # ✅ 2. NẾU HEADER TRỐNG THÌ LẤY CONTENT
                content_summary = ""
                
                # Check if document_data has nested structure
                if 'kie_content_data' in document_data:
                    content_data = document_data.get('kie_content_data', {})
                    content_summary = content_data.get('summary', '').strip()
                elif 'content_summary' in document_data:
                    content_summary = document_data.get('content_summary', '').strip()
                
                if content_summary and content_summary not in ['', 'N/A', 'Không có', 'null']:
                    summary_value = content_summary
                    print(f"📝 Using CONTENT summary (fallback): {summary_value[:50]}...")
                else:
                    print(f"⚠️ No valid summary found in header or content")
            
            # ✅ MAP KIE DATA SANG EXTERNAL FORMAT
            payload = {
                "vanBanId": int(van_ban_id) if van_ban_id and van_ban_id.isdigit() else 0,
                "vbSoVanBan": document_data.get('document_number', ''),
                "vbSoKyHieu": document_data.get('document_symbol', ''),  
                "vbNgayThang": document_data.get('issue_date', ''),
                "vbLoaiVb": document_data.get('document_type', ''),
                "vbCoQuanBanHanh": document_data.get('issuing_authority', ''),
                "vbTrichyeu": summary_value,  # ✅ SỬ DỤNG SMART SUMMARY
                "vbToSo": document_data.get('handwritten_number', '')
            }

            print(f"📤 Updating external document vanBanId={van_ban_id}")
            print(f"📝 Final summary length: {len(summary_value)} chars")

            response = self._make_authenticated_request('POST', url, json=payload)
            
            if response.status_code == 200:
                result = response.json()
                if result.get('success'):
                    print(f"✅ External document {van_ban_id} updated successfully")
                    return True
                else:
                    print(f"❌ External update failed: {result.get('message', 'Unknown error')}")
                    return False
            else:
                print(f"❌ External update HTTP error: {response.status_code} - {response.text}")
                return False
                
        except Exception as e:
            print(f"❌ External update error: {e}")
            return False

    def update_documents_batch(self, documents_data: List[Dict]) -> Dict:
        """Update multiple documents với auto retry và smart summary extraction"""
        try:
            url = f"{self.base_url}/api/VanBan/CapNhatDanhSachVanBan"
            
            # ✅ MAP MULTIPLE DOCUMENTS VỚI SMART SUMMARY
            payload = []
            for doc_data in documents_data:
                van_ban_id = doc_data.get('van_ban_id', '')
                kie_data = doc_data.get('kie_data', {})
                
                # ✅ SMART SUMMARY EXTRACTION CHO TỪNG DOCUMENT - ƯU TIÊN HEADER
                summary_value = ""
                
                # ✅ 1. ƯU TIÊN HEADER SUMMARY TRƯỚC
                header_summary = kie_data.get('summary', '').strip()
                if header_summary and header_summary not in ['', 'N/A', 'Không có', 'null']:
                    summary_value = header_summary
                    print(f"📝 Doc {van_ban_id} - Using HEADER summary: {summary_value[:50]}...")
                else:
                    # ✅ 2. NẾU HEADER TRỐNG THÌ LẤY CONTENT
                    content_summary = ""
                    
                    # Check if kie_data has nested structure
                    if 'kie_content_data' in kie_data:
                        content_data = kie_data.get('kie_content_data', {})
                        content_summary = content_data.get('summary', '').strip()
                    elif 'content_summary' in kie_data:
                        content_summary = kie_data.get('content_summary', '').strip()
                    
                    if content_summary and content_summary not in ['', 'N/A', 'Không có', 'null']:
                        summary_value = content_summary
                        print(f"📝 Doc {van_ban_id} - Using CONTENT summary (fallback): {summary_value[:50]}...")
                    else:
                        print(f"⚠️ Doc {van_ban_id} - No valid summary found in header or content")
                
                # ✅ MAP DOCUMENT DATA VỚI SMART SUMMARY
                payload.append({
                    "vanBanId": int(van_ban_id) if van_ban_id and van_ban_id.isdigit() else 0,
                    "vbSoVanBan": kie_data.get('document_number', ''),
                    "vbSoKyHieu": kie_data.get('document_symbol', ''),
                    "vbNgayThang": kie_data.get('issue_date', ''),
                    "vbLoaiVb": kie_data.get('document_type', ''),
                    "vbCoQuanBanHanh": kie_data.get('issuing_authority', ''),
                    "vbTrichyeu": summary_value,  # ✅ SỬ DỤNG SMART SUMMARY
                    "vbToSo": kie_data.get('handwritten_number', '')
                })
            
            print(f"📤 Batch updating {len(payload)} external documents with smart summary")
            
            response = self._make_authenticated_request('POST', url, json=payload)
            
            if response.status_code == 200:
                result = response.json()
                if result.get('success'):
                    print(f"✅ Batch external update successful for {len(payload)} documents")
                    
                    # ✅ LOG SUMMARY STATISTICS
                    summary_count = sum(1 for doc in payload if doc.get('vbTrichyeu'))
                    print(f"📊 Batch update stats: {summary_count}/{len(payload)} documents had summary")
                    
                    return {'success': True, 'updated_count': len(payload)}
                else:
                    print(f"❌ Batch external update failed: {result.get('message', 'Unknown error')}")
                    return {'success': False, 'error': result.get('message', 'Unknown error')}
            else:
                print(f"❌ Batch external update HTTP error: {response.status_code} - {response.text}")
                return {'success': False, 'error': f'HTTP {response.status_code}'}
                
        except Exception as e:
            print(f"❌ Batch external update error: {e}")
            return {'success': False, 'error': str(e)}

# Global instance
_external_api = None

def get_external_api():
    """Get global external API instance"""
    global _external_api
    if _external_api is None:
        _external_api = ExternalAPIManager()
    return _external_api