let currentPage = 1;
let pageSize = 10;
let currentSearch = "";
let currentStatusFilter = "";
let totalPages = 1;
let totalItems = 0;
let currentBatchDocuments = [];
let totalBatchDocuments = 0;
let currentDocId = null;
let currentDocument = null;
let currentDocumentIndex = -1;
let currentView = "full";

// ===== INITIALIZATION =====
document.addEventListener("DOMContentLoaded", async function () {
  await checkAuthentication();
  setupFilters();
  loadDocumentsPage();

  // Auto refresh every 10 seconds
  setInterval(() => {
    loadDocumentsPage();
  }, 10000);
});

// ===== AUTHENTICATION =====
async function checkAuthentication() {
  try {
    const response = await fetch("/api/auth/me", {
      credentials: "include",
    });

    if (response.ok) {
      const user = await response.json();
      updateUserInfo(user);
      return true;
    } else if (response.status === 401) {
      window.location.href = "/login";
      return false;
    }
  } catch (error) {
    console.error("Auth check failed:", error);
    return false;
  }
}

function updateUserInfo(user) {
  document.getElementById("userName").textContent = user.username || user.email;
  document.getElementById("userInfo").style.display = "block";
  document.getElementById("logoutBtn").style.display = "block";

  // Build sidebar based on role
  const menu = document.getElementById("sidebarMenu");
  if (menu) {
    let items =
      '<li class="mt"><a href="/"><i class="fa fa-home"></i><span>Trang chủ</span></a></li>';
    if (user.role === "admin") {
      items +=
        '<li><a href="/admin"><i class="fa fa-shield-alt"></i><span>Admin Panel</span></a></li>';
    } else if (user.role === "customer") {
      items +=
        '<li><a href="/customer-statistics-page"><i class="fa fa-chart-bar"></i><span>Thống kê</span></a></li>';
      items +=
        '<li><a href="/manage-packages-page"><i class="fa fa-archive"></i><span>Quản lý gói</span></a></li>';
    } else if (user.role === "cooperator") {
      items +=
        '<li><a href="/work-packages-page"><i class="fa fa-tasks"></i><span>Gói công việc</span></a></li>';
    }
    menu.innerHTML = items;
  }
}

// ===== FILTERS SETUP =====
function setupFilters() {
  let searchTimeout;
  const searchInput = document.getElementById("searchDocuments");
  if (searchInput) {
    searchInput.addEventListener("input", function () {
      clearTimeout(searchTimeout);
      searchTimeout = setTimeout(() => {
        currentSearch = this.value.trim();
        currentPage = 1;
        loadDocumentsPage();
      }, 500);
    });
  }

  const statusFilter = document.getElementById("statusFilter");
  if (statusFilter) {
    statusFilter.addEventListener("change", function () {
      currentStatusFilter = this.value;
      currentPage = 1;
      loadDocumentsPage();
    });
  }

  const pageSizeSelect = document.getElementById("pageSizeSelect");
  if (pageSizeSelect) {
    pageSizeSelect.addEventListener("change", function () {
      pageSize = parseInt(this.value);
      currentPage = 1;
      loadDocumentsPage();
    });
  }
}

// ===== LOAD DOCUMENTS =====
async function loadDocumentsPage() {
  try {
    const params = new URLSearchParams({
      page: currentPage,
      page_size: pageSize,
      search: currentSearch,
      status: currentStatusFilter,
    });

    const response = await fetch(`/api/batch/${BATCH_ID}/documents?${params}`);

    if (!response.ok) {
      throw new Error(`HTTP ${response.status}`);
    }

    const data = await response.json();

    totalPages = data.pagination.total_pages;
    totalItems = data.pagination.total_items;
    currentBatchDocuments = data.documents;
    totalBatchDocuments = data.pagination.total_items;

    renderDocumentsTable(data.documents);
    updatePaginationInfo(data.pagination);
    renderPaginationControls(data.pagination);
  } catch (error) {
    showToast("Lỗi tải danh sách: " + error.message, "danger");
  }
}

// ===== RENDER TABLE =====
function renderDocumentsTable(documents) {
  const tbody = document.getElementById("documentsTableBody");

  if (documents.length === 0) {
    tbody.innerHTML = `
      <tr>
        <td colspan="5" class="text-center text-muted py-4">
          <i class="fas fa-inbox fa-2x mb-2"></i><br>
          Không tìm thấy tài liệu nào
        </td>
      </tr>
    `;
    return;
  }

  tbody.innerHTML = documents
    .map((doc) => {
      let documentType = "N/A";

      try {
        if (doc.kie_header_data && doc.kie_header_data.document_type) {
          documentType = doc.kie_header_data.document_type;
        } else {
          documentType = doc.document_type || "N/A";
        }
      } catch (error) {
        documentType = doc.document_type || "N/A";
      }

      const typeSource =
        documentType !== "N/A" && documentType !== (doc.document_type || "N/A")
          ? "kie"
          : "db";
      const typeClass = typeSource === "kie" ? "text-primary" : "text-muted";
      const typeTitle =
        typeSource === "kie"
          ? "Loại văn bản từ KIE"
          : "Loại văn bản từ database";

      return `
        <tr id="doc-row-${doc.id}">
          <td class="filename-cell">
            <div class="filename-tooltip">
              <i class="fas fa-file-pdf text-danger me-1"></i>
              <span class="filename-text">${doc.filename}</span>
            </div>
          </td>
          <td>
            <small class="${typeClass}" title="${typeTitle}">
              ${documentType}
              ${
                typeSource === "kie"
                  ? '<i class="fas fa-check-circle text-success ms-1"></i>'
                  : ""
              }
            </small>
          </td>
          <td class="text-center">
            <span class="badge bg-info text-dark">
              <i class="fas fa-file-alt me-1"></i>${doc.total_pages || 1}
            </span>
          </td>
          <td>
            <span class="badge bg-${getStatusColor(doc.status)}" id="status-${
              doc.id
            }">
              ${getStatusText(doc.status)}
            </span>
          </td>
          <td>
            ${generateCompactActionButtons(doc.id, doc.status)}
          </td>
        </tr>
      `;
    })
    .join("");
}

