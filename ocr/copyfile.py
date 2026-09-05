import os
import shutil
import json
from pathlib import Path
from typing import Dict, List, Tuple, Optional
from dataclasses import dataclass
from datetime import datetime
import random
from collections import defaultdict

@dataclass
class PDFCopyConfig:
    """Cấu hình cho việc copy PDF"""
    source_folder: str
    destination_folder: str
    limit_per_folder: int
    selection_strategy: str = "first"  # "first", "random", "largest", "newest"
    preserve_structure: bool = True
    overwrite_existing: bool = False
    include_subfolders: bool = True
    max_depth: int = 10
    dry_run: bool = False

@dataclass
class PDFFileInfo:
    """Thông tin về một file PDF"""
    source_path: str
    relative_path: str
    parent_folder: str
    size_bytes: int
    modified_time: float
    filename: str

class PDFCopyManager:
    """Quản lý việc copy PDF files"""
    
    def __init__(self, config: PDFCopyConfig):
        self.config = config
        self.pdf_files: Dict[str, List[PDFFileInfo]] = defaultdict(list)
        self.copy_plan: List[Tuple[str, str]] = []
        self.results = {
            "total_folders_scanned": 0,
            "total_pdfs_found": 0,
            "total_pdfs_copied": 0,
            "folders_processed": {},
            "errors": [],
            "execution_time": 0,
            "start_time": None,
            "dry_run": config.dry_run
        }
    
    def discover_pdf_files(self) -> None:
        """Phase 1: Tìm tất cả PDF files và group theo folder cha"""
        print(f"🔍 Đang scan folder: {self.config.source_folder}")
        
        if not os.path.exists(self.config.source_folder):
            raise FileNotFoundError(f"Source folder không tồn tại: {self.config.source_folder}")
        
        source_path = Path(self.config.source_folder)
        
        # Duyệt tất cả folder con level 1
        for item in source_path.iterdir():
            if item.is_dir():
                self.results["total_folders_scanned"] += 1
                folder_name = item.name
                print(f"📁 Đang scan folder con: {folder_name}")
                
                # Tìm PDF trong folder con này (và subfolder nếu enable)
                pdf_files = self._find_pdfs_in_folder(item, folder_name)
                
                if pdf_files:
                    self.pdf_files[folder_name] = pdf_files
                    print(f"   ✅ Tìm thấy {len(pdf_files)} PDF files")
                else:
                    print(f"   ⚪ Không có PDF files")
        
        self.results["total_pdfs_found"] = sum(len(files) for files in self.pdf_files.values())
        print(f"\n📊 Tổng kết discovery:")
        print(f"   📁 Folders scanned: {self.results['total_folders_scanned']}")
        print(f"   📄 Total PDFs found: {self.results['total_pdfs_found']}")
    
    def _find_pdfs_in_folder(self, folder_path: Path, parent_folder: str, current_depth: int = 0) -> List[PDFFileInfo]:
        """Tìm PDF files trong một folder (có thể recursive)"""
        pdf_files = []
        
        if current_depth > self.config.max_depth:
            return pdf_files
        
        try:
            for item in folder_path.iterdir():
                if item.is_file() and item.suffix.lower() == '.pdf':
                    # Tính relative path từ source folder
                    relative_path = item.relative_to(Path(self.config.source_folder))
                    
                    try:
                        stat = item.stat()
                        pdf_info = PDFFileInfo(
                            source_path=str(item),
                            relative_path=str(relative_path),
                            parent_folder=parent_folder,
                            size_bytes=stat.st_size,
                            modified_time=stat.st_mtime,
                            filename=item.name
                        )
                        pdf_files.append(pdf_info)
                    except OSError as e:
                        self.results["errors"].append(f"Cannot access file {item}: {e}")
                
                elif item.is_dir() and self.config.include_subfolders and current_depth < self.config.max_depth:
                    # Recursive vào subfolder
                    sub_pdfs = self._find_pdfs_in_folder(item, parent_folder, current_depth + 1)
                    pdf_files.extend(sub_pdfs)
                    
        except PermissionError as e:
            self.results["errors"].append(f"Permission denied accessing {folder_path}: {e}")
        except Exception as e:
            self.results["errors"].append(f"Error scanning {folder_path}: {e}")
        
        return pdf_files
    
    def create_copy_plan(self) -> None:
        """Phase 2: Tạo plan copy files với limit"""
        print(f"\n📋 Đang tạo copy plan với limit {self.config.limit_per_folder} files per folder...")
        
        for folder_name, pdf_files in self.pdf_files.items():
            print(f"\n📁 Folder: {folder_name}")
            print(f"   📄 Found: {len(pdf_files)} PDFs")
            
            # Apply selection strategy
            selected_files = self._select_files(pdf_files, self.config.limit_per_folder)
            
            print(f"   ✅ Selected: {len(selected_files)} PDFs")
            
            # Tạo copy plan cho files được chọn
            for pdf_file in selected_files:
                source_path = pdf_file.source_path
                
                if self.config.preserve_structure:
                    # Giữ nguyên cấu trúc thư mục
                    dest_path = os.path.join(self.config.destination_folder, pdf_file.relative_path)
                else:
                    # Flat structure - tất cả vào folder đích
                    dest_path = os.path.join(self.config.destination_folder, pdf_file.filename)
                
                self.copy_plan.append((source_path, dest_path))
            
            # Update results
            self.results["folders_processed"][folder_name] = {
                "found": len(pdf_files),
                "selected": len(selected_files)
            }
        
        # ✅ FIX: len(self.copy_plan) đã là int, không cần sum()
        print(f"\n📊 Copy plan created:")
        print(f"   📄 Total files to copy: {len(self.copy_plan)}")
    def _select_files(self, pdf_files: List[PDFFileInfo], limit: int) -> List[PDFFileInfo]:
        """Chọn files theo strategy"""
        if len(pdf_files) <= limit:
            return pdf_files
        
        if self.config.selection_strategy == "first":
            return pdf_files[:limit]
        
        elif self.config.selection_strategy == "random":
            return random.sample(pdf_files, limit)
        
        elif self.config.selection_strategy == "largest":
            return sorted(pdf_files, key=lambda x: x.size_bytes, reverse=True)[:limit]
        
        elif self.config.selection_strategy == "newest":
            return sorted(pdf_files, key=lambda x: x.modified_time, reverse=True)[:limit]
        
        elif self.config.selection_strategy == "smallest":
            return sorted(pdf_files, key=lambda x: x.size_bytes)[:limit]
        
        elif self.config.selection_strategy == "oldest":
            return sorted(pdf_files, key=lambda x: x.modified_time)[:limit]
        
        else:
            print(f"⚠️ Unknown selection strategy: {self.config.selection_strategy}, using 'first'")
            return pdf_files[:limit]
    
    def execute_copy_plan(self) -> None:
        """Phase 3: Thực hiện copy files"""
        if self.config.dry_run:
            print(f"\n🧪 DRY RUN MODE - Không copy thực tế")
            self._preview_copy_plan()
            return
        
        print(f"\n🚀 Đang thực hiện copy {len(self.copy_plan)} files...")
        
        copied_count = 0
        total_size = 0
        
        for i, (source_path, dest_path) in enumerate(self.copy_plan, 1):
            try:
                # Tạo thư mục đích nếu chưa có
                dest_dir = os.path.dirname(dest_path)
                os.makedirs(dest_dir, exist_ok=True)
                
                # Kiểm tra file đích đã tồn tại
                if os.path.exists(dest_path) and not self.config.overwrite_existing:
                    print(f"   ⚠️ [{i}/{len(self.copy_plan)}] Skipped (exists): {os.path.basename(dest_path)}")
                    continue
                
                # Copy file
                file_size = os.path.getsize(source_path)
                shutil.copy2(source_path, dest_path)
                
                copied_count += 1
                total_size += file_size
                
                size_mb = file_size / (1024 * 1024)
                print(f"   ✅ [{i}/{len(self.copy_plan)}] Copied: {os.path.basename(dest_path)} ({size_mb:.1f}MB)")
                
            except Exception as e:
                error_msg = f"Failed to copy {source_path} -> {dest_path}: {e}"
                self.results["errors"].append(error_msg)
                print(f"   ❌ [{i}/{len(self.copy_plan)}] Error: {os.path.basename(source_path)} - {e}")
        
        self.results["total_pdfs_copied"] = copied_count
        total_size_mb = total_size / (1024 * 1024)
        
        print(f"\n✅ Copy hoàn thành:")
        print(f"   📄 Files copied: {copied_count}/{len(self.copy_plan)}")
        print(f"   💾 Total size: {total_size_mb:.1f}MB")
        if self.results["errors"]:
            print(f"   ⚠️ Errors: {len(self.results['errors'])}")
    
    def _preview_copy_plan(self) -> None:
        """Preview copy plan trong dry run mode"""
        print(f"\n📋 COPY PLAN PREVIEW:")
        print(f"{'='*80}")
        
        for folder_name, info in self.results["folders_processed"].items():
            print(f"\n📁 {folder_name}:")
            print(f"   Found: {info['found']} PDFs")
            print(f"   Will copy: {info['selected']} PDFs")
        
        print(f"\n📄 FILES TO COPY:")
        print(f"{'='*80}")
        
        for i, (source_path, dest_path) in enumerate(self.copy_plan, 1):
            source_file = os.path.basename(source_path)
            dest_relative = os.path.relpath(dest_path, self.config.destination_folder)
            size_mb = os.path.getsize(source_path) / (1024 * 1024)
            print(f"{i:3d}. {source_file} -> {dest_relative} ({size_mb:.1f}MB)")
    
    def run(self) -> Dict:
        """Chạy toàn bộ process"""
        self.results["start_time"] = datetime.now()
        start_time = datetime.now()
        
        try:
            print("🚀 Bắt đầu PDF Copy Process")
            print(f"📁 Source: {self.config.source_folder}")
            print(f"📁 Destination: {self.config.destination_folder}")
            print(f"🔢 Limit per folder: {self.config.limit_per_folder}")
            print(f"🎯 Selection strategy: {self.config.selection_strategy}")
            print(f"🧪 Dry run: {self.config.dry_run}")
            
            # Phase 1: Discovery
            self.discover_pdf_files()
            
            # Phase 2: Planning  
            self.create_copy_plan()
            
            # Phase 3: Execution
            self.execute_copy_plan()
            
        except Exception as e:
            self.results["errors"].append(f"Critical error: {e}")
            print(f"❌ Critical error: {e}")
        
        # Tính execution time
        end_time = datetime.now()
        self.results["execution_time"] = (end_time - start_time).total_seconds()
        
        # Print summary
        self._print_summary()
        
        return self.results
    
    def _print_summary(self) -> None:
        """In tóm tắt kết quả"""
        print(f"\n{'='*80}")
        print(f"📊 SUMMARY REPORT")
        print(f"{'='*80}")
        print(f"⏱️  Execution time: {self.results['execution_time']:.2f} seconds")
        print(f"📁 Folders scanned: {self.results['total_folders_scanned']}")
        print(f"📄 PDFs found: {self.results['total_pdfs_found']}")
        print(f"✅ PDFs copied: {self.results['total_pdfs_copied']}")
        
        if self.results["errors"]:
            print(f"⚠️  Errors: {len(self.results['errors'])}")
            for error in self.results["errors"][:5]:  # Show first 5 errors
                print(f"   - {error}")
            if len(self.results["errors"]) > 5:
                print(f"   ... và {len(self.results['errors']) - 5} errors khác")
        
        print(f"\n📋 Per-folder results:")
        for folder_name, info in self.results["folders_processed"].items():
            print(f"   📁 {folder_name}: {info['found']} found → {info['selected']} selected")
    
    def save_report(self, report_path: str) -> None:
        """Lưu báo cáo ra file JSON"""
        try:
            # Convert datetime to string for JSON serialization
            report_data = self.results.copy()
            if report_data["start_time"]:
                report_data["start_time"] = report_data["start_time"].isoformat()
            
            with open(report_path, 'w', encoding='utf-8') as f:
                json.dump(report_data, f, indent=2, ensure_ascii=False)
            print(f"📄 Report saved: {report_path}")
        except Exception as e:
            print(f"❌ Error saving report: {e}")

