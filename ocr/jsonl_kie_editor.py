import sys
import json
import os
from pathlib import Path
import unicodedata
from PyQt5.QtWidgets import (
    QApplication, QMainWindow, QWidget, QVBoxLayout, QHBoxLayout,
    QPushButton, QLabel, QLineEdit, QTextEdit, QComboBox,
    QFileDialog, QScrollArea, QSpinBox, QDateEdit, QMessageBox,
    QFormLayout, QFrame, QGridLayout, QSlider
)
from PyQt5.QtGui import QPixmap, QFont, QColor
from PyQt5.QtCore import Qt, QDate, QSize
from PyQt5.QtWidgets import QTabWidget
from PIL import Image
import jsonlines

# ✅ THÊM UTILITY FUNCTION
def remove_accents_and_normalize(text):
    """Xóa dấu và convert thành chữ thường để compare"""
    if not text:
        return ""
    # Normalize unicode
    text = unicodedata.normalize('NFD', text)
    # Xóa dấu
    text = ''.join(c for c in text if unicodedata.category(c) != 'Mn')
    # Chữ thường
    return text.lower().strip()

class KIEEditorApp(QMainWindow):
    def __init__(self):
        super().__init__()
        self.jsonl_file = None
        self.data = []
        self.current_index = 0
        self.image_dir = None
        self.current_pixmap = None
        self.zoom_level = 1.5
        self.init_ui()
        
    def init_ui(self):
        """Khởi tạo giao diện"""
        self.setWindowTitle("📝 JSONL KIE Data Editor")
        self.setGeometry(100, 100, 1600, 900)
        
        central_widget = QWidget()
        self.setCentralWidget(central_widget)
        main_layout = QVBoxLayout(central_widget)
        
        # ===== TOP BAR: FILE & NAVIGATION =====
        top_bar = QHBoxLayout()
        
        btn_open = QPushButton("📂 Open JSONL File")
        btn_open.clicked.connect(self.open_jsonl_file)
        btn_open.setStyleSheet("padding: 8px; background: #51a478; color: white; border-radius: 5px;")
        top_bar.addWidget(btn_open)
        
        self.label_file = QLabel("No file loaded")
        self.label_file.setStyleSheet("color: #666; margin-left: 10px;")
        top_bar.addWidget(self.label_file)
        
        top_bar.addStretch()
        
        # Navigation
        btn_prev = QPushButton("⬅️ Previous")
        btn_prev.clicked.connect(self.prev_item)
        btn_prev.setStyleSheet("padding: 8px;")
        top_bar.addWidget(btn_prev)
        
        self.label_nav = QLabel("0 / 0")
        self.label_nav.setStyleSheet("min-width: 100px; text-align: center;")
        top_bar.addWidget(self.label_nav)
        
        btn_next = QPushButton("Next ➡️")
        btn_next.clicked.connect(self.next_item)
        btn_next.setStyleSheet("padding: 8px;")
        top_bar.addWidget(btn_next)
        
        main_layout.addLayout(top_bar)
        
        # ===== CONTENT: IMAGE + FORM =====
        content_layout = QHBoxLayout()
        
        # ===== LEFT: IMAGE =====
        left_frame = QFrame()
        left_frame.setStyleSheet("background: #f5f5f5; border-radius: 8px;")
        left_layout = QVBoxLayout(left_frame)
        
        # ===== IMAGE HEADER + ZOOM CONTROLS =====
        image_header = QHBoxLayout()
        
        label_image_title = QLabel("📷 Document Image")
        label_image_title.setFont(QFont("Arial", 12, QFont.Bold))
        label_image_title.setStyleSheet("color: #156c3e;")
        image_header.addWidget(label_image_title)
        
        image_header.addStretch()
        
        # ✅ ZOOM CONTROLS
        label_zoom = QLabel("🔍 Zoom:")
        image_header.addWidget(label_zoom)
        
        btn_zoom_minus = QPushButton("−")
        btn_zoom_minus.setMaximumWidth(40)
        btn_zoom_minus.clicked.connect(self.zoom_out)
        image_header.addWidget(btn_zoom_minus)
        
        self.label_zoom_level = QLabel("150%")
        self.label_zoom_level.setStyleSheet("min-width: 50px; text-align: center;")
        image_header.addWidget(self.label_zoom_level)
        
        btn_zoom_plus = QPushButton("+")
        btn_zoom_plus.setMaximumWidth(40)
        btn_zoom_plus.clicked.connect(self.zoom_in)
        image_header.addWidget(btn_zoom_plus)
        
        btn_zoom_reset = QPushButton("🔄 Reset")
        btn_zoom_reset.setMaximumWidth(80)
        btn_zoom_reset.clicked.connect(self.zoom_reset)
        image_header.addWidget(btn_zoom_reset)
        
        left_layout.addLayout(image_header)
        
        # ✅ ZOOM SLIDER
        slider_layout = QHBoxLayout()
        slider_layout.addWidget(QLabel("Slider:"))
        
        self.zoom_slider = QSlider(Qt.Horizontal)
        self.zoom_slider.setMinimum(50)
        self.zoom_slider.setMaximum(300)
        self.zoom_slider.setValue(150)
        self.zoom_slider.setTickPosition(QSlider.TicksBelow)
        self.zoom_slider.setTickInterval(50)
        self.zoom_slider.sliderMoved.connect(self.on_slider_moved)
        self.zoom_slider.valueChanged.connect(self.on_slider_changed)
        slider_layout.addWidget(self.zoom_slider)
        
        left_layout.addLayout(slider_layout)
        
        # ===== IMAGE DISPLAY WITH SCROLL =====
        scroll_image = QScrollArea()
        scroll_image.setWidgetResizable(True)
        scroll_image.setStyleSheet("border: 2px solid #ddd; border-radius: 6px;")
        
        self.image_label = QLabel()
        self.image_label.setAlignment(Qt.AlignCenter)
        self.image_label.setStyleSheet("background: white;")
        scroll_image.setWidget(self.image_label)
        
        left_layout.addWidget(scroll_image)
        
        content_layout.addWidget(left_frame, 1)
        
        # ===== RIGHT: FORM =====
        right_frame = QFrame()
        right_frame.setStyleSheet("background: #ffffff;")
        right_layout = QVBoxLayout(right_frame)
        
        label_form_title = QLabel("✏️ KIE Data Fields")
        label_form_title.setFont(QFont("Arial", 12, QFont.Bold))
        label_form_title.setStyleSheet("color: #156c3e;")
        right_layout.addWidget(label_form_title)
        
        # Scroll area for form fields
        scroll_area = QScrollArea()
        scroll_area.setWidgetResizable(True)
        scroll_area.setStyleSheet("border: none;")
        
        form_widget = QWidget()
        self.form_layout = QFormLayout(form_widget)
        self.form_layout.setSpacing(10)
        
        # ===== KIE FIELDS =====
        self.kie_fields = {}
        
        kie_field_names = [
            ("document_type", "Loại văn bản"),
            ("document_number_and_symbol", "Số & ký hiệu văn bản"),
            ("handwritten_number", "Số viết tay"),
            ("issue_date", "Ngày ban hành"),
            ("supervising_authority", "Cơ quan chủ quản"),
            ("issuing_authority", "Cơ quan ban hành"),
            ("signer_name", "Người ký"),
            ("summary", "Trích yếu"),
        ]
        
        for field_key, field_label in kie_field_names:
            label = QLabel(field_label)
            label.setStyleSheet("font-weight: bold; color: #333;")
            
            if field_key == "summary":
                field_input = QTextEdit()
                field_input.setMinimumHeight(80)
                field_input.setStyleSheet("""
                    border: 2px solid #e0e0e0;
                    border-radius: 6px;
                    padding: 8px;
                    font-size: 12px;
                """)
            else:
                field_input = QLineEdit()
                field_input.setStyleSheet("""
                    border: 2px solid #e0e0e0;
                    border-radius: 6px;
                    padding: 8px;
                    font-size: 12px;
                """)
            
            self.kie_fields[field_key] = field_input
            self.form_layout.addRow(label, field_input)
        
        scroll_area.setWidget(form_widget)
        right_layout.addWidget(scroll_area)
        
        # ===== BUTTONS =====
        btn_layout = QHBoxLayout()
        
        btn_save = QPushButton("💾 Save Changes")
        btn_save.clicked.connect(self.save_current_item)
        btn_save.setStyleSheet("""
            background: linear-gradient(135deg, #a9d86e, #8fc450);
            color: white;
            border: none;
            padding: 10px 20px;
            border-radius: 20px;
            font-weight: bold;
        """)
        btn_layout.addWidget(btn_save)
        
        btn_export = QPushButton("📤 Export Edited JSONL")
        btn_export.clicked.connect(self.export_jsonl)
        btn_export.setStyleSheet("""
            background: linear-gradient(135deg, #48cfad, #37bc9b);
            color: white;
            border: none;
            padding: 10px 20px;
            border-radius: 20px;
            font-weight: bold;
        """)
        btn_layout.addWidget(btn_export)
        
        right_layout.addLayout(btn_layout)
        
        content_layout.addWidget(right_frame, 1)
        
        main_layout.addLayout(content_layout)
        
    # ✅ ZOOM FUNCTIONS
    def zoom_in(self):
        """Phóng to"""
        self.zoom_level += 0.2
        self.zoom_level = min(self.zoom_level, 3.0)
        self.update_image_display()
    
    def zoom_out(self):
        """Thu nhỏ"""
        self.zoom_level -= 0.2
        self.zoom_level = max(self.zoom_level, 0.5)
        self.update_image_display()
    
    def zoom_reset(self):
        """Reset zoom về 150%"""
        self.zoom_level = 1.5
        self.update_image_display()
    
    def on_slider_moved(self, value):
        """Khi drag slider"""
        self.zoom_level = value / 100.0
        self.update_image_display()
    
    def on_slider_changed(self, value):
        """Khi slider value thay đổi"""
        self.zoom_slider.blockSignals(True)
        self.zoom_slider.setValue(int(self.zoom_level * 100))
        self.zoom_slider.blockSignals(False)
    
    def update_image_display(self):
        """Cập nhật hiển thị ảnh với zoom level hiện tại"""
        if self.current_pixmap is None:
            return
        
        new_width = int(self.current_pixmap.width() * self.zoom_level)
        scaled_pixmap = self.current_pixmap.scaledToWidth(new_width, Qt.SmoothTransformation)
        
        self.image_label.setPixmap(scaled_pixmap)
        
        zoom_percent = int(self.zoom_level * 100)
        self.label_zoom_level.setText(f"{zoom_percent}%")
        
        if not self.zoom_slider.isSliderDown():
            self.zoom_slider.blockSignals(True)
            self.zoom_slider.setValue(zoom_percent)
            self.zoom_slider.blockSignals(False)
    
        # ✅ THÊM FUNCTION MỚI: CHECK VÀ MERGE SUMMARY
    def should_merge_summary_with_type(self, doc_type, summary):
        """
        Kiểm tra: nếu văn bản KHÔNG phải "Công văn" hoặc "Khác" và summary KHÔNG chứa tên loại văn bản
        thì merge: "TÊN LOẠI VĂN BẢN" + "Trích yếu"
        
        So sánh dùng normalize (không dấu, không viết hoa)
        """
        if not doc_type or not summary:
            return False, summary
        
        # Normalize để compare
        doc_type_normalized = remove_accents_and_normalize(doc_type)
        summary_normalized = remove_accents_and_normalize(summary)
        
        # ✅ LOGIC: Nếu loại văn bản là "Công văn" hoặc "Khác" thì không merge
        if "cong van" in doc_type_normalized or "khac" in doc_type_normalized:
            print(f"ℹ️ Document type is '{doc_type}' - không merge")
            return False, summary
        
        # ✅ LOGIC: Kiểm tra xem summary có chứa tên loại văn bản không
        if doc_type_normalized in summary_normalized:
            print(f"✓ Summary đã chứa '{doc_type}' - không cần merge")
            return False, summary
        
        # ✅ LOGIC: MERGE - Uppercase toàn bộ tên loại văn bản + thêm vào đầu summary
        uppercase_doc_type = doc_type.upper()
        
        print(f"🔄 Merging: '{uppercase_doc_type}' + '{summary}'")
        merged_summary = f"{uppercase_doc_type} {summary}"
        return True, merged_summary
    
    def open_jsonl_file(self):
        """Mở file JSONL"""
        file_path, _ = QFileDialog.getOpenFileName(
            self,
            "Open JSONL File",
            "",
            "JSONL Files (*.jsonl);;All Files (*)"
        )
        
        if not file_path:
            return
        
        self.jsonl_file = file_path
        self.image_dir = Path(file_path).parent
        self.load_jsonl_data()
        
    def load_jsonl_data(self):
        """Load dữ liệu từ JSONL file"""
        try:
            self.data = []
            with jsonlines.open(self.jsonl_file) as reader:
                for obj in reader:
                    self.data.append(obj)
            
            self.current_index = 0
            self.label_file.setText(f"✅ Loaded: {Path(self.jsonl_file).name} ({len(self.data)} items)")
            
            self.zoom_level = 1.5
            self.zoom_slider.setValue(150)
            
            self.display_current_item()
            
        except Exception as e:
            QMessageBox.critical(self, "Error", f"Failed to load JSONL: {str(e)}")
    
    def display_current_item(self):
        """Hiển thị item hiện tại"""
        if not self.data:
            return
        
        item = self.data[self.current_index]
        
        # ===== HIỂN THỊ ẢNH =====
        image_path = item.get("image", "")
        if image_path:
            if not image_path.startswith("images/"):
                image_path = f"images/{image_path}"
            
            full_image_path = self.image_dir / image_path
            
            print(f"🔍 Looking for image: {full_image_path}")
            
            if full_image_path.exists():
                self.current_pixmap = QPixmap(str(full_image_path))
                
                if self.current_pixmap.isNull():
                    self.image_label.setText(f"❌ Cannot load image:\n{image_path}")
                else:
                    self.update_image_display()
                    print(f"✅ Image loaded: {image_path}")
            else:
                self.current_pixmap = None
                self.image_label.setText(f"❌ Image not found:\n{full_image_path}")
                print(f"❌ File not found: {full_image_path}")
        else:
            self.current_pixmap = None
            self.image_label.setText("No image path")
        
        # ===== HIỂN THỊ FORM =====
        kie_data = item.get("conversations", [])
        if kie_data and len(kie_data) > 1:
            response_text = kie_data[-1].get("value", "{}")
            try:
                kie_dict = json.loads(response_text)
                print(f"🔍 KIE Data keys: {kie_dict.keys()}")
                
                # ✅ THÊM LOGIC: CHECK VÀ MERGE SUMMARY NẾU CẦN
                doc_type = kie_dict.get("document_type", "")
                summary = kie_dict.get("summary", "")
                
                should_merge, merged_summary = self.should_merge_summary_with_type(doc_type, summary)
                
                # if should_merge:
                #     kie_dict["summary"] = merged_summary
                
                # Populate fields
                for field_key, field_widget in self.kie_fields.items():
                    value = kie_dict.get(field_key, "")
                    if isinstance(field_widget, QTextEdit):
                        field_widget.setText(str(value))
                    else:
                        field_widget.setText(str(value))
                        
            except json.JSONDecodeError as e:
                print(f"❌ JSON Parse Error: {e}")
                print(f"Response text: {response_text[:200]}")
        
        self.label_nav.setText(f"{self.current_index + 1} / {len(self.data)}")

    def save_current_item(self):
        """Lưu thay đổi cho item hiện tại - AUTO SAVE vào file gốc"""
        if not self.data:
            QMessageBox.warning(self, "Warning", "No data loaded")
            return
        
        item = self.data[self.current_index]
        
        # ===== COLLECT DATA FROM FORM =====
        kie_dict = {}
        for field_key, field_widget in self.kie_fields.items():
            if isinstance(field_widget, QTextEdit):
                kie_dict[field_key] = field_widget.toPlainText().strip()
            else:
                kie_dict[field_key] = field_widget.text().strip()
        
        # ===== UPDATE ITEM IN MEMORY =====
        if "conversations" in item and len(item["conversations"]) > 1:
            item["conversations"][-1]["value"] = json.dumps(kie_dict, ensure_ascii=False, indent=2)
        
        self.data[self.current_index] = item
        
        # ✅ THÊM: AUTO-SAVE vào file gốc
        try:
            with jsonlines.open(self.jsonl_file, mode='w') as writer:
                for data_item in self.data:
                    writer.write(data_item)
            
            QMessageBox.information(
                self, 
                "Success", 
                f"✅ Item {self.current_index + 1} saved to file!"
            )
            print(f"💾 Auto-saved to: {self.jsonl_file}")
        except Exception as e:
            QMessageBox.critical(self, "Error", f"Failed to save file: {str(e)}")
    def next_item(self):
        """Item tiếp theo - AUTO SAVE trước khi chuyển"""
        if self.current_index < len(self.data) - 1:
            # ✅ THÊM: Auto-save item hiện tại trước khi chuyển
            self.auto_save_current_item()
            
            self.current_index += 1
            self.display_current_item()
    
    def prev_item(self):
        """Item trước đó - AUTO SAVE trước khi chuyển"""
        if self.current_index > 0:
            # ✅ THÊM: Auto-save item hiện tại trước khi chuyển
            self.auto_save_current_item()
            
            self.current_index -= 1
            self.display_current_item()
    
    # ✅ THÊM HÀM MỚI: Auto-save (im lặng, không show dialog)
    def auto_save_current_item(self):
        """Tự động lưu item hiện tại - im lặng (không show notification)"""
        if not self.data or self.current_index >= len(self.data):
            return
        
        item = self.data[self.current_index]
        
        # ===== COLLECT DATA FROM FORM =====
        kie_dict = {}
        for field_key, field_widget in self.kie_fields.items():
            if isinstance(field_widget, QTextEdit):
                kie_dict[field_key] = field_widget.toPlainText().strip()
            else:
                kie_dict[field_key] = field_widget.text().strip()
        
        # ===== UPDATE ITEM IN MEMORY =====
        if "conversations" in item and len(item["conversations"]) > 1:
            item["conversations"][-1]["value"] = json.dumps(kie_dict, ensure_ascii=False, indent=2)
        
        self.data[self.current_index] = item
        
        # ✅ SAVE VÀO FILE (im lặng, không show dialog)
        try:
            with jsonlines.open(self.jsonl_file, mode='w') as writer:
                for data_item in self.data:
                    writer.write(data_item)
            
            print(f"💾 Auto-saved item {self.current_index + 1}")
        except Exception as e:
            print(f"❌ Failed to auto-save: {e}")
    
    def export_jsonl(self):
        """Export edited data về JSONL file"""
        if not self.data:
            QMessageBox.warning(self, "Warning", "No data to export")
            return
        
        output_file, _ = QFileDialog.getSaveFileName(
            self,
            "Export to JSONL",
            "",
            "JSONL Files (*.jsonl)"
        )
        
        if not output_file:
            return
        
        try:
            with jsonlines.open(output_file, mode='w') as writer:
                for item in self.data:
                    writer.write(item)
            
            QMessageBox.information(
                self,
                "Success",
                f"✅ Exported {len(self.data)} items to:\n{output_file}"
            )
        except Exception as e:
            QMessageBox.critical(self, "Error", f"Failed to export: {str(e)}")


def main():
    app = QApplication(sys.argv)
    window = KIEEditorApp()
    window.show()
    sys.exit(app.exec_())


if __name__ == "__main__":
    main()