// ===== ACTION BUTTONS =====
function generateCompactActionButtons(docId, status) {
  let buttons = '<div class="btn-group btn-group-sm" role="group">';

  if (status === "completed") {
    buttons += `
      <button class="btn btn-outline-primary btn-sm" onclick="editDocument('${docId}')" title="Chỉnh sửa">
        <i class="fas fa-edit"></i>
      </button>`;
  }

  if (status !== "uploading") {
    const iconClass =
      status === "processing" ? "fa-sync-alt fa-spin" : "fa-redo";
    const titleText = status === "processing" ? "Đang xử lý..." : "Chạy lại";
    const disabled = status === "processing" ? "disabled" : "";

    buttons += `
      <button class="btn btn-outline-warning btn-sm ${disabled}"
              onclick="processSingleDocument('${docId}', true)"
              title="${titleText}">
        <i class="fas ${iconClass}"></i>
      </button>`;
  }

  if (status === "processing" || status === "error") {
    buttons += `
      <button class="btn btn-outline-secondary btn-sm" onclick="tryEditDocument('${docId}')" title="Xem/Sửa">
        <i class="fas fa-eye"></i>
      </button>`;
  }

  buttons += "</div>";
  return buttons;
}

// ===== PAGINATION =====
function renderPaginationControls(pagination) {
  const paginationEl = document.getElementById("pagination");

  if (pagination.total_pages <= 1) {
    paginationEl.innerHTML = "";
    return;
  }

  let html = "";

  html += `
    <li class="page-item ${!pagination.has_prev ? "disabled" : ""}">
      <a class="page-link" href="#" onclick="goToPage(${
        pagination.current_page - 1
      }); return false;" ${!pagination.has_prev ? 'tabindex="-1"' : ""}>
        <i class="fas fa-chevron-left"></i>
      </a>
    </li>
  `;

  const startPage = Math.max(1, pagination.current_page - 2);
  const endPage = Math.min(pagination.total_pages, startPage + 4);

  for (let i = startPage; i <= endPage; i++) {
    html += `
      <li class="page-item ${pagination.current_page === i ? "active" : ""}">
        <a class="page-link" href="#" onclick="goToPage(${i}); return false;">${i}</a>
      </li>
    `;
  }

  html += `
    <li class="page-item ${!pagination.has_next ? "disabled" : ""}">
      <a class="page-link" href="#" onclick="goToPage(${
        pagination.current_page + 1
      }); return false;" ${!pagination.has_next ? 'tabindex="-1"' : ""}>
        <i class="fas fa-chevron-right"></i>
      </a>
    </li>
  `;

  paginationEl.innerHTML = html;
}

function updatePaginationInfo(pagination) {
  const start = (pagination.current_page - 1) * pagination.page_size + 1;
  const end = Math.min(
    pagination.current_page * pagination.page_size,
    pagination.total_items,
  );

  const showingInfo = document.getElementById("showingInfo");
  if (showingInfo) {
    showingInfo.textContent = `Hiển thị ${start}-${end} của ${pagination.total_items}`;
  }

  const totalCountDisplay = document.getElementById("totalCountDisplay");
  if (totalCountDisplay) {
    totalCountDisplay.textContent = `(${pagination.total_items} files)`;
  }
}

function goToPage(page) {
  if (page < 1 || page > totalPages) return;
  currentPage = page;
  loadDocumentsPage();
}

// ===== REFRESH =====
async function refreshDocumentsList() {
  await loadDocumentsPage();
  showToast("Đã làm mới danh sách", "success");
}

// ===== EDIT DOCUMENT (OPEN MODAL) =====
async function editDocument(docId) {
  currentDocId = docId;

  try {
    const response = await fetch(`/api/document/${docId}`);
    const docData = await response.json();
    currentDocument = docData;

    let documentIndex = currentBatchDocuments.findIndex(
      (doc) => doc.id === docId,
    );

    showDocumentEditor(docData, currentBatchDocuments, documentIndex);
  } catch (error) {
    showToast("Lỗi khi tải document: " + error.message, "danger");
  }
}

async function tryEditDocument(docId) {
  currentDocId = docId;

  try {
    const response = await fetch(`/api/document/${docId}`);
    const docData = await response.json();

    const hasHeaderData =
      docData.kie_header_data &&
      Object.keys(docData.kie_header_data).length > 0;
    const hasContentData =
      docData.kie_content_data &&
      Object.keys(docData.kie_content_data).length > 0;
    const hasImages =
      docData.image_paths &&
      (docData.image_paths.header || docData.image_paths.full);

    if (hasHeaderData || hasContentData || hasImages) {
      currentDocument = docData;
      let documentIndex = currentBatchDocuments.findIndex(
        (doc) => doc.id === docId,
      );
      showDocumentEditor(docData, currentBatchDocuments, documentIndex);
    } else {
      showToast("Document chưa được xử lý hoặc chưa có dữ liệu KIE", "warning");
    }
  } catch (error) {
    showToast("Lỗi khi tải document: " + error.message, "danger");
  }
}