def main():
    """Demo sử dụng"""
    
    # Cấu hình
    config = PDFCopyConfig(
        source_folder=r"C:\SourceFolder",  # Thay đổi đường dẫn này
        destination_folder=r"C:\DestinationFolder",  # Thay đổi đường dẫn này
        limit_per_folder=5,  # Copy tối đa 5 files per folder con
        selection_strategy="newest",  # "first", "random", "largest", "newest", "smallest", "oldest"
        preserve_structure=True,  # Giữ nguyên cấu trúc thư mục
        overwrite_existing=False,  # Không ghi đè file đã tồn tại
        include_subfolders=True,  # Bao gồm subfolder
        max_depth=5,  # Giới hạn độ sâu
        dry_run=True  # Dry run để preview trước
    )
    
    # Chạy
    manager = PDFCopyManager(config)
    results = manager.run()
    
    # Lưu report
    report_path = "pdf_copy_report.json"
    manager.save_report(report_path)
    
    return results

if __name__ == "__main__":
    # Example usage với các config khác nhau
    
    print("📋 PDF Copy Manager - Multiple Examples")
    print("="*50)
    
    # # Example 1: Copy 3 newest files per folder, preserve structure
    # config1 = PDFCopyConfig(
    #     source_folder=r"F:\\Project\\PLA\\Data\\data_ocr",
    #     destination_folder=r"F:\\Project\\PLA\\Data\\data_ocr_finetune",
    #     limit_per_folder=5,
    #     selection_strategy="newest",
    #     preserve_structure=True,
    #     dry_run=True
    # )
    
    # print("\n🔹 Example 1: Copy 3 newest PDFs per folder")
    # manager1 = PDFCopyManager(config1)
    # results1 = manager1.run()
    
    # Example 2: Copy 10 random files per folder, flat structure  
    config2 = PDFCopyConfig(
        source_folder=r"F:\\DỮ LIỆU HỌC OCR\\UBND tỉnh HBI\\KHỐI NỘI CHÍNH\\CÓ THỜI HẠN\\Hộp số  209",
        destination_folder=r"D:\\PLA\\Data\\ocr_data_finetune_5",
        limit_per_folder=2,
        selection_strategy="random",
        preserve_structure=True,
        dry_run=False
    )
    
    print("\n🔹 Example 2: Copy 10 random PDFs per folder (flat)")
    manager2 = PDFCopyManager(config2)
    results2 = manager2.run()
    
    # # Example 3: Copy largest files only
    # config3 = PDFCopyConfig(
    #     source_folder=r"F:\\Project\\PLA\\Data\\data_ocr",
    #     destination_folder=r"F:\\Project\\PLA\\Data\\data_ocr_finetune",
    #     limit_per_folder=5,
    #     selection_strategy="largest",
    #     preserve_structure=True,
    #     overwrite_existing=True,
    #     dry_run=True  # Actual copy
    # )
    
    # print("\n🔹 Example 3: Copy 2 largest PDFs per folder")
    # manager3 = PDFCopyManager(config2)
    # results3 = manager3.run()
    
    print("\n✅ All examples completed!")
    print("💡 Thay đổi 'dry_run=False' để thực hiện copy thật")
    print("💡 Modify đường dẫn source_folder và destination_folder phù hợp")