// ===== PROCESS SINGLE DOCUMENT =====
async function processSingleDocument(docId, retry = false) {
  const action = retry ? "chạy lại" : "xử lý";
  if (!confirm(`Bạn có chắc muốn ${action} tài liệu này?`)) return;

  const processBtn = document.querySelector(
    `button[onclick*="processSingleDocument('${docId}'"]`,
  );
  const statusElement = document.getElementById(`status-${docId}`);
  const originalBtnHtml = processBtn ? processBtn.innerHTML : "";

  try {
    if (processBtn) {
      processBtn.disabled = true;
      processBtn.innerHTML =
        '<i class="fas fa-spinner fa-spin me-1"></i>Đang xử lý...';
    }

    if (statusElement) {
      statusElement.className = "badge bg-warning";
      statusElement.textContent = "Đang xử lý";
    }

    const response = await fetch(`/api/process_document/${docId}?retry=1`, {
      method: "POST",
      headers: {
        "Content-Type": "application/json",
      },
    });

    if (response.ok) {
      const result = await response.json();
      showToast("🚀 " + result.message, "success");
      monitorSingleDocumentProgress(docId, processBtn, originalBtnHtml);
    } else {
      const error = await response.json();
      showToast(
        "❌ Lỗi: " + (error.error || "Không thể xử lý document"),
        "danger",
      );
      resetProcessButton(processBtn, originalBtnHtml, docId);
    }
  } catch (error) {
    showToast("❌ Lỗi khi xử lý document: " + error.message, "danger");
    resetProcessButton(processBtn, originalBtnHtml, docId);
  }
}

function resetProcessButton(processBtn, originalBtnHtml, docId) {
  if (processBtn) {
    processBtn.disabled = false;
    if (originalBtnHtml) {
      processBtn.innerHTML = originalBtnHtml;
    } else {
      processBtn.innerHTML = '<i class="fas fa-redo me-1"></i>Chạy lại';
    }
  }
}

function monitorSingleDocumentProgress(docId, processBtn, originalBtnHtml) {
  let checkCount = 0;
  const maxChecks = 60;

  const interval = setInterval(async () => {
    checkCount++;

    try {
      const response = await fetch(`/api/document/${docId}/status`);

      if (response.ok) {
        const result = await response.json();

        if (result.status === "completed") {
          clearInterval(interval);
          showToast(`✅ Document đã hoàn thành xử lý`, "success");
          updateDocumentStatusInList(docId, "completed");
          resetProcessButton(processBtn, originalBtnHtml, docId);
          await loadDocumentsPage();
        } else if (result.status === "error") {
          clearInterval(interval);
          showToast(`❌ Document xử lý lỗi`, "danger");
          updateDocumentStatusInList(docId, "error");
          resetProcessButton(processBtn, originalBtnHtml, docId);
        } else if (checkCount >= maxChecks) {
          clearInterval(interval);
          showToast(`⏱️ Timeout - document vẫn đang xử lý`, "warning");
          resetProcessButton(processBtn, originalBtnHtml, docId);
        }
      } else {
        clearInterval(interval);
        resetProcessButton(processBtn, originalBtnHtml, docId);
      }
    } catch (error) {
      clearInterval(interval);
      console.error("❌ Error monitoring document progress:", error);
      resetProcessButton(processBtn, originalBtnHtml, docId);
    }
  }, 2000);
}

function updateDocumentStatusInList(docId, newStatus) {
  const statusElement = document.getElementById(`status-${docId}`);
  if (statusElement) {
    statusElement.className = `badge bg-${getStatusColor(newStatus)}`;
    statusElement.textContent = getStatusText(newStatus);
  }

  const row = document.getElementById(`doc-row-${docId}`);
  if (row) {
    const actionCell = row.cells[4];
    if (actionCell) {
      actionCell.innerHTML = generateCompactActionButtons(docId, newStatus);
    }
  }

  const docIndex = currentBatchDocuments.findIndex((doc) => doc.id === docId);
  if (docIndex !== -1) {
    currentBatchDocuments[docIndex].status = newStatus;
  }
}

// ===== DOCUMENT EDITOR MODAL =====
function showDocumentEditor(docData, documentsList, documentIndex) {
  updateFilePathDisplay(docData);

  if (documentsList && documentIndex >= 0) {
    currentBatchDocuments = documentsList;
    currentDocumentIndex = documentIndex;

    document.getElementById("documentNavigation").style.display = "flex";

    const currentPagePosition =
      (currentPage - 1) * pageSize + documentIndex + 1;
    document.getElementById("currentDocPosition").textContent =
      currentPagePosition;
    document.getElementById("totalDocsInBatch").textContent =
      totalBatchDocuments;

    const canGoPrevious = currentPagePosition > 1;
    const canGoNext = currentPagePosition < totalBatchDocuments;

    document.getElementById("prevDocBtn").disabled = !canGoPrevious;
    document.getElementById("nextDocBtn").disabled = !canGoNext;
  } else {
    document.getElementById("documentNavigation").style.display = "none";
  }

  const modal = new bootstrap.Modal(document.getElementById("editorModal"));
  modal.show();

  const modalElement = document.getElementById("editorModal");
  modalElement.addEventListener(
    "shown.bs.modal",
    function onModalShown() {
      modalElement.removeEventListener("shown.bs.modal", onModalShown);
      setTimeout(() => {
        loadDocumentImages(docData);
        loadDocumentFields(docData);
      }, 100);
    },
    { once: true },
  );
}

function updateFilePathDisplay(docData) {
  const filePathText = document.getElementById("filePathText");

  if (docData && docData.filename) {
    const fileName = docData.filename;
    const fileSize = docData.file_size
      ? ` • ${(docData.file_size / 1024 / 1024).toFixed(1)}MB`
      : "";
    const totalPages = docData.total_pages
      ? ` • ${docData.total_pages} trang`
      : "";
    const documentType =
      docData.document_type && docData.document_type !== "N/A"
        ? ` • ${docData.document_type}`
        : "";

    filePathText.innerHTML = `
      <span class="fw-bold text-primary">${fileName}</span>
      <span class="text-muted">${fileSize}${totalPages}${documentType}</span>
    `;

    if (docData.file_path) {
      filePathText.title = `Đường dẫn đầy đủ: ${docData.file_path}`;
    }
  } else {
    filePathText.textContent = "Không xác định được file";
  }
}

function loadDocumentImages(docData) {
  const contentTab = document.getElementById("contentViewTab");
  if (docData.image_paths && docData.image_paths.content) {
    contentTab.style.display = "block";
  } else {
    contentTab.style.display = "none";
  }

  loadFullView(docData);
  loadHeaderView(docData);

  if (docData.image_paths && docData.image_paths.content) {
    loadContentView(docData);
  }
}

// ✅ SỬA HÀM loadFullView():
function loadFullView(docData) {
  const container = document.getElementById("fullView");

  // Show loading
  container.innerHTML =
    '<div class="text-center py-5"><i class="fas fa-spinner fa-spin fa-2x"></i></div>';

  // ✅ CHECK IF PDF
  const filePath = docData.file_path || "";
  const isPDF = filePath.toLowerCase().endsWith(".pdf");

  if (isPDF) {
    // ✅ ĐÚNG: XÂY DỰNG URL QUA HTTP
    // filePath từ DB: "uploads/batch_id/file.pdf" hoặc "d:\path\uploads\batch_id\file.pdf"

    // ✅ EXTRACT RELATIVE PATH
    let relativePath = filePath;

    // Nếu là absolute path, extract phần sau "uploads"
    if (filePath.includes("uploads")) {
      const uploadsIndex = filePath.indexOf("uploads");
      relativePath = filePath.substring(uploadsIndex);
    }

    // ✅ NORMALIZE PATH SEPARATORS (\ -> /)
    relativePath = relativePath.replace(/\\/g, "/");

    // ✅ BUILD HTTP URL
    const pdfUrl = `/${relativePath}`; // Will be: /uploads/batch_id/file.pdf

    console.log(`📄 PDF URL: ${pdfUrl}`); // Debug log

    container.innerHTML = `
      <embed 
        src="${pdfUrl}#toolbar=1" 
        type="application/pdf" 
        width="100%" 
        height="800px" 
        style="border: 1px solid #ddd; border-radius: 4px;" 
      />
    `;
  } else if (docData.image_paths && docData.image_paths.full) {
    // ✅ HIỂN THỊ IMAGE
    const img = document.createElement("img");
    img.src = `/${docData.image_paths.full}?v=${Date.now()}`;
    img.className = "img-fluid";
    img.style.maxWidth = "100%";
    img.style.height = "auto";

    container.innerHTML = "";
    container.appendChild(img);
  } else {
    container.innerHTML =
      '<div class="text-center text-muted">Không có file để hiển thị</div>';
  }
}

function loadHeaderView(docData) {
  const container = document.getElementById("headerView");
  if (!docData.image_paths || !docData.image_paths.header) {
    container.innerHTML =
      '<div class="text-center text-muted">Không có ảnh header</div>';
    return;
  }

  const img = document.createElement("img");
  img.src = `/${docData.image_paths.header}?v=${Date.now()}`;
  img.className = "img-fluid";
  img.style.maxWidth = "100%";
  img.style.height = "auto";

  container.innerHTML =
    '<div class="section-header">📋 Phần Header (Dùng cho KIE)</div>';
  container.appendChild(img);
}

function loadContentView(docData) {
  const container = document.getElementById("contentView");
  if (!docData.image_paths || !docData.image_paths.content) {
    container.innerHTML =
      '<div class="text-center text-muted">Không có ảnh nội dung</div>';
    return;
  }

  const img = document.createElement("img");
  img.src = `/${docData.image_paths.content}?v=${Date.now()}`;
  img.className = "img-fluid";
  img.style.maxWidth = "100%";
  img.style.height = "auto";

  container.innerHTML =
    '<div class="section-header">📝 Phần Nội dung (Dùng cho Tóm tắt)</div>';
  container.appendChild(img);
}

function loadDocumentFields(docData) {
  const container = document.getElementById("fieldsContainer");
  const headerData = docData.kie_header_data || {};
  const contentData = docData.kie_content_data || {};

  container.innerHTML = "";

  // ✅ HEADER SECTION (code cũ - giữ nguyên)
  const headerSection = document.createElement("div");
  headerSection.innerHTML = `
    <div class="fields-section-header">
      <i class="fas fa-header"></i>Thông tin Header
    </div>
  `;
  container.appendChild(headerSection);

  // ✅ HEADER FIELDS (code cũ - giữ nguyên)
  const fieldDefinitions = [
    {
      key: "document_type",
      label: "Loại tài liệu",
      type: "select",
      required: true,
    },
    { key: "document_number", label: "Số văn bản", type: "text" },
    { key: "document_symbol", label: "Ký hiệu VB", type: "text" },
    { key: "handwritten_number", label: "Tờ số", type: "text" },
    { key: "issue_date", label: "Ngày ban hành", type: "date" },
    { key: "issuing_authority", label: "Cơ quan ban hành", type: "text" },
    { key: "signer_name", label: "Người ký", type: "text" },
  ];

  fieldDefinitions.forEach((field) => {
    const fieldValue = headerData[field.key] || "";
    const currentValue =
      typeof fieldValue === "string"
        ? fieldValue
        : typeof fieldValue === "object" && fieldValue.value
          ? fieldValue.value
          : "";

    const fieldDiv = document.createElement("div");
    const isLongField = field.type === "select" || field.key === "summary";

    fieldDiv.className = isLongField
      ? "field-editor"
      : "field-editor inline-layout";
    fieldDiv.setAttribute("data-field-container", field.key);
    fieldDiv.setAttribute("data-source", "header");

    fieldDiv.innerHTML = `
      <label class="form-label">
        ${field.label}${
          field.required ? '<span class="text-danger">*</span>' : ""
        }
      </label>
      ${createFieldInput(field, currentValue)}
    `;

    container.appendChild(fieldDiv);
  });

  // ✅ THÊM SECTION MỚI - THÔNG TIN BỔ SUNG (SAU HEADER FIELDS)
  const documentInfoSection = document.createElement("div");
  documentInfoSection.className = "mt-3";
  documentInfoSection.innerHTML = `
    
    <!-- ✅ LOẠI HÌNH - CHECKBOX -->
    <div class="field-editor">
  <label class="form-label">
    <i class="fas fa-certificate me-1 text-primary"></i>Loại hình
  </label>
  <select 
    class="form-select form-select-sm" 
    id="documentOriginalType" 
    data-field="document_original_type"
    style="max-width: 250px;"
  >
    <option value="Bản chính" ${
      (docData.document_original_type || "Bản chính") === "Bản chính"
        ? "selected"
        : ""
    }>Bản chính</option>
    <option value="Bản sao" ${
      docData.document_original_type === "Bản sao" ? "selected" : ""
    }>Bản sao</option>
    <option value="Bản gốc" ${
      docData.document_original_type === "Bản gốc" ? "selected" : ""
    }>Bản gốc</option>
    <option value="Dự thảo" ${
      docData.document_original_type === "Dự thảo" ? "selected" : ""
    }>Dự thảo</option>
  </select>
</div>

    
    <!-- ✅ GHI CHÚ - TEXTAREA -->
    <div class="field-editor mt-3">
      <label class="form-label">
        <i class="fas fa-sticky-note me-1"></i>Ghi chú
      </label>
      <textarea 
        class="form-control field-input" 
        data-field="document_note" 
        rows="2"
        placeholder="Nhập ghi chú về tài liệu..."
        style="font-size: 13px; line-height: 1.6;"
      >${docData.document_note || ""}</textarea>
      <div class="form-text mt-1">
        <i class="fas fa-info-circle me-1"></i>
        Ghi chú về tình trạng, nguồn gốc, hoặc thông tin đặc biệt khác
      </div>
    </div>
  `;

  container.appendChild(documentInfoSection);

  // ✅ SUMMARY SECTION (code cũ - giữ nguyên)
  const summarySection = document.createElement("div");
  summarySection.className = "summary-section";

  let summaryValue = "";
  let summarySource = "";

  if (headerData.summary && headerData.has_summary) {
    summaryValue = headerData.summary;
    summarySource = "header";
  } else if (contentData.summary) {
    summaryValue = contentData.summary;
    summarySource = "content";
  }

  summarySection.innerHTML = `
    <div class="fields-section-header">
      <i class="fas fa-file-alt"></i>Trích yếu
      <small class="text-muted ms-2">(từ ${
        summarySource === "header" ? "Header" : "Nội dung"
      })</small>
    </div>
    <div class="field-editor" data-field-container="summary" data-source="${summarySource}">
      <textarea 
        class="form-control field-input" 
        data-field="summary" 
        placeholder="Nhập trích yếu..."
        style="font-size: 13px; line-height: 1.6;"
      >${summaryValue}</textarea>
      <div class="form-text mt-1">
        <i class="fas fa-info-circle me-1"></i>
        Kéo góc phải dưới để điều chỉnh kích thước
      </div>
    </div>
  `;

  container.appendChild(summarySection);

  // ✅ ACTION BUTTONS (code cũ - giữ nguyên)
  const actionSection = document.createElement("div");
  actionSection.className = "save-button-container";
  actionSection.innerHTML = `
    <div class="d-grid gap-2">
      <button class="btn btn-success btn-sm" id="saveDocBtn">
        <i class="fas fa-save me-1"></i>Lưu & Tiếp tục
      </button>
      <button class="btn btn-warning btn-sm" id="reprocessDocBtn">
        <i class="fas fa-sync-alt me-1"></i>Trích xuất lại
      </button>
      <button class="btn btn-danger btn-sm" id="deleteDocBtn">
        <i class="fas fa-trash me-1"></i>Xóa file này
      </button>
    </div>
  `;
  container.appendChild(actionSection);

  setupFieldEditorEventListeners();
}
function createFieldInput(field, value) {
  switch (field.type) {
    case "select":
      if (field.key === "document_type") {
        const documentTypes = [
          "Nghị quyết",
          "Quyết định",
          "Chỉ thị",
          "Quy chế",
          "Quy định",
          "Thông cáo",
          "Thông báo",
          "Hướng dẫn",
          "Chương trình",
          "Kế hoạch",
          "Phương án",
          "Đề án",
          "Dự án",
          "Báo cáo",
          "Tờ trình",
          "Giấy ủy quyền",
          "Phiếu gửi",
          "Phiếu chuyển",
          "Phiếu báo",
          "Biên bản",
          "Hợp đồng",
          "Công văn",
          "Công điện",
          "Bản ghi nhớ",
          "Bản thỏa thuận",
          "Giấy mời",
          "Giấy giới thiệu",
          "Giấy nghỉ phép",
          "Thư công",
          "Bản đồ",
          "Bản vẽ kỹ thuật",
          "Phiếu chi",
          "Thông tri",
          "Bảng tổng hợp",
          "Ủy nhiệm chi",
          "Bản kiểm điểm",
          "Nội quy",
          "Phiếu giới thiệu",
          "Phiếu",
          "Kết luận",
          "Nghị định",
          "Thông tư",
          "Đề nghị",
          "Khác",
        ];
        return `
          <select class="form-select field-input" data-field="${field.key}">
            <option value="">Chọn loại văn bản...</option>
            ${documentTypes
              .map(
                (type) =>
                  `<option value="${type}" ${
                    value === type ? "selected" : ""
                  }>${type}</option>`,
              )
              .join("")}
          </select>
        `;
      }
      return `<input type="text" class="form-control field-input" value="${value}" data-field="${field.key}">`;

    case "textarea":
      return `<textarea class="form-control field-input" rows="2" data-field="${
        field.key
      }" placeholder="Nhập ${field.label.toLowerCase()}...">${value}</textarea>`;

    case "date":
      return `<input type="text" class="form-control field-input" value="${value}" data-field="${field.key}" placeholder="DD/MM/YYYY">`;

    default:
      return `<input type="text" class="form-control field-input" value="${value}" data-field="${
        field.key
      }" placeholder="Nhập ${field.label.toLowerCase()}...">`;
  }
}

function setupFieldEditorEventListeners() {
  const saveBtn = document.getElementById("saveDocBtn");
  if (saveBtn) {
    saveBtn.removeEventListener("click", saveAndNavigateNext);
    saveBtn.addEventListener("click", saveAndNavigateNext);
  }

  const reprocessBtn = document.getElementById("reprocessDocBtn");
  if (reprocessBtn) {
    reprocessBtn.removeEventListener("click", reprocessCurrentDocument);
    reprocessBtn.addEventListener("click", reprocessCurrentDocument);
  }

  const deleteBtn = document.getElementById("deleteDocBtn");
  if (deleteBtn) {
    deleteBtn.removeEventListener("click", deleteCurrentDocument);
    deleteBtn.addEventListener("click", deleteCurrentDocument);
  }
}

async function saveAndNavigateNext() {
  if (!currentDocId) return;

  const saveBtn = document.getElementById("saveDocBtn");

  try {
    if (saveBtn) {
      saveBtn.disabled = true;
      saveBtn.innerHTML =
        '<i class="fas fa-spinner fa-spin me-1"></i>Đang lưu...';
    }

    await saveDocument();

    if (saveBtn) {
      saveBtn.innerHTML = '<i class="fas fa-check me-1"></i>Đã lưu!';
    }

    setTimeout(async () => {
      await navigateToNextDocumentAfterAction();
    }, 500);
  } catch (error) {
    showToast("Lỗi khi lưu: " + error.message, "danger");

    if (saveBtn) {
      saveBtn.disabled = false;
      saveBtn.innerHTML = '<i class="fas fa-save me-1"></i>Lưu & Tiếp tục';
    }
  }
}
async function saveDocument() {
  if (!currentDocId) return;

  const headerData = {};
  const contentData = currentDocument.kie_content_data || {};

  // ✅ LẤY HEADER FIELDS (code cũ - giữ nguyên)
  const headerFields = [
    "document_type",
    "document_number",
    "document_symbol",
    "handwritten_number",
    "issue_date",
    "issuing_authority",
    "signer_name",
  ];

  headerFields.forEach((fieldKey) => {
    const input = document.querySelector(
      `[data-field-container="${fieldKey}"] .field-input`,
    );
    if (input) {
      headerData[fieldKey] = input.value.trim();
    }
  });

  // ✅ LẤY SUMMARY (code cũ - giữ nguyên)
  const summaryInput = document.querySelector(
    '[data-field="summary"].field-input',
  );
  if (summaryInput) {
    const summaryContainer = summaryInput.closest(
      "[data-field-container='summary']",
    );
    const summarySource = summaryContainer.getAttribute("data-source");

    if (summarySource === "header") {
      headerData["summary"] = summaryInput.value.trim();
      headerData["has_summary"] = true;
    } else {
      contentData["summary"] = summaryInput.value.trim();
    }
  }

  // ✅ LẤY 2 FIELDS MỚI - LOẠI HÌNH VÀ GHI CHÚ
  const documentOriginalTypeSelect = document.getElementById(
    "documentOriginalType",
  );
  const documentOriginalType = documentOriginalTypeSelect
    ? documentOriginalTypeSelect.value
    : "Bản chính";

  const documentNoteTextarea = document.querySelector(
    '[data-field="document_note"]',
  );
  const documentNote = documentNoteTextarea
    ? documentNoteTextarea.value.trim()
    : "";

  try {
    const response = await fetch(`/api/document/${currentDocId}`, {
      method: "PUT",
      headers: {
        "Content-Type": "application/json",
      },
      body: JSON.stringify({
        kie_header_data: headerData,
        kie_content_data: contentData,
        document_original_type: documentOriginalType, // ✅ MỚI
        document_note: documentNote, // ✅ MỚI
      }),
    });

    if (response.ok) {
      showToast("Đã lưu thành công", "success");
      currentDocument.kie_header_data = headerData;
      currentDocument.kie_content_data = contentData;
      currentDocument.document_original_type = documentOriginalType; // ✅ MỚI
      currentDocument.document_note = documentNote; // ✅ MỚI
    } else {
      const error = await response.json();
      showToast(error.error || "Lỗi khi lưu", "danger");
    }
  } catch (error) {
    showToast("Lỗi khi lưu: " + error.message, "danger");
  }
}

async function reprocessCurrentDocument() {
  if (!currentDocId) {
    showToast("Không xác định được document để trích xuất lại", "warning");
    return;
  }

  if (
    !confirm(
      "Bạn có chắc muốn trích xuất lại document này? Dữ liệu hiện tại sẽ bị ghi đè.",
    )
  ) {
    return;
  }

  const reprocessBtn = document.getElementById("reprocessDocBtn");

  try {
    if (reprocessBtn) {
      reprocessBtn.disabled = true;
      reprocessBtn.innerHTML =
        '<i class="fas fa-spinner fa-spin me-1"></i>Đang trích xuất...';
    }

    const response = await fetch(
      `/api/process_document/${currentDocId}?retry=1`,
      {
        method: "POST",
        headers: {
          "Content-Type": "application/json",
        },
      },
    );

    if (response.ok) {
      const result = await response.json();
      showToast("🚀 " + result.message, "success");

      const modal = bootstrap.Modal.getInstance(
        document.getElementById("editorModal"),
      );
      if (modal) {
        modal.hide();
      }

      monitorReprocessProgress(currentDocId);
    } else {
      const error = await response.json();
      showToast("❌ " + (error.error || "Lỗi khi trích xuất lại"), "danger");
    }
  } catch (error) {
    showToast("❌ Lỗi khi trích xuất lại: " + error.message, "danger");
  } finally {
    if (reprocessBtn) {
      reprocessBtn.disabled = false;
      reprocessBtn.innerHTML =
        '<i class="fas fa-sync-alt me-1"></i>Trích xuất lại';
    }
  }
}

function monitorReprocessProgress(docId) {
  const interval = setInterval(async () => {
    try {
      const response = await fetch(`/api/document/${docId}/status`);
      if (response.ok) {
        const result = await response.json();

        if (result.status === "completed") {
          clearInterval(interval);
          showToast("✅ Trích xuất lại thành công!", "success");
          await loadDocumentsPage();
        } else if (result.status === "error") {
          clearInterval(interval);
          showToast("❌ Trích xuất lại thất bại", "danger");
          await loadDocumentsPage();
        }
      }
    } catch (error) {
      clearInterval(interval);
      console.error("Error monitoring reprocess progress:", error);
    }
  }, 3000);
}

async function deleteCurrentDocument() {
  if (!currentDocId) return;

  if (
    !confirm(
      `Bạn có chắc muốn xóa file "${
        currentDocument?.filename || "này"
      }" không?\n\nHành động này không thể hoàn tác!`,
    )
  ) {
    return;
  }

  const deleteBtn = document.getElementById("deleteDocBtn");

  try {
    if (deleteBtn) {
      deleteBtn.disabled = true;
      deleteBtn.innerHTML =
        '<i class="fas fa-spinner fa-spin me-1"></i>Đang xóa...';
    }

    const response = await fetch(`/api/document/${currentDocId}`, {
      method: "DELETE",
      headers: {
        "Content-Type": "application/json",
      },
    });

    if (response.ok) {
      const result = await response.json();
      showToast("🗑️ " + result.message, "success");

      await navigateToNextDocumentAfterAction();
      setTimeout(() => {
        loadDocumentsPage();
      }, 1000);
    } else {
      const error = await response.json();
      throw new Error(error.error || "Lỗi khi xóa document");
    }
  } catch (error) {
    showToast("❌ " + error.message, "danger");

    if (deleteBtn) {
      deleteBtn.disabled = false;
      deleteBtn.innerHTML = '<i class="fas fa-trash me-1"></i>Xóa file này';
    }
  }
}

async function navigateToNextDocumentAfterAction() {
  const globalPosition =
    (currentPage - 1) * pageSize + currentDocumentIndex + 1;

  if (globalPosition >= totalBatchDocuments) {
    showToast("Đã hoàn thành tất cả documents trong batch!", "success");

    const modal = bootstrap.Modal.getInstance(
      document.getElementById("editorModal"),
    );
    if (modal) {
      modal.hide();
    }
    return;
  }

  try {
    if (currentDocumentIndex + 1 < currentBatchDocuments.length) {
      const nextDoc = currentBatchDocuments[currentDocumentIndex + 1];
      await loadAndShowDocument(nextDoc.id, currentDocumentIndex + 1);
    } else {
      if (currentPage < totalPages) {
        await loadNextPageAndNavigate();
      } else {
        showToast("Đã đến document cuối cùng", "info");

        const modal = bootstrap.Modal.getInstance(
          document.getElementById("editorModal"),
        );
        if (modal) {
          modal.hide();
        }
      }
    }
  } catch (error) {
    showToast(
      "Lỗi khi chuyển sang document tiếp theo: " + error.message,
      "danger",
    );
  }
}

async function navigateToPreviousDocument() {
  const globalPosition =
    (currentPage - 1) * pageSize + currentDocumentIndex + 1;

  if (globalPosition <= 1) {
    showToast("Đã đến document đầu tiên", "info");
    return;
  }

  if (currentDocumentIndex > 0) {
    const prevDoc = currentBatchDocuments[currentDocumentIndex - 1];
    await loadAndShowDocument(prevDoc.id, currentDocumentIndex - 1);
  } else {
    if (currentPage > 1) {
      await loadPreviousPageAndNavigate();
    } else {
      showToast("Đã đến document đầu tiên", "info");
    }
  }
}

async function navigateToNextDocument() {
  const globalPosition =
    (currentPage - 1) * pageSize + currentDocumentIndex + 1;

  if (globalPosition >= totalBatchDocuments) {
    showToast("Đã đến document cuối cùng", "info");
    return;
  }

  if (currentDocumentIndex + 1 < currentBatchDocuments.length) {
    const nextDoc = currentBatchDocuments[currentDocumentIndex + 1];
    await loadAndShowDocument(nextDoc.id, currentDocumentIndex + 1);
  } else {
    if (currentPage < totalPages) {
      await loadNextPageAndNavigate();
    } else {
      showToast("Đã đến document cuối cùng", "info");
    }
  }
}

async function loadAndShowDocument(docId, newIndex) {
  try {
    const response = await fetch(`/api/document/${docId}`);
    const docData = await response.json();

    currentDocumentIndex = newIndex;
    currentDocId = docId;
    currentDocument = docData;

    updateFilePathDisplay(docData);

    const globalPosition = (currentPage - 1) * pageSize + newIndex + 1;

    document.getElementById("currentDocPosition").textContent = globalPosition;
    document.getElementById("totalDocsInBatch").textContent =
      totalBatchDocuments;

    const canGoPrevious = globalPosition > 1;
    const canGoNext = globalPosition < totalBatchDocuments;

    document.getElementById("prevDocBtn").disabled = !canGoPrevious;
    document.getElementById("nextDocBtn").disabled = !canGoNext;

    loadDocumentImages(docData);
    loadDocumentFields(docData);
  } catch (error) {
    showToast("Lỗi khi tải document: " + error.message, "danger");
  }
}

async function loadNextPageAndNavigate() {
  try {
    showToast("Đang chuyển sang trang tiếp theo...", "info");

    currentPage++;
    await loadDocumentsPage();

    if (currentBatchDocuments.length > 0) {
      const firstDoc = currentBatchDocuments[0];
      await loadAndShowDocument(firstDoc.id, 0);
      showToast("Đã chuyển sang trang tiếp theo", "success");
    } else {
      showToast("Trang tiếp theo không có document", "warning");
      currentPage--;
    }
  } catch (error) {
    showToast("Lỗi khi chuyển trang: " + error.message, "danger");
    currentPage--;
  }
}

async function loadPreviousPageAndNavigate() {
  try {
    showToast("Đang chuyển sang trang trước...", "info");

    currentPage--;
    await loadDocumentsPage();

    if (currentBatchDocuments.length > 0) {
      const lastIndex = currentBatchDocuments.length - 1;
      const lastDoc = currentBatchDocuments[lastIndex];
      await loadAndShowDocument(lastDoc.id, lastIndex);
      showToast("Đã chuyển sang trang trước", "success");
    } else {
      showToast("Trang trước không có document", "warning");
      currentPage++;
    }
  } catch (error) {
    showToast("Lỗi khi chuyển trang: " + error.message, "danger");
    currentPage++;
  }
}

function switchView(viewType) {
  currentView = viewType;

  document
    .querySelectorAll("#viewTabs .nav-link")
    .forEach((tab) => tab.classList.remove("active"));
  document.getElementById(viewType + "ViewTab").classList.add("active");

  document
    .querySelectorAll(".image-view")
    .forEach((view) => view.classList.remove("active"));
  document.getElementById(viewType + "View").classList.add("active");
}

// ===== HELPER FUNCTIONS =====
function getStatusColor(status) {
  const colors = {
    pending: "secondary",
    uploading: "warning",
    uploaded: "info",
    processing: "warning",
    completed: "success",
    error: "danger",
  };
  return colors[status] || "secondary";
}

function getStatusText(status) {
  const texts = {
    pending: "Chờ xử lý",
    uploading: "Đang upload",
    uploaded: "Đã upload",
    processing: "Đang xử lý",
    completed: "Hoàn thành",
    error: "Lỗi",
  };
  return texts[status] || status;
}

function showToast(message, type = "info") {
  const toast = document.createElement("div");
  toast.className = `alert alert-${type} alert-dismissible fade show position-fixed`;
  toast.style.top = "20px";
  toast.style.right = "20px";
  toast.style.zIndex = "9999";
  toast.innerHTML = `
    ${message}
    <button type="button" class="btn-close" data-bs-dismiss="alert"></button>
  `;
  document.body.appendChild(toast);

  setTimeout(() => {
    if (toast && toast.parentNode) {
      toast.remove();
    }
  }, 4000);
}

function toggleSidebar() {
  document.body.classList.toggle("sidebar-closed");
}
