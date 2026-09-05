let currentBatchId = null;
let currentDocId = null;
let fieldDefinitions = [];
let currentDocument = null;
let currentView = "full";
let originalCutPoint = 0;
let isDragging = false;
let isAuthenticating = false;

async function checkAuthentication() {
  if (isAuthenticating) {
    return; // Prevent multiple simultaneous auth checks
  }

  isAuthenticating = true;

  try {
    const response = await fetch("/api/auth/me", {
      credentials: "include",
    });

    if (response.ok) {
      const user = await response.json();
      updateUserInfo(user);
      isAuthenticating = false;
      return true;
    } else if (response.status === 401) {
      // Only redirect if we're not already on login page
      if (!window.location.pathname.includes("/login")) {
        window.location.href = "/login";
      }
      isAuthenticating = false;
      return false;
    }
  } catch (error) {
    console.error("Auth check failed:", error);
    isAuthenticating = false;
    return false;
  }
}

function updateUserInfo(user) {
  document.getElementById("userName").textContent = user.username || user.email;
  document.getElementById("userInfo").style.display = "block";
  document.getElementById("logoutBtn").style.display = "block";
}

// ===== SIDEBAR TOGGLE =====
function toggleSidebar() {
  document.body.classList.toggle("sidebar-closed");
}

async function authenticatedFetch(url, options = {}) {
  try {
    const response = await fetch(url, options);

    if (response.status === 401) {
      // Token expired hoặc chưa đăng nhập
      window.location.href = "/login";
      return null;
    }

    return response;
  } catch (error) {
    console.error("Request failed:", error);
    throw error;
  }
}

// Load data khi trang load
document.addEventListener("DOMContentLoaded", async function () {
  await checkAuthentication();
  setupBatchFilters();
  loadBatches();
  loadFieldDefinitions();
  if (window.BroadcastChannel) {
    const channel = new BroadcastChannel("batch-updates");
    channel.onmessage = function (event) {
      if (event.data.action === "refresh-batches") {
        console.log("📡 Received batch refresh signal");
        loadBatches();
      }
    };
  }

  // ✅ AUTO REFRESH EVERY 10 SECONDS
  setInterval(() => {
    loadBatches();
  }, 10000);

  // Upload form events
  document
    .getElementById("fileInput")
    .addEventListener("change", updateFileList);
  document.getElementById("uploadBtn").addEventListener("click", uploadFiles);
});

function showToast(message, type = "info") {
  const toast = document.createElement("div");
  toast.className = `alert alert-${type} alert-dismissible fade show position-fixed`;
  toast.style.top = "20px";
  toast.style.right = "20px";
  toast.style.zIndex = "9999"; // ✅ TĂNG Z-INDEX
  toast.innerHTML = `
        ${message}
        <button type="button" class="btn-close" data-bs-dismiss="alert"></button>
      `;
  document.body.appendChild(toast);

  // ✅ AUTO REMOVE TOAST
  setTimeout(() => {
    if (toast && toast.parentNode) {
      toast.remove();
    }
  }, 4000); // Giảm từ 5s xuống 4s
}

function updateProgress(current, total, text) {
  const percent = ((current / total) * 100).toFixed(1);
  document.querySelector(".progress-bar").style.width = percent + "%";
  document.querySelector(".progress-bar").textContent = percent + "%";
  document.querySelector("#progress-text").textContent = text;
}
let batchCurrentPage = 1;
let batchPageSize = 10;
let batchCurrentSearch = "";
let batchCurrentStatusFilter = "";
let batchTotalPages = 1;
let batchTotalItems = 0;

// ✅ CẬP NHẬT loadBatches() ĐỂ SỬ DỤNG PAGINATION
async function loadBatches() {
  try {
    const params = new URLSearchParams({
      page: batchCurrentPage,
      page_size: batchPageSize,
      search: batchCurrentSearch,
      status: batchCurrentStatusFilter,
    });

    const response = await authenticatedFetch(`/api/batches?${params}`);
    const data = await response.json();

    // Update pagination state
    batchTotalPages = data.pagination.total_pages;
    batchTotalItems = data.pagination.total_items;

    // Render batches table
    renderBatchesTable(data.batches);

    // Update pagination info if pagination controls exist
    updateBatchPaginationInfo(data.pagination);
    renderBatchPaginationControls(data.pagination);
  } catch (error) {
    showToast("Lỗi khi tải danh sách batch: " + error.message, "danger");
  }
}
async function createPackageFromBatch(batchId, batchName) {
  try {
    // Kiểm tra số mảnh khả dụng từ batch này
    const response = await fetch("/api/available-pieces-count-by-batches", {
      method: "POST",
      headers: {
        "Content-Type": "application/json",
      },
      body: JSON.stringify({
        batch_mode: "specific",
        selected_batch_ids: [batchId],
      }),
    });

    const data = await response.json();

    if (!data.success) {
      showToast("Lỗi khi kiểm tra mảnh khả dụng: " + data.error, "danger");
      return;
    }

    if (data.count === 0) {
      showToast("Batch này không có mảnh nào khả dụng để tạo gói", "warning");
      return;
    }

    // Redirect đến trang tạo gói với batch đã chọn
    const createUrl = `/create-package-page?batch_id=${batchId}&batch_name=${encodeURIComponent(
      batchName,
    )}&available_count=${data.count}`;
    window.location.href = createUrl;
  } catch (error) {
    showToast("Lỗi khi tạo gói: " + error.message, "danger");
  }
}

async function stopBatchProcessing(batchId) {
  if (!confirm("Bạn có chắc muốn dừng quá trình trích xuất của batch này?")) {
    return;
  }

  try {
    const processBtn = document.getElementById(`process-btn-${batchId}`);
    if (processBtn) {
      processBtn.disabled = true;
      processBtn.innerHTML =
        '<i class="fas fa-spinner fa-spin me-1"></i>Đang dừng...';
    }

    // ✅ BƯỚC 1: RESET BATCH STATUS VÀ DOCUMENTS (KHÔNG CẦN JOB_ID)
    console.log(`🔄 Step 1: Resetting batch ${batchId} status...`);

    const resetResponse = await fetch(`/api/batch/${batchId}/reset-for-stop`, {
      method: "POST",
      headers: { "Content-Type": "application/json" },
    });

    const resetData = await resetResponse.json();

    if (!resetData.success) {
      throw new Error(resetData.error || "Không thể reset batch");
    }

    console.log(
      `✅ Step 1 complete: Reset ${resetData.documents_reset} documents`,
    );
    showToast(`✅ Đã reset ${resetData.documents_reset} documents`, "info");

    // ✅ BƯỚC 2: TÌM VÀ DỪNG JOB ĐANG CHẠY CHO BATCH NÀY
    console.log(`🔄 Step 2: Finding and stopping job for batch ${batchId}...`);

    const jobsResponse = await fetch("/api/parallel/jobs");
    const jobsData = await jobsResponse.json();

    if (!jobsData.success) {
      throw new Error("Không thể lấy danh sách jobs");
    }

    // Tìm job đang chạy cho batch này
    const activeJob = jobsData.jobs.find((job) => {
      return (
        job.status === "processing" &&
        job.batch_ids &&
        job.batch_ids.includes(batchId)
      );
    });

    if (activeJob) {
      console.log(`🛑 Found active job ${activeJob.job_id}, stopping...`);

      const stopJobResponse = await fetch(
        `/api/parallel/stop-job/${activeJob.job_id}`,
        {
          method: "POST",
          headers: { "Content-Type": "application/json" },
        },
      );

      const stopJobData = await stopJobResponse.json();

      if (stopJobData.success) {
        console.log(`✅ Step 2 complete: Job ${activeJob.job_id} stopped`);
        showToast(`✅ Đã dừng job thành công`, "success");
      } else {
        console.warn(`⚠️ Could not stop job: ${stopJobData.error}`);
        showToast(`⚠️ ${stopJobData.error}`, "warning");
      }
    } else {
      console.log(`ℹ️ No active job found for batch ${batchId}`);
      showToast(`ℹ️ Không tìm thấy job đang chạy cho batch này`, "info");
    }

    // ✅ BƯỚC 3: REFRESH BATCH LIST
    console.log(`🔄 Step 3: Refreshing batch list...`);
    setTimeout(() => {
      loadBatches();
      console.log(`✅ Step 3 complete: Batch list refreshed`);
    }, 1000);
  } catch (error) {
    console.error("❌ Error in stopBatchProcessing:", error);
    showToast(`❌ Lỗi: ${error.message}`, "danger");
  } finally {
    // Reset button state
    const processBtn = document.getElementById(`process-btn-${batchId}`);
    if (processBtn) {
      processBtn.disabled = false;
      processBtn.innerHTML = '<i class="fas fa-redo me-1"></i>Chạy lại';
    }
  }
}

// ✅ EXPORT BATCH TO EXCEL FUNCTION
async function exportBatchToExcel(batchId, batchName) {
  try {
    // Show loading toast
    showToast(`📊 Đang xuất batch "${batchName}" sang Excel...`, "info");

    // Call API to export
    const response = await fetch(`/api/batch/${batchId}/export-excel`, {
      method: "GET",
    });

    if (!response.ok) {
      const error = await response.json();
      throw new Error(error.error || "Không thể export batch");
    }

    // Get file from response
    const blob = await response.blob();

    // Get filename from Content-Disposition header
    const contentDisposition = response.headers.get("Content-Disposition");
    let filename = `${batchName}_export.xlsx`;

    if (contentDisposition) {
      const filenameMatch = contentDisposition.match(/filename="?(.+)"?/);
      if (filenameMatch && filenameMatch[1]) {
        filename = filenameMatch[1];
      }
    }

    // Create download link
    const url = window.URL.createObjectURL(blob);
    const a = document.createElement("a");
    a.href = url;
    a.download = filename;
    document.body.appendChild(a);
    a.click();

    // Cleanup
    window.URL.revokeObjectURL(url);
    document.body.removeChild(a);

    showToast(`✅ Đã xuất thành công: ${filename}`, "success");
  } catch (error) {
    console.error("Export error:", error);
    showToast(`❌ Lỗi khi export: ${error.message}`, "danger");
  }
}
function renderBatchesTable(batches) {
  const tbody = document.querySelector("#batchTable tbody");

  if (batches.length === 0) {
    tbody.innerHTML = `
      <tr>
        <td colspan="7" class="text-center text-muted py-4">
          <i class="fas fa-inbox fa-2x mb-2"></i><br>
          Không tìm thấy batch nào
        </td>
      </tr>
    `;
    return;
  }

  tbody.innerHTML = batches
    .map((batch) => {
      const isProcessing = batch.status === "processing";

      // ✅ LOGIC NÚT MỚI: Start/Stop toggle
      let actionButton = "";
      if (isProcessing) {
        actionButton = `
          <button class="btn btn-sm btn-outline-danger" onclick="stopBatchProcessing('${batch.id}')" title="Dừng trích xuất" id="process-btn-${batch.id}">
            <i class="fas fa-stop me-1"></i>Dừng
          </button>
        `;
      } else {
        const buttonText =
          batch.status === "completed" ? "Chạy lại" : "Trích xuất";
        const buttonIcon = batch.status === "completed" ? "fa-redo" : "fa-play";

        actionButton = `
          <button class="btn btn-sm btn-outline-success" onclick="processBatch('${batch.id}', true)" title="${buttonText}" id="process-btn-${batch.id}">
            <i class="fas ${buttonIcon} me-1"></i>${buttonText}
          </button>
        `;
      }

      // ✅ KIỂM TRA XEM CÓ THỂ TẠO GÓI KHÔNG
      const canCreatePackage =
        batch.status === "completed" && batch.processed_files > 0;

      return `
        <tr>
          <td>${batch.name}</td>
          <td>${new Date(batch.created_at).toLocaleDateString("vi-VN")}</td>
          <td>${batch.total_files}</td>
          <td>${batch.processed_files}</td>
          <td>
            <span class="badge bg-${getStatusColor(
              batch.status,
            )}">${getStatusText(batch.status)}</span>
          </td>
          <td>
            <div class="btn-group" role="group">
              <button class="btn btn-sm btn-outline-primary" onclick="viewBatch('${
                batch.id
              }')" title="Xem documents">
                <i class="fas fa-list me-1"></i>
              </button>

              <!-- ✅ NÚT TẠO GÓI -->
              ${
                canCreatePackage
                  ? `
                <button class="btn btn-sm btn-outline-info" onclick="createPackageFromBatch('${batch.id}', '${batch.name}', ${batch.processed_files})" title="Tạo gói từ batch này">
                  <i class="fas fa-box me-1"></i>
                </button>
              `
                  : `
                <button class="btn btn-sm btn-outline-secondary disabled" title="Chưa có dữ liệu để tạo gói">
                  <i class="fas fa-box me-1"></i>
                </button>
              `
              }

              <!-- ✅ NÚT EXPORT EXCEL MỚI -->
              ${
                batch.processed_files > 0
                  ? `
                <button class="btn btn-sm btn-outline-success" onclick="exportBatchToExcel('${batch.id}', '${batch.name}')" title="Export sang Excel">
                  <i class="fas fa-file-excel me-1"></i>
                </button>
              `
                  : `
                <button class="btn btn-sm btn-outline-secondary disabled" title="Chưa có dữ liệu để export">
                  <i class="fas fa-file-excel me-1"></i>
                </button>
              `
              }

              <!-- ✅ NÚT START/STOP DYNAMIC -->
              ${actionButton}

              <button class="btn btn-sm btn-outline-danger" onclick="deleteBatch('${
                batch.id
              }')" title="Xóa batch">
                <i class="fas fa-trash me-1"></i>
              </button>
            </div>
          </td>
        </tr>
      `;
    })
    .join("");
}

async function loadFieldDefinitions() {
  try {
    const response = await fetch("/api/field_definitions");
    fieldDefinitions = await response.json();
  } catch (error) {
    console.error("Error loading field definitions:", error);
  }
}
function goToBatchPage(page) {
  if (page < 1 || page > batchTotalPages) return;
  batchCurrentPage = page;
  loadBatches();
}

function updateBatchPaginationInfo(pagination) {
  const start = (pagination.current_page - 1) * pagination.page_size + 1;
  const end = Math.min(
    pagination.current_page * pagination.page_size,
    pagination.total_items,
  );

  const showingInfo = document.getElementById("batchShowingInfo");
  if (showingInfo) {
    showingInfo.textContent = `Hiển thị ${start}-${end} của ${pagination.total_items} batches`;
  }

  // Update page size select
  const pageSizeSelect = document.getElementById("batchPageSizeSelect");
  if (pageSizeSelect) {
    pageSizeSelect.value = pagination.page_size;
  }
}
function setupBatchFilters() {
  // Tạo search và filter controls cho batch table
  const batchTableContainer =
    document.querySelector("#batchTable").parentElement;

  // Tạo filter controls container
  const filtersHTML = `
        <div class="row mb-3" id="batchFilters">
          <div class="col-md-6">
            <div class="input-group">
              <span class="input-group-text"><i class="fas fa-search"></i></span>
              <input type="text" class="form-control" id="batchSearchInput" placeholder="Tìm kiếm batch...">
            </div>
          </div>
          <div class="col-md-3">
            <select class="form-select" id="batchStatusFilter">
              <option value="">Tất cả trạng thái</option>
              <option value="uploaded">Đã upload</option>
              <option value="processing">Đang xử lý</option>
              <option value="completed">Hoàn thành</option>
              <option value="error">Lỗi</option>
            </select>
          </div>
          <div class="col-md-3">
            <select class="form-select" id="batchPageSizeSelect">
              <option value="10">10 batches/trang</option>
              <option value="20">20 batches/trang</option>
              <option value="50">50 batches/trang</option>
            </select>
          </div>
        </div>
      `;

  // Insert filters before batch table
  batchTableContainer.insertAdjacentHTML("beforebegin", filtersHTML);

  // Setup event listeners với debounce
  let searchTimeout;
  document
    .getElementById("batchSearchInput")
    .addEventListener("input", function () {
      clearTimeout(searchTimeout);
      searchTimeout = setTimeout(() => {
        batchCurrentSearch = this.value.trim();
        batchCurrentPage = 1;
        loadBatches();
      }, 500);
    });

  // Status filter
  document
    .getElementById("batchStatusFilter")
    .addEventListener("change", function () {
      batchCurrentStatusFilter = this.value;
      batchCurrentPage = 1;
      loadBatches();
    });

  // Page size change
  document
    .getElementById("batchPageSizeSelect")
    .addEventListener("change", function () {
      batchPageSize = parseInt(this.value);
      batchCurrentPage = 1;
      loadBatches();
    });
}

// ✅ RENDER BATCH PAGINATION CONTROLS
function renderBatchPaginationControls(pagination) {
  let paginationEl = document.getElementById("batchPagination");

  // Tạo pagination container nếu chưa có
  if (!paginationEl) {
    const batchTableContainer =
      document.querySelector("#batchTable").parentElement;
    const paginationHTML = `
          <div class="d-flex justify-content-between align-items-center mt-3" id="batchPaginationContainer">
            <div>
              <small class="text-muted" id="batchShowingInfo"></small>
            </div>
            <nav>
              <ul class="pagination pagination-sm mb-0" id="batchPagination">
              </ul>
            </nav>
          </div>
        `;
    batchTableContainer.insertAdjacentHTML("afterend", paginationHTML);
    paginationEl = document.getElementById("batchPagination");
  }

  if (pagination.total_pages <= 1) {
    paginationEl.innerHTML = "";
    document.getElementById("batchShowingInfo").textContent =
      `Tổng cộng ${pagination.total_items} batches`;
    return;
  }

  let html = "";

  // Previous button
  html += `
        <li class="page-item ${!pagination.has_prev ? "disabled" : ""}">
          <a class="page-link" href="#" onclick="goToBatchPage(${
            pagination.current_page - 1
          })" ${!pagination.has_prev ? 'tabindex="-1"' : ""}>
            <i class="fas fa-chevron-left"></i>
          </a>
        </li>
      `;

  // Page numbers - hiển thị tối đa 5 pages
  const startPage = Math.max(1, pagination.current_page - 2);
  const endPage = Math.min(pagination.total_pages, startPage + 4);

  // First page link nếu startPage > 1
  if (startPage > 1) {
    html += `
          <li class="page-item">
            <a class="page-link" href="#" onclick="goToBatchPage(1)">1</a>
          </li>
        `;
    if (startPage > 2) {
      html += `<li class="page-item disabled"><span class="page-link">...</span></li>`;
    }
  }

  // Main page numbers
  for (let i = startPage; i <= endPage; i++) {
    html += `
          <li class="page-item ${
            pagination.current_page === i ? "active" : ""
          }">
            <a class="page-link" href="#" onclick="goToBatchPage(${i})">${i}</a>
          </li>
        `;
  }

  // Last page link nếu endPage < total_pages
  if (endPage < pagination.total_pages) {
    if (endPage < pagination.total_pages - 1) {
      html += `<li class="page-item disabled"><span class="page-link">...</span></li>`;
    }
    html += `
          <li class="page-item">
            <a class="page-link" href="#" onclick="goToBatchPage(${pagination.total_pages})">${pagination.total_pages}</a>
          </li>
        `;
  }

  // Next button
  html += `
        <li class="page-item ${!pagination.has_next ? "disabled" : ""}">
          <a class="page-link" href="#" onclick="goToBatchPage(${
            pagination.current_page + 1
          })" ${!pagination.has_next ? 'tabindex="-1"' : ""}>
            <i class="fas fa-chevron-right"></i>
          </a>
        </li>
      `;

  paginationEl.innerHTML = html;
}

function showProcessingIndicator(show) {
  const indicator = document.getElementById("processingIndicator");
  indicator.style.display = show ? "block" : "none";
}

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

// Các function khác như updateFileList, uploadFiles, processBatch, etc. giữ nguyên
function updateFileList() {
  const files = document.getElementById("fileInput").files;
  const fileList = document.getElementById("fileList");

  if (files.length === 0) {
    fileList.innerHTML = "";
    return;
  }

  const supportedExts = [".pdf"];
  const validFiles = Array.from(files).filter((file) =>
    supportedExts.some((ext) => file.name.toLowerCase().endsWith(ext)),
  );

  fileList.innerHTML = `
                   <div class="alert alert-info">
                       <strong>Đã chọn:</strong> ${
                         validFiles.length
                       } file hợp lệ / ${files.length} file tổng
                   </div>
                   <div class="list-group" style="max-height: 200px; overflow-y: auto;">
                       ${validFiles
                         .slice(0, 10)
                         .map(
                           (file) =>
                             `<div class="list-group-item">${file.name}</div>`,
                         )
                         .join("")}
                       ${
                         validFiles.length > 10
                           ? `<div class="list-group-item text-muted">... và ${
                               validFiles.length - 10
                             } file khác</div>`
                           : ""
                       }
                   </div>
                 `;
}
let uploadController = null;
let isUploadCancelled = false;
let uploadStats = {
  totalFiles: 0,
  currentIndex: 0,
  successCount: 0,
  failCount: 0,
  totalSizeMB: 0,
  failedFiles: [],
};
// ✅ FUNCTION uploadFiles() - ĐẦY ĐỦ VỚI RETRY LOGIC

async function uploadFiles() {
  const fileInput = document.getElementById("fileInput");
  const files = Array.from(fileInput.files);

  if (files.length === 0) {
    showToast("Vui lòng chọn files để upload", "warning");
    return;
  }

  const supportedExts = [".pdf", ".png", ".jpg", ".jpeg", ".bmp", ".tiff"];
  const validFiles = files.filter((file) =>
    supportedExts.some((ext) => file.name.toLowerCase().endsWith(ext)),
  );

  if (validFiles.length === 0) {
    showToast("Không có file hợp lệ để upload", "warning");
    return;
  }

  // Initialize stats
  uploadStats = {
    totalFiles: validFiles.length,
    currentIndex: 0,
    successCount: 0,
    failCount: 0,
    totalSizeMB: validFiles.reduce(
      (total, file) => total + file.size / 1024 / 1024,
      0,
    ),
    failedFiles: [],
    currentBatchId: null,
  };

  uploadController = new AbortController();
  isUploadCancelled = false;

  const progressModal = new bootstrap.Modal(
    document.getElementById("progressModal"),
  );
  progressModal.show();
  initializeUploadProgress();

  bootstrap.Modal.getInstance(document.getElementById("uploadModal")).hide();

  const batchName =
    document.getElementById("batchName").value ||
    `Batch_${new Date().toLocaleString("vi-VN").replace(/[/:]/g, "_")}`;

  const batchId = generateUUID();

  try {
    // ✅ BƯỚC 1: TẠO BATCH TRƯỚC VỚI RETRY
    let batchCreated = false;
    let batchRetries = 0;
    const maxBatchRetries = 2;

    while (!batchCreated && batchRetries <= maxBatchRetries) {
      try {
        const createBatchResponse = await fetch("/api/batch/create", {
          method: "POST",
          headers: { "Content-Type": "application/json" },
          body: JSON.stringify({
            batch_id: batchId,
            batch_name: batchName,
            status: "uploading",
          }),
        });

        if (createBatchResponse.ok) {
          const batchData = await createBatchResponse.json();
          if (batchData.success) {
            batchCreated = true;
            uploadStats.currentBatchId = batchId;
            addUploadLog(`📦 Đã tạo batch: ${batchName}`, "success");
          } else {
            throw new Error(batchData.error || "Cannot create batch");
          }
        } else {
          throw new Error(`HTTP ${createBatchResponse.status}`);
        }
      } catch (batchError) {
        batchRetries++;

        if (batchRetries <= maxBatchRetries) {
          addUploadLog(
            `⚠️ Lỗi tạo batch (lần ${batchRetries}), retry sau 2s...`,
            "warning",
          );
          await new Promise((resolve) => setTimeout(resolve, 2000));
        } else {
          addUploadLog(
            `❌ Không thể tạo batch sau ${maxBatchRetries} lần retry`,
            "danger",
          );
          showToast(`❌ Lỗi tạo batch: ${batchError.message}`, "danger");
          showUploadCompleted(false);
          return;
        }
      }
    }

    if (!batchCreated) {
      showToast("❌ Không thể tạo batch, hủy upload", "danger");
      showUploadCompleted(false);
      return;
    }

    // ✅ BƯỚC 2: UPLOAD TỪNG FILE VỚI RETRY (max 2 retries mỗi file)
    addUploadLog(
      `🚀 Bắt đầu upload ${validFiles.length} files (mỗi file retry tối đa 2 lần)...`,
      "info",
    );

    for (let i = 0; i < validFiles.length; i++) {
      if (isUploadCancelled) {
        addUploadLog("⚠️ User đã hủy upload", "warning");
        break;
      }

      uploadStats.currentIndex = i + 1;
      const file = validFiles[i];

      updateCurrentFileName(
        file.name,
        `${(file.size / 1024 / 1024).toFixed(1)} MB`,
      );
      addUploadLog(
        `📤 [${i + 1}/${validFiles.length}] Đang upload ${file.name}...`,
        "info",
      );

      // ✅ PREPARE FORMDATA
      const formData = new FormData();
      formData.append("batch_id", batchId);
      formData.append("file", file);

      // ✅ UPLOAD VỚI RETRY (maxRetries = 2)
      const success = await uploadSingleFileWithProgress(
        formData,
        file.name,
        2,
      );

      if (success) {
        uploadStats.successCount++;
      } else {
        uploadStats.failCount++;
        uploadStats.failedFiles.push({
          filename: file.name,
          error: "Upload failed after 2 retries",
          file: file, // ✅ LƯU FILE OBJECT ĐỂ CÓ THỂ RETRY LẠI
        });
      }

      // Update UI
      updateOverallProgress(i + 1, validFiles.length);
      document.getElementById("successCount").textContent =
        uploadStats.successCount;
      document.getElementById("failCount").textContent = uploadStats.failCount;
    }

    // ✅ BƯỚC 3: KIỂM TRA KẾT QUẢ VÀ XỬ LÝ BATCH
    if (uploadStats.successCount === 0) {
      // ❌ KHÔNG CÓ FILE NÀO THÀNH CÔNG - XÓA BATCH
      addUploadLog("❌ Không có file nào upload thành công", "danger");

      try {
        await fetch(`/api/batch/${batchId}`, { method: "DELETE" });
        addUploadLog("🗑️ Đã xóa batch rỗng", "warning");
      } catch (deleteError) {
        console.error("Error deleting empty batch:", deleteError);
      }

      showToast(
        `❌ Không thể upload file nào sau nhiều lần retry.\nBatch đã bị xóa.`,
        "danger",
      );
      showUploadCompleted(false);

      // Show failed files detail
      if (uploadStats.failedFiles.length > 0) {
        showFailedFiles(uploadStats.failedFiles);
      }

      return;
    }

    // ✅ CÓ ÍT NHẤT 1 FILE THÀNH CÔNG - FINALIZE BATCH
    await fetch(`/api/batch/${batchId}/finalize`, {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify({
        total_files: uploadStats.successCount,
        status: "uploaded",
      }),
    });

    addUploadLog(
      `✅ Batch finalized: ${uploadStats.successCount}/${validFiles.length} files thành công`,
      "success",
    );

    showUploadCompleted(true);

    // ✅ HIỂN THỊ KẾT QUẢ
    const summaryMsg =
      `📊 Kết quả upload:\n` +
      `✅ Thành công: ${uploadStats.successCount}/${validFiles.length}\n` +
      `❌ Thất bại (sau retry): ${uploadStats.failCount}/${validFiles.length}`;

    showToast(summaryMsg, uploadStats.failCount > 0 ? "warning" : "success");

    if (uploadStats.failedFiles.length > 0) {
      showFailedFiles(uploadStats.failedFiles);
    }

    loadBatches();
  } catch (error) {
    console.error("Upload error:", error);
    addUploadLog(`❌ Lỗi hệ thống: ${error.message}`, "danger");
    showToast(`❌ Lỗi: ${error.message}`, "danger");
    showUploadCompleted(false);
  }
}

async function retryFailedFile(fileIndex) {
  const failedFile = uploadStats.failedFiles[fileIndex];

  if (!failedFile || !failedFile.file) {
    showToast("Không tìm thấy thông tin file để retry", "warning");
    return;
  }

  const retryBtn = document.getElementById(`retry-btn-${fileIndex}`);
  const failedFileItem = document.getElementById(`failed-file-${fileIndex}`);

  if (!retryBtn || !failedFileItem) return;

  try {
    // ✅ SHOW LOADING STATE
    retryBtn.disabled = true;
    retryBtn.innerHTML =
      '<i class="fas fa-spinner fa-spin me-1"></i>Đang tải...';
    failedFileItem.classList.remove("list-group-item-danger");
    failedFileItem.classList.add("list-group-item-warning");

    addUploadLog(`🔄 Retry upload: ${failedFile.filename}`, "info");

    // ✅ PREPARE FORMDATA
    const formData = new FormData();
    formData.append("batch_id", uploadStats.currentBatchId);
    formData.append("file", failedFile.file);

    // ✅ RETRY UPLOAD WITH PROGRESS
    const success = await uploadSingleFileWithProgress(
      formData,
      failedFile.filename,
      2, // max 2 retries
    );

    if (success) {
      // ✅ SUCCESS - UPDATE UI
      uploadStats.successCount++;
      uploadStats.failCount--;

      // Remove from failed list
      uploadStats.failedFiles.splice(fileIndex, 1);

      // Update stats display
      document.getElementById("successCount").textContent =
        uploadStats.successCount;
      document.getElementById("failCount").textContent = uploadStats.failCount;

      // Update file item
      failedFileItem.classList.remove("list-group-item-warning");
      failedFileItem.classList.add("list-group-item-success");
      failedFileItem.innerHTML = `
        <div class="d-flex justify-content-between align-items-center">
          <div class="flex-grow-1">
            <div class="fw-bold">${failedFile.filename}</div>
            <div class="text-success">✅ Upload thành công sau retry</div>
          </div>
          <i class="fas fa-check-circle text-success"></i>
        </div>
      `;

      addUploadLog(`✅ ${failedFile.filename} - retry thành công`, "success");

      // ✅ AUTO HIDE ERROR SECTION NẾU KHÔNG CÒN FILE LỖI
      if (uploadStats.failedFiles.length === 0) {
        setTimeout(() => {
          document.getElementById("errorSection").style.display = "none";
          showToast("🎉 Tất cả files đã upload thành công!", "success");
        }, 2000);
      }
    } else {
      // ❌ STILL FAILED AFTER RETRY
      retryBtn.disabled = false;
      retryBtn.innerHTML = '<i class="fas fa-redo me-1"></i>Thử lại';
      failedFileItem.classList.remove("list-group-item-warning");
      failedFileItem.classList.add("list-group-item-danger");

      addUploadLog(`❌ ${failedFile.filename} - vẫn lỗi sau retry`, "danger");
      showToast(`❌ ${failedFile.filename} vẫn không thể upload`, "danger");
    }
  } catch (error) {
    console.error("Error retrying file:", error);

    // ✅ RESET BUTTON STATE ON ERROR
    retryBtn.disabled = false;
    retryBtn.innerHTML = '<i class="fas fa-redo me-1"></i>Thử lại';
    failedFileItem.classList.remove("list-group-item-warning");
    failedFileItem.classList.add("list-group-item-danger");

    addUploadLog(
      `❌ Lỗi retry ${failedFile.filename}: ${error.message}`,
      "danger",
    );
    showToast(`❌ Lỗi khi retry: ${error.message}`, "danger");
  }
}
// ✅ HELPER FUNCTION
function generateUUID() {
  return "xxxxxxxx-xxxx-4xxx-yxxx-xxxxxxxxxxxx".replace(/[xy]/g, function (c) {
    const r = (Math.random() * 16) | 0;
    const v = c === "x" ? r : (r & 0x3) | 0x8;
    return v.toString(16);
  });
}

async function uploadSingleFileWithProgress(
  formData,
  filename,
  maxRetries = 2,
) {
  let attempt = 0;

  while (attempt <= maxRetries) {
    try {
      const success = await attemptUpload(formData, filename);

      if (success) {
        // ✅ SUCCESS
        if (attempt > 0) {
          addUploadLog(
            `✅ ${filename} - thành công sau ${attempt} lần retry`,
            "success",
          );
        }
        return true;
      }

      // ❌ FAILED - RETRY IF ATTEMPTS LEFT
      attempt++;

      if (attempt <= maxRetries) {
        const waitTime = 1000 * attempt; // Exponential backoff: 1s, 2s
        addUploadLog(
          `🔄 ${filename} - lỗi lần ${attempt}, retry sau ${
            waitTime / 1000
          }s...`,
          "warning",
        );

        await new Promise((resolve) => setTimeout(resolve, waitTime));
      } else {
        addUploadLog(
          `❌ ${filename} - thất bại sau ${maxRetries} lần retry`,
          "danger",
        );
        return false;
      }
    } catch (error) {
      attempt++;

      if (attempt <= maxRetries) {
        const waitTime = 1000 * attempt;
        addUploadLog(
          `⚠️ ${filename} - lỗi network, retry sau ${waitTime / 1000}s...`,
          "warning",
        );
        await new Promise((resolve) => setTimeout(resolve, waitTime));
      } else {
        addUploadLog(
          `❌ ${filename} - lỗi network sau ${maxRetries} lần retry`,
          "danger",
        );
        return false;
      }
    }
  }

  return false;
}
function attemptUpload(formData, filename) {
  return new Promise((resolve) => {
    const xhr = new XMLHttpRequest();

    xhr.upload.addEventListener("progress", (event) => {
      if (event.lengthComputable) {
        const percent = (event.loaded / event.total) * 100;
        updateCurrentFileProgress(percent);
      }
    });

    xhr.addEventListener("load", () => {
      if (xhr.status === 200) {
        try {
          const response = JSON.parse(xhr.responseText);
          resolve(response.success === true);
        } catch (e) {
          console.error(`Parse error for ${filename}:`, e);
          resolve(false);
        }
      } else {
        console.error(`Upload failed for ${filename}: HTTP ${xhr.status}`);
        resolve(false);
      }
    });

    xhr.addEventListener("error", () => resolve(false));
    xhr.addEventListener("abort", () => resolve(false));

    uploadController.signal.addEventListener("abort", () => xhr.abort());

    xhr.open("POST", "/api/upload_single_file");
    xhr.send(formData);
  });
}
async function uploadWithProgress(formData, files) {
  return new Promise((resolve, reject) => {
    const xhr = new XMLHttpRequest();

    // Setup progress tracking
    xhr.upload.addEventListener("progress", (event) => {
      if (event.lengthComputable) {
        const percentComplete = (event.loaded / event.total) * 100;
        updateCurrentFileProgress(percentComplete);
      }
    });

    // Setup response handlers
    xhr.addEventListener("load", () => {
      if (xhr.status === 200) {
        try {
          const result = JSON.parse(xhr.responseText);
          handleUploadSuccess(result);
          resolve(result);
        } catch (error) {
          reject(new Error("Invalid response format"));
        }
      } else {
        try {
          const error = JSON.parse(xhr.responseText);
          reject(new Error(error.error || `HTTP ${xhr.status}`));
        } catch {
          reject(new Error(`Upload failed with status ${xhr.status}`));
        }
      }
    });

    xhr.addEventListener("error", () => {
      reject(new Error("Network error during upload"));
    });

    xhr.addEventListener("abort", () => {
      addUploadLog("⚠️ Upload đã bị hủy", "warning");
      showUploadCompleted(false);
    });

    // Setup abort signal
    uploadController.signal.addEventListener("abort", () => {
      xhr.abort();
    });

    // Start upload
    addUploadLog(`🚀 Bắt đầu upload ${files.length} files...`, "info");
    updateCurrentFileName("Đang upload toàn bộ batch...");

    xhr.open("POST", "/api/upload_folder");
    xhr.send(formData);
  });
}
function handleUploadSuccess(result) {
  uploadStats.successCount = result.total_files || 0;
  uploadStats.failCount = result.failed_count || 0;
  uploadStats.failedFiles = result.failed_files || [];

  // Update overall progress
  updateOverallProgress(
    uploadStats.successCount + uploadStats.failCount,
    uploadStats.totalFiles,
  );
  updateCurrentFileProgress(100);
  updateCurrentFileName(
    "Upload hoàn thành!",
    `${result.total_size_mb || 0} MB`,
  );

  // Update stats
  document.getElementById("successCount").textContent =
    uploadStats.successCount;
  document.getElementById("failCount").textContent = uploadStats.failCount;

  // Add success log
  addUploadLog(
    `✅ Upload hoàn thành: ${uploadStats.successCount} thành công, ${uploadStats.failCount} thất bại`,
    "success",
  );

  // Show failed files if any
  if (uploadStats.failedFiles.length > 0) {
    showFailedFiles(uploadStats.failedFiles);
  }

  // Show completion state
  showUploadCompleted(true);

  // Refresh batch list
  loadBatches();

  // Show toast
  showToast(result.message || "Upload thành công!", "success");
}
function showUploadCompleted(success) {
  const progressBar = document.getElementById("overallProgressBar");

  // Stop animation
  progressBar.classList.remove("progress-bar-animated");

  // Update UI state
  document.getElementById("progressCloseBtn").style.display = "block";
  document.getElementById("uploadingFooter").style.display = "none";
  document.getElementById("completedFooter").style.display = "block";

  // Update modal title
  const modalTitle = document.querySelector("#progressModal .modal-title");
  modalTitle.innerHTML = success
    ? '<i class="fas fa-check-circle text-success me-2"></i>Upload hoàn thành'
    : '<i class="fas fa-exclamation-triangle text-warning me-2"></i>Upload không hoàn thành';
}

// ✅ CANCEL UPLOAD
function cancelUpload() {
  if (uploadController && !isUploadCancelled) {
    isUploadCancelled = true;
    uploadController.abort();

    addUploadLog("⚠️ Đang hủy upload...", "warning");
  }
}

// ✅ TOGGLE UPLOAD LOG
function toggleUploadLog() {
  const logContainer = document.getElementById("uploadLog");
  const toggleBtn = document.getElementById("toggleLogBtn");

  if (logContainer.style.display === "none") {
    logContainer.style.display = "block";
    toggleBtn.innerHTML = '<i class="fas fa-chevron-up"></i> Ẩn chi tiết';
  } else {
    logContainer.style.display = "none";
    toggleBtn.innerHTML = '<i class="fas fa-chevron-down"></i> Xem chi tiết';
  }
}
function showFailedFiles(failedFiles) {
  const errorSection = document.getElementById("errorSection");
  const errorList = document.getElementById("errorList");

  errorList.innerHTML = "";

  failedFiles.forEach((file, index) => {
    const errorItem = document.createElement("div");
    errorItem.className = "list-group-item list-group-item-danger small";
    errorItem.id = `failed-file-${index}`;
    errorItem.innerHTML = `
      <div class="d-flex justify-content-between align-items-center">
        <div class="flex-grow-1">
          <div class="fw-bold">${file.filename}</div>
          <div class="text-muted">${file.error}</div>
        </div>
        <button class="btn btn-sm btn-outline-warning" 
                onclick="retryFailedFile(${index})"
                title="Thử upload lại file này"
                id="retry-btn-${index}">
          <i class="fas fa-redo me-1"></i>Tải lại
        </button>
      </div>
    `;
    errorList.appendChild(errorItem);
  });

  errorSection.style.display = "block";
}
// ✅ INITIALIZE UPLOAD PROGRESS UI
function initializeUploadProgress() {
  // Reset progress bars
  updateOverallProgress(0, uploadStats.totalFiles);
  updateCurrentFileProgress(0);

  // Reset stats
  document.getElementById("successCount").textContent = "0";
  document.getElementById("failCount").textContent = "0";
  document.getElementById("totalSize").textContent =
    `${uploadStats.totalSizeMB.toFixed(1)} MB`;

  // Clear logs
  document.getElementById("uploadLog").innerHTML =
    '<div class="small text-muted">Bắt đầu upload...</div>';

  // Reset UI state
  document.getElementById("progressCloseBtn").style.display = "none";
  document.getElementById("uploadingFooter").style.display = "block";
  document.getElementById("completedFooter").style.display = "none";
  document.getElementById("errorSection").style.display = "none";
}

// ✅ UPDATE PROGRESS FUNCTIONS
function updateOverallProgress(current, total) {
  const percent = total > 0 ? Math.round((current / total) * 100) : 0;

  document.getElementById("overallProgress").textContent =
    `${current}/${total} files`;
  document.getElementById("overallProgressBar").style.width = `${percent}%`;
  document.getElementById("overallProgressBar").textContent = `${percent}%`;

  // Update progress bar color based on completion
  const progressBar = document.getElementById("overallProgressBar");
  progressBar.className = "progress-bar progress-bar-striped";

  if (current === total && total > 0) {
    progressBar.classList.add("bg-success");
  } else if (current > 0) {
    progressBar.classList.add("progress-bar-animated");
  }
}

function updateCurrentFileProgress(percent) {
  const progressBar = document.getElementById("currentFileProgressBar");
  const roundedPercent = Math.round(percent);

  progressBar.style.width = `${roundedPercent}%`;
  progressBar.textContent = `${roundedPercent}%`;
}

function updateCurrentFileName(filename, fileSize = "") {
  document.getElementById("currentFileName").textContent = filename;
  document.getElementById("currentFileSize").textContent = fileSize;
}

function addUploadLog(message, type = "info") {
  const logContainer = document.getElementById("uploadLog");
  const timestamp = new Date().toLocaleTimeString("vi-VN");

  const logEntry = document.createElement("div");
  logEntry.className = `small mb-1 text-${type === "info" ? "muted" : type}`;
  logEntry.innerHTML = `<span class="text-muted">[${timestamp}]</span> ${message}`;

  logContainer.appendChild(logEntry);
  logContainer.scrollTop = logContainer.scrollHeight;
}
// Cập nhật phần xóa batch trong frontend
async function deleteBatch(batchId) {
  if (
    !confirm(
      "Bạn có chắc chắn muốn xóa batch này? Hành động này không thể hoàn tác.",
    )
  ) {
    return;
  }

  try {
    const response = await fetch(`/api/batch/${batchId}`, {
      method: "DELETE",
      headers: {
        "Content-Type": "application/json",
      },
    });

    const data = await response.json();

    if (response.ok) {
      alert(data.message || "Batch đã được xóa thành công");
      // ✅ ALWAYS refresh batches on success
      loadBatches();
    } else {
      // ✅ HANDLE SPECIFIC ERROR CASES
      if (response.status === 404) {
        console.warn("Batch không tồn tại, có thể đã bị xóa trước đó");
        alert("Batch không tồn tại hoặc đã bị xóa. Đang refresh danh sách...");
        // Still refresh to update UI
        loadBatches();
      } else if (response.status === 403) {
        alert("Bạn không có quyền xóa batch này");
      } else {
        console.error("Error deleting batch:", data);
        alert(`Lỗi khi xóa batch: ${data.error || "Unknown error"}`);
      }
    }
  } catch (error) {
    console.error("Network error deleting batch:", error);
    alert("Lỗi kết nối khi xóa batch. Vui lòng thử lại.");
  }
}
async function processBatch(batchId, retry = false) {
  if (!batchId) {
    showToast("Vui lòng chọn batch để xử lý", "warning");
    return;
  }

  try {
    // Show loading state on button
    const processBtn = document.querySelector(`button[onclick*="${batchId}"]`);
    if (processBtn) {
      processBtn.disabled = true;
      processBtn.innerHTML =
        '<i class="fas fa-spinner fa-spin me-1"></i>Đang khởi động...';
    }

    // ✅ START PARALLEL PROCESSING với default settings từ .env
    const response = await fetch("/api/parallel/start-extraction", {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify({
        batch_ids: [batchId],
        // Không cần specify max_workers, timeout - sẽ dùng default từ .env
      }),
    });

    const data = await response.json();

    if (data.success) {
      // ✅ BẮT ĐẦU MONITOR PROGRESS NGẦM (không modal)
      startBatchProgressMonitoring(batchId);
    } else {
      throw new Error(data.error || "Không thể bắt đầu xử lý");
    }
  } catch (error) {
    console.error("Error starting parallel processing:", error);
    showToast("❌ Lỗi: " + error.message, "danger");

    // Reset button state
    const processBtn = document.querySelector(`button[onclick*="${batchId}"]`);
    if (processBtn) {
      processBtn.disabled = false;
      processBtn.innerHTML = '<i class="fas fa-play me-1"></i>Trích xuất';
    }
  }
}
function startBatchProgressMonitoring(batchId) {
  let lastProcessedCount = 0;

  const interval = setInterval(async () => {
    try {
      const response = await fetch(`/api/batch/${batchId}/status`);
      const status = await response.json();

      // Update button state based on batch status
      const processBtn = document.getElementById(`process-btn-${batchId}`);

      if (processBtn) {
        if (status.status === "processing") {
          // ✅ ĐANG PROCESSING - HIỂN THỊ NÚT STOP
          processBtn.disabled = false;
          processBtn.innerHTML = '<i class="fas fa-stop me-1"></i>Dừng';
          processBtn.onclick = () => stopBatchProcessing(batchId);
          processBtn.className = "btn btn-sm btn-outline-danger";
          processBtn.title = "Dừng trích xuất";
        } else {
          // ✅ KHÔNG PROCESSING - HIỂN THỊ NÚT START
          processBtn.disabled = false;

          if (status.status === "completed") {
            processBtn.innerHTML = '<i class="fas fa-redo me-1"></i>Chạy lại';
            processBtn.title = "Chạy lại";
          } else {
            processBtn.innerHTML = '<i class="fas fa-play me-1"></i>Trích xuất';
            processBtn.title = "Trích xuất";
          }

          processBtn.onclick = () => processBatch(batchId, true);
          processBtn.className = "btn btn-sm btn-outline-success";
        }
      }

      // Show progress toast nếu có tiến độ mới
      if (status.processed_files > lastProcessedCount) {
        lastProcessedCount = status.processed_files;
        showToast(
          `📊 Batch ${batchId}: ${status.processed_files}/${status.total_files} completed`,
          "info",
        );
      }

      // ✅ STOP MONITORING KHI HOÀN THÀNH HOẶC DỪNG
      if (
        status.status === "completed" ||
        status.status === "error" ||
        status.status === "uploaded"
      ) {
        clearInterval(interval);

        if (status.status === "completed") {
          showToast(
            `✅ Batch đã hoàn thành: ${status.processed_files}/${status.total_files} files`,
            "success",
          );
        } else if (status.status === "error") {
          showToast(
            `❌ Batch gặp lỗi: ${status.processed_files}/${status.total_files} completed`,
            "danger",
          );
        } else if (status.status === "uploaded") {
          showToast(`⏹️ Batch đã được dừng`, "warning");
        }

        // Refresh batches list
        loadBatches();
      }
    } catch (error) {
      clearInterval(interval);
      console.error("Error monitoring batch progress:", error);
      showToast("❌ Lỗi monitor progress: " + error.message, "danger");
    }
  }, 3000); // Check every 3 seconds
}
async function viewBatch(batchId) {
  // Redirect to dedicated page
  window.location.href = `/batch/${batchId}/documents`;
}
let currentPage = 1;
let pageSize = 10;
let currentSearch = "";
let currentStatusFilter = "";
let totalPages = 1;
let totalItems = 0;

let currentBatchDocuments = []; // Documents của trang hiện tại
let totalBatchDocuments = 0; // ✅ THÊM: Tổng số documents trong batch

function renderPaginationControls(pagination) {
  const paginationEl = document.getElementById("pagination");

  if (pagination.total_pages <= 1) {
    paginationEl.innerHTML = "";
    return;
  }

  let html = "";

  // Previous button
  html += `
        <li class="page-item ${!pagination.has_prev ? "disabled" : ""}">
          <a class="page-link" href="#" onclick="goToPage(${
            pagination.current_page - 1
          })" ${!pagination.has_prev ? 'tabindex="-1"' : ""}>
            <i class="fas fa-chevron-left"></i>
          </a>
        </li>
      `;

  // Page numbers
  const startPage = Math.max(1, pagination.current_page - 2);
  const endPage = Math.min(pagination.total_pages, startPage + 4);

  for (let i = startPage; i <= endPage; i++) {
    html += `
          <li class="page-item ${
            pagination.current_page === i ? "active" : ""
          }">
            <a class="page-link" href="#" onclick="goToPage(${i})">${i}</a>
          </li>
        `;
  }

  // Next button
  html += `
        <li class="page-item ${!pagination.has_next ? "disabled" : ""}">
          <a class="page-link" href="#" onclick="goToPage(${
            pagination.current_page + 1
          })" ${!pagination.has_next ? 'tabindex="-1"' : ""}>
            <i class="fas fa-chevron-right"></i>
          </a>
        </li>
      `;

  paginationEl.innerHTML = html;
}
function renderDocumentsTable(documents) {
  const tbody = document.getElementById("documentsTableBody");

  if (!tbody) return;

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
      // ✅ LẤY DOCUMENT_TYPE TỪ KIE_HEADER_DATA THAY VÌ DATABASE
      let documentType = "N/A";

      try {
        if (doc.kie_header_data && doc.kie_header_data.document_type) {
          documentType = doc.kie_header_data.document_type;
        } else if (typeof doc.kie_header_data === "string") {
          // Nếu kie_header_data là string, parse JSON
          const kieData = JSON.parse(doc.kie_header_data);
          documentType = kieData.document_type || "N/A";
        }
      } catch (error) {
        // Nếu parse lỗi hoặc không có KIE data, fallback to database
        documentType = doc.document_type || "N/A";
        console.warn(`Cannot parse KIE data for document ${doc.id}:`, error);
      }

      // ✅ TẠO TOOLTIP VỚI THÔNG TIN CHI TIẾT - SỬ DỤNG KIE DOCUMENT TYPE
      const tooltipInfo = [
        `File: ${doc.filename}`,
        `Loại: ${documentType}`, // ✅ DÙNG KIE DOCUMENT TYPE
        `Số trang: ${doc.total_pages || 1}`,
        `Trạng thái: ${getStatusText(doc.status)}`,
        `Ngày tạo: ${
          doc.created_at
            ? new Date(doc.created_at).toLocaleString("vi-VN")
            : "N/A"
        }`,
      ].join("\n");

      // ✅ THÊM VISUAL INDICATOR CHO DOCUMENT TYPE SOURCE
      const typeSource =
        documentType !== "N/A" && documentType !== (doc.document_type || "N/A")
          ? "kie"
          : "db";
      const typeClass = typeSource === "kie" ? "text-primary" : "text-muted";
      const typeTitle =
        typeSource === "kie"
          ? "Loại văn bản từ KIE (đã trích xuất)"
          : "Loại văn bản từ database hoặc chưa có KIE";

      return `
        <tr id="doc-row-${doc.id}">
          <td class="filename-cell">
            <div class="filename-tooltip" data-full-filename="${
              doc.filename
            }" title="${tooltipInfo}">
              <i class="fas fa-file-pdf text-danger me-1"></i>
              <span class="filename-text">${doc.filename}</span>
            </div>
          </td>
          <td>
            <small class="${typeClass}" title="${typeTitle}">
              ${documentType}
              ${
                typeSource === "kie"
                  ? '<i class="fas fa-check-circle text-success ms-1" title="Từ KIE"></i>'
                  : ""
              }
            </small>
          </td>
          <td class="text-center">
            <span class="badge bg-info text-dark" title="${
              doc.total_pages || 1
            } trang PDF">
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

function filterDocuments() {
  const searchTerm = document
    .getElementById("searchDocuments")
    .value.toLowerCase();
  const statusFilter = document.getElementById("statusFilter").value;

  filteredDocuments = totalDocuments.filter((doc) => {
    const matchesSearch = doc.filename.toLowerCase().includes(searchTerm);
    const matchesStatus = !statusFilter || doc.status === statusFilter;
    return matchesSearch && matchesStatus;
  });

  currentPage = 1; // Reset to first page
  renderDocumentsPage();
}

// ✅ RENDER DOCUMENTS PAGE
function renderDocumentsPage() {
  const tbody = document.getElementById("documentsTableBody");
  const start = (currentPage - 1) * pageSize;
  const end = start + pageSize;
  const pageDocuments = filteredDocuments.slice(start, end);

  tbody.innerHTML = pageDocuments
    .map(
      (doc) => `
        <tr id="doc-row-${doc.id}">
          <td>
            <div class="text-truncate" style="max-width: 200px;" title="${
              doc.filename
            }">
              <i class="fas fa-file-pdf text-danger me-1"></i>
              ${doc.filename}
            </div>
          </td>
          <td>
            <small class="text-muted">${doc.document_type || "N/A"}</small>
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
      `,
    )
    .join("");

  // Update pagination info
  updatePaginationInfo();
  renderPagination();
}
function generateCompactActionButtons(docId, status) {
  let buttons = '<div class="btn-group btn-group-sm" role="group">';

  // Edit button
  if (status === "completed") {
    buttons += `
          <button class="btn btn-outline-primary btn-sm" onclick="editDocument('${docId}')" title="Chỉnh sửa">
            <i class="fas fa-edit"></i>
          </button>`;
  }

  // ✅ REPROCESS BUTTON VỚI CONSISTENT HTML
  if (status !== "uploading") {
    const iconClass =
      status === "processing" ? "fa-sync-alt fa-spin" : "fa-redo";
    const titleText = status === "processing" ? "Đang xử lý..." : "Chạy lại";
    const disabled = status === "processing" ? "disabled" : "";

    buttons += `
          <button class="btn btn-outline-warning btn-sm ${disabled}"
                  onclick="processSingleDocument('${docId}', true)"
                  title="${titleText}">
            <i class="fas ${iconClass} me-1"></i>${
              status === "processing" ? "Đang xử lý" : "Chạy lại"
            }
          </button>`;
  }

  // View/Edit for processing/error
  if (status === "processing" || status === "error") {
    buttons += `
          <button class="btn btn-outline-secondary btn-sm" onclick="tryEditDocument('${docId}')" title="Xem/Sửa">
            <i class="fas fa-eye"></i>
          </button>`;
  }

  // Status check for processing
  if (status === "processing") {
    buttons += `
          <button class="btn btn-outline-info btn-sm" onclick="checkDocumentStatus('${docId}')" title="Kiểm tra">
            <i class="fas fa-search"></i>
          </button>`;
  }

  buttons += "</div>";
  return buttons;
}

// ✅ UPDATE PAGINATION INFO
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

  const pageSizeSelect = document.getElementById("pageSizeSelect");
  if (pageSizeSelect) {
    pageSizeSelect.value = pagination.page_size;
  }
}

// ✅ RENDER PAGINATION
function renderPagination() {
  const totalPages = Math.ceil(filteredDocuments.length / pageSize);
  const pagination = document.getElementById("pagination");

  if (totalPages <= 1) {
    pagination.innerHTML = "";
    return;
  }

  let paginationHTML = "";

  // Previous button
  paginationHTML += `
        <li class="page-item ${currentPage === 1 ? "disabled" : ""}">
          <a class="page-link" href="#" onclick="goToPage(${
            currentPage - 1
          })" ${currentPage === 1 ? 'tabindex="-1"' : ""}>
            <i class="fas fa-chevron-left"></i>
          </a>
        </li>
      `;

  // Page numbers (show max 5 pages)
  let startPage = Math.max(1, currentPage - 2);
  let endPage = Math.min(totalPages, startPage + 4);

  if (endPage - startPage < 4) {
    startPage = Math.max(1, endPage - 4);
  }

  for (let i = startPage; i <= endPage; i++) {
    paginationHTML += `
          <li class="page-item ${currentPage === i ? "active" : ""}">
            <a class="page-link" href="#" onclick="goToPage(${i})">${i}</a>
          </li>
        `;
  }

  // Next button
  paginationHTML += `
        <li class="page-item ${currentPage === totalPages ? "disabled" : ""}">
          <a class="page-link" href="#" onclick="goToPage(${
            currentPage + 1
          })" ${currentPage === totalPages ? 'tabindex="-1"' : ""}>
            <i class="fas fa-chevron-right"></i>
          </a>
        </li>
      `;

  pagination.innerHTML = paginationHTML;
}

// ✅ PAGINATION FUNCTIONS
function goToPage(page) {
  if (page < 1 || page > totalPages) return;

  currentPage = page;
  loadDocumentsPage();
}

function changePageSize() {
  pageSize = parseInt(document.getElementById("pageSizeSelect").value);
  currentPage = 1; // Reset to first page
  renderDocumentsPage();
}
async function processSingleDocument(docId, retry = false) {
  const action = retry ? "chạy lại" : "xử lý";
  if (!confirm(`Bạn có chắc muốn ${action} tài liệu này?`)) return;

  // ✅ TÌM BUTTON CHÍNH XÁC THEO DOC ID
  const processBtn = document.querySelector(
    `button[onclick*="processSingleDocument('${docId}'"]`,
  );
  const statusElement = document.getElementById(`status-${docId}`);
  const originalBtnHtml = processBtn ? processBtn.innerHTML : "";

  try {
    // ✅ SHOW LOADING STATE ON BUTTON
    if (processBtn) {
      processBtn.disabled = true;
      processBtn.innerHTML =
        '<i class="fas fa-spinner fa-spin me-1"></i>Đang xử lý...';
    }

    // ✅ UPDATE STATUS BADGE TO PROCESSING
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

      // ✅ START MONITORING VỚI PROPER CLEANUP
      monitorSingleDocumentProgress(docId, processBtn, originalBtnHtml);
    } else {
      const error = await response.json();
      showToast(
        "❌ Lỗi: " + (error.error || "Không thể xử lý document"),
        "danger",
      );

      // ✅ RESET BUTTON IMMEDIATELY ON ERROR
      resetProcessButton(processBtn, originalBtnHtml, docId);
    }
  } catch (error) {
    showToast("❌ Lỗi khi xử lý document: " + error.message, "danger");

    // ✅ RESET BUTTON ON NETWORK ERROR
    resetProcessButton(processBtn, originalBtnHtml, docId);
  }
}
function resetProcessButton(processBtn, originalBtnHtml, docId) {
  if (processBtn) {
    processBtn.disabled = false;

    // ✅ RESTORE ORIGINAL HTML OR DEFAULT
    if (originalBtnHtml) {
      processBtn.innerHTML = originalBtnHtml;
    } else {
      // ✅ FALLBACK DEFAULT BUTTON HTML
      processBtn.innerHTML = '<i class="fas fa-redo me-1"></i>Chạy lại';
    }

    console.log(`🔄 Reset process button for document ${docId}`);
  }
}
// ✅ CẬP NHẬT FUNCTION monitorSingleDocumentProgress VỚI AUTO STOP

function monitorSingleDocumentProgress(docId, processBtn, originalBtnHtml) {
  let checkCount = 0;
  const maxChecks = 60; // 60 * 2s = 2 minutes timeout

  const interval = setInterval(async () => {
    checkCount++;

    try {
      const response = await fetch(`/api/document/${docId}/status`);

      if (response.ok) {
        const result = await response.json();
        console.log(
          `🔍 Document ${docId} status: ${result.status} (check ${checkCount})`,
        );

        // ✅ UPDATE STATUS IN LIST
        updateDocumentStatusInList(docId, result.status);

        // ✅ CHECK IF PROCESSING IS COMPLETE
        if (result.status === "completed") {
          clearInterval(interval);
          showToast("✅ Document đã xử lý xong!", "success");

          // ✅ RESET BUTTON TO NORMAL STATE
          resetProcessButton(processBtn, originalBtnHtml, docId);

          // ✅ REFRESH DOCUMENTS LIST
          setTimeout(() => {
            loadDocumentsPage();
          }, 1000);
        } else if (result.status === "error") {
          clearInterval(interval);
          showToast("❌ Document xử lý lỗi", "danger");

          // ✅ RESET BUTTON TO NORMAL STATE
          resetProcessButton(processBtn, originalBtnHtml, docId);
        } else if (result.status !== "processing") {
          // ✅ UNEXPECTED STATUS - STOP MONITORING
          clearInterval(interval);
          console.warn(`⚠️ Unexpected status: ${result.status}`);

          resetProcessButton(processBtn, originalBtnHtml, docId);
        }

        // ✅ TIMEOUT PROTECTION
        if (checkCount >= maxChecks) {
          clearInterval(interval);
          showToast(
            "⏰ Timeout monitoring document. Vui lòng kiểm tra lại.",
            "warning",
          );

          resetProcessButton(processBtn, originalBtnHtml, docId);
        }
      } else {
        // ✅ API ERROR - STOP MONITORING
        clearInterval(interval);
        console.error(`❌ Status check failed: ${response.status}`);

        resetProcessButton(processBtn, originalBtnHtml, docId);
      }
    } catch (error) {
      // ✅ NETWORK ERROR - STOP MONITORING
      clearInterval(interval);
      console.error("❌ Error monitoring document progress:", error);

      resetProcessButton(processBtn, originalBtnHtml, docId);
    }
  }, 2000); // Check every 2 seconds
}
async function checkDocumentStatus(docId) {
  try {
    const response = await fetch(`/api/document/${docId}/status`);
    if (response.ok) {
      const result = await response.json();
      showToast(`Trạng thái: ${getStatusText(result.status)}`, "info");
      updateDocumentStatusInList(docId, result.status);
    } else {
      showToast("Lỗi khi kiểm tra trạng thái", "danger");
    }
  } catch (error) {
    showToast("Lỗi khi kiểm tra trạng thái: " + error.message, "danger");
  }
}
function updateDocumentStatusInList(docId, newStatus) {
  // ✅ UPDATE STATUS BADGE
  const statusElement = document.getElementById(`status-${docId}`);
  if (statusElement) {
    statusElement.className = `badge bg-${getStatusColor(newStatus)}`;
    statusElement.textContent = getStatusText(newStatus);
  }

  // ✅ UPDATE ACTION BUTTONS IN THE ROW
  const row = document.getElementById(`doc-row-${docId}`);
  if (row) {
    const actionCell = row.cells[4]; // Action buttons column (0-indexed)
    if (actionCell) {
      actionCell.innerHTML = generateCompactActionButtons(docId, newStatus);
    }
  }

  // ✅ UPDATE DOCUMENT IN CURRENT BATCH DOCUMENTS ARRAY
  const docIndex = currentBatchDocuments.findIndex((doc) => doc.id === docId);
  if (docIndex !== -1) {
    currentBatchDocuments[docIndex].status = newStatus;
  }

  console.log(`✅ Updated document ${docId} status to: ${newStatus}`);
}

function updateDocumentInArrays(docId, newStatus) {
  // Update in totalDocuments
  const totalIndex = totalDocuments.findIndex((doc) => doc.id === docId);
  if (totalIndex !== -1) {
    totalDocuments[totalIndex].status = newStatus;
  }

  // Update in filteredDocuments
  const filteredIndex = filteredDocuments.findIndex((doc) => doc.id === docId);
  if (filteredIndex !== -1) {
    filteredDocuments[filteredIndex].status = newStatus;
  }
}
function generateActionButtons(docId, status) {
  let buttons = '<div class="btn-group" role="group">';

  // ✅ EDIT BUTTON: Hiển thị cho cả completed VÀ processing (đã có data)
  if (status === "completed") {
    buttons += `<button class="btn btn-sm btn-primary" onclick="editDocument('${docId}')" title="Chỉnh sửa">
                 <i class="fas fa-edit"></i>
               </button>`;
  }

  // ✅ LUÔN HIỂN THỊ NÚT CHẠY LẠI (trừ uploading)
  if (status !== "uploading") {
    const iconClass =
      status === "processing" ? "fa-sync-alt fa-spin" : "fa-redo";
    const titleText = status === "processing" ? "Force restart" : "Chạy lại";

    buttons += `<button class="btn btn-sm btn-warning" onclick="processSingleDocument('${docId}', true)" title="${titleText}">
                 <i class="fas ${iconClass}"></i>
               </button>`;
  }

  // ✅ THÊM EDIT BUTTON CHO DOCUMENTS CÓ KIE DATA (kể cả đang processing)
  // Sẽ check trong editDocument() function xem có data không
  if (status === "processing" || status === "error") {
    buttons += `<button class="btn btn-sm btn-secondary" onclick="tryEditDocument('${docId}')" title="Xem/Sửa (nếu có dữ liệu)">
                 <i class="fas fa-eye"></i>
               </button>`;
  }

  // Status check button cho processing
  if (status === "processing") {
    buttons += `<button class="btn btn-sm btn-info" onclick="checkDocumentStatus('${docId}')" title="Kiểm tra trạng thái">
                 <i class="fas fa-spinner fa-spin"></i>
               </button>`;
  }

  buttons += "</div>";
  return buttons;
}

function monitorDocumentProgress(docId) {
  const interval = setInterval(async () => {
    try {
      const response = await fetch(`/api/document/${docId}/status`);
      if (response.ok) {
        const result = await response.json();
        updateDocumentStatusInList(docId, result.status);

        // Auto refresh buttons khi status thay đổi
        const row = document.getElementById(`doc-row-${docId}`);
        if (row) {
          const actionCell = row.cells[3];
          actionCell.innerHTML = generateActionButtons(docId, result.status);
        }

        if (result.status === "completed" || result.status === "error") {
          clearInterval(interval);
          showToast(
            `Tài liệu ${result.filename} đã ${
              result.status === "completed" ? "hoàn thành" : "lỗi"
            }`,
            result.status === "completed" ? "success" : "danger",
          );

          // ✅ REFRESH BATCH STATUS SAU KHI DOCUMENT HOÀN THÀNH
          await refreshBatchStatusAfterDocumentChange();

          // Refresh batch list để cập nhật counter
          loadBatches();
        }
      }
    } catch (error) {
      clearInterval(interval);
      console.error("Error monitoring document progress:", error);
    }
  }, 2000);
}
async function refreshBatchStatusAfterDocumentChange() {
  if (!currentBatchId) return;

  try {
    await fetch(`/api/batch/${currentBatchId}/refresh-status`, {
      method: "POST",
    });
    console.log(`✅ Refreshed batch ${currentBatchId} status`);
  } catch (error) {
    console.error("Error refreshing batch status:", error);
  }
}
async function refreshDocumentsList() {
  try {
    // Show loading state cho refresh button
    const refreshBtn = document.querySelector(
      'button[onclick="refreshDocumentsList()"]',
    );
    if (refreshBtn) {
      refreshBtn.innerHTML =
        '<i class="fas fa-spinner fa-spin me-1"></i>Đang tải...';
      refreshBtn.disabled = true;
    }

    await loadDocumentsPage();
    showToast("Đã làm mới danh sách", "success");
  } catch (error) {
    showToast("Lỗi khi làm mới: " + error.message, "danger");
  } finally {
    // Reset button state
    const refreshBtn = document.querySelector(
      'button[onclick="refreshDocumentsList()"]',
    );
    if (refreshBtn) {
      refreshBtn.innerHTML = '<i class="fas fa-sync-alt me-1"></i>Làm mới';
      refreshBtn.disabled = false;
    }
  }
}

let availableCooperators = [];
let selectedCooperatorIdsModal = [];
let currentBatchForPackage = null;

// ✅ FUNCTION MỞ MODAL TẠO GÓI TỪ BATCH
async function createPackageFromBatch(batchId, batchName, availablePieces = 0) {
  try {
    currentBatchForPackage = {
      id: batchId,
      name: batchName,
      availablePieces,
    };

    // Load cooperators if not loaded
    if (availableCooperators.length === 0) {
      await loadCooperatorsForPackage();
    }

    // ✅ CHECK BATCH MODE VÀ ĐẾM AVAILABLE ITEMS
    const modeResponse = await fetch(
      `/api/batch-fragmentation-mode/${batchId}`,
    );
    const modeData = await modeResponse.json();

    let availableCount = 0;
    let countLabel = "";

    if (modeData.success && modeData.has_packages) {
      // Batch đã có gói - đếm theo mode hiện có
      if (modeData.is_fragmented) {
        // Phân mảnh - đếm pieces
        availableCount = await getAvailablePiecesForBatch(batchId);
        countLabel = "mảnh";
      } else {
        // Không phân mảnh - đếm documents
        availableCount = await getAvailableDocumentsForBatch(batchId);
        countLabel = "tài liệu";
      }
    } else {
      // Batch chưa có gói - mặc định đếm pieces (mode phân mảnh)
      availableCount = await getAvailablePiecesForBatch(batchId);
      countLabel = "mảnh";
    }

    if (availableCount === 0) {
      showToast(
        `Batch này không còn ${countLabel} nào khả dụng để tạo gói`,
        "warning",
      );
      return;
    }

    // Setup modal data
    setupCreatePackageFromBatchModal(
      batchId,
      batchName,
      availableCount,
      countLabel,
    );

    // Show modal
    const modal = new bootstrap.Modal(
      document.getElementById("createPackageFromBatchModal"),
    );
    modal.show();
  } catch (error) {
    showToast("Lỗi khi mở modal tạo gói: " + error.message, "danger");
  }
}

function setupCreatePackageFromBatchModal(
  batchId,
  batchName,
  availablePieces,
  countLabel = "mảnh",
) {
  // Clear messages
  document.getElementById("packageSuccessMessage").classList.add("d-none");
  document.getElementById("packageErrorMessage").classList.add("d-none");

  // Set batch info
  document.getElementById("selectedBatchName").textContent = batchName;
  document.getElementById("selectedBatchInfo").textContent =
    `${availablePieces} ${countLabel} khả dụng • ID: ${batchId.substring(
      0,
      8,
    )}...`;

  // Set package name
  const packageName = `Gói từ ${batchName} - ${new Date().toLocaleDateString(
    "vi-VN",
  )}`;
  document.getElementById("packageNameFromBatch").value = packageName;

  // ✅ SETUP ITEM COUNT INPUT với validation
  const itemCountInput = document.getElementById("itemCountFromBatch");
  const maxItemCountSpan = document.getElementById("maxItemCount");

  // ✅ LƯU COUNT LABEL VÀO DATASET ĐỂ SỬ DỤNG SAU
  itemCountInput.dataset.countLabel = countLabel;

  // Set default và limits
  const defaultItemCount = Math.min(10, availablePieces);
  itemCountInput.value = defaultItemCount;
  itemCountInput.max = availablePieces;
  itemCountInput.min = 1;

  // Update helper text
  if (maxItemCountSpan) {
    maxItemCountSpan.textContent = `${availablePieces} ${countLabel}`;
  }

  // ✅ ENHANCED INPUT VALIDATION với realtime feedback
  itemCountInput.addEventListener("input", function () {
    const value = parseInt(this.value);
    const max = parseInt(this.max);
    const errorEl = document.getElementById("packageErrorMessage");
    const helperEl = document.getElementById("itemCountHelper");
    const label = this.dataset.countLabel || "mảnh";

    // Clear previous errors
    errorEl.classList.add("d-none");

    if (isNaN(value) || value < 1) {
      this.value = 1;
      showPackageError(`Số lượng ${label} phải từ 1 trở lên`);
      if (helperEl) {
        helperEl.innerHTML = `
              <i class="fas fa-exclamation-triangle text-warning me-1"></i>
              Số lượng không hợp lệ. Đã điều chỉnh về 1.
            `;
      }
    } else if (value > max) {
      this.value = max;
      showPackageError(
        `Số lượng ${label} không được vượt quá ${max} (tổng ${label} khả dụng trong batch)`,
      );
      if (helperEl) {
        helperEl.innerHTML = `
              <i class="fas fa-exclamation-triangle text-warning me-1"></i>
              Đã điều chỉnh về giá trị tối đa: ${max} ${label}
            `;
      }
    } else {
      // Valid input - clear error và show info
      errorEl.classList.add("d-none");

      // ✅ HIỂN THỊ THÔNG TIN BỔ SUNG
      const percentage = ((value / max) * 100).toFixed(1);
      if (helperEl) {
        helperEl.innerHTML = `
              <i class="fas fa-info-circle me-1"></i>
              Sẽ gán ${value}/${max} ${label} (${percentage}%) cho cooperator
            `;

        // ✅ THÊM MÀU SẮC DỰA TRÊN PERCENTAGE
        if (percentage <= 50) {
          helperEl.className = "form-text text-success";
        } else if (percentage <= 80) {
          helperEl.className = "form-text text-info";
        } else {
          helperEl.className = "form-text text-warning";
        }
      }
    }
  });

  // ✅ TRIGGER INITIAL VALIDATION để hiển thị thông tin ban đầu
  itemCountInput.dispatchEvent(new Event("input"));

  // Set hidden batch ID
  document.getElementById("selectedBatchIdForPackage").value = batchId;

  // ✅ CHECK BATCH FRAGMENTATION MODE VÀ DISABLE RADIO BUTTONS
  checkAndSetFragmentationMode(batchId);

  // Reset cooperator selection
  selectedCooperatorIdsModal = [];
  renderCooperatorCheckboxesModal();
  updateCooperatorSummaryModal();

  // ✅ THÊM VALIDATION CHO PACKAGE NAME
  const packageNameInput = document.getElementById("packageNameFromBatch");
  packageNameInput.addEventListener("input", function () {
    const errorEl = document.getElementById("packageErrorMessage");
    if (this.value.trim().length >= 3) {
      errorEl.classList.add("d-none");
    } else if (this.value.trim().length > 0) {
      showPackageError("Tên gói phải có ít nhất 3 ký tự");
    }
  });

  // ✅ GẮN EVENT LISTENER CHO NÚT CREATE
  const createBtn = document.getElementById("createPackageFromBatchBtn");
  if (createBtn) {
    // Remove existing listeners
    createBtn.onclick = null;

    // Add new listener
    createBtn.onclick = function () {
      handleCreatePackageFromBatch();
    };
  }

  // ✅ FOCUS VÀO PACKAGE NAME INPUT
  setTimeout(() => {
    packageNameInput.focus();
    packageNameInput.select();
  }, 100);

  // ✅ THÊM KEYBOARD SHORTCUTS
  const modal = document.getElementById("createPackageFromBatchModal");
  modal.addEventListener("keydown", function (event) {
    if (event.key === "Enter" && (event.ctrlKey || event.metaKey)) {
      // Ctrl+Enter hoặc Cmd+Enter để tạo gói
      event.preventDefault();
      handleCreatePackageFromBatch();
    }
  });

  console.log(
    `📦 Setup modal for batch: ${batchName} (${availablePieces} pieces available)`,
  );
}

// ✅ CHECK VÀ SET FRAGMENTATION MODE DỰA TRÊN BATCH ĐÃ CÓ GÓI CHƯA
async function checkAndSetFragmentationMode(batchId) {
  try {
    const response = await fetch(`/api/batch-fragmentation-mode/${batchId}`);
    const data = await response.json();

    if (data.success && data.has_packages) {
      // Batch đã có gói với mode cụ thể
      const isFragmented = data.is_fragmented;
      const fragmentedRadio = document.getElementById(
        "fragmentedModeFromBatch",
      );
      const nonFragmentedRadio = document.getElementById(
        "nonFragmentedModeFromBatch",
      );
      const helpText = document.getElementById("fragmentationHelpFromBatch");

      if (isFragmented) {
        // Đã có gói phân mảnh - chỉ cho phép phân mảnh
        fragmentedRadio.checked = true;
        fragmentedRadio.disabled = false;
        nonFragmentedRadio.disabled = true;

        helpText.innerHTML = `
          <i class="fas fa-info-circle text-primary me-1"></i>
          <strong>Batch này đã có gói phân mảnh.</strong> Chỉ có thể tạo thêm gói phân mảnh.
        `;
        helpText.className = "form-text text-primary";
      } else {
        // Đã có gói không phân mảnh - chỉ cho phép không phân mảnh
        nonFragmentedRadio.checked = true;
        nonFragmentedRadio.disabled = false;
        fragmentedRadio.disabled = true;

        helpText.innerHTML = `
          <i class="fas fa-info-circle text-success me-1"></i>
          <strong>Batch này đã có gói không phân mảnh.</strong> Chỉ có thể tạo thêm gói không phân mảnh.
        `;
        helpText.className = "form-text text-success";
      }

      console.log(
        `🔒 Locked fragmentation mode: ${data.mode} (existing package: ${data.package_name})`,
      );
    } else {
      // Batch chưa có gói - cho phép cả hai mode
      const fragmentedRadio = document.getElementById(
        "fragmentedModeFromBatch",
      );
      const nonFragmentedRadio = document.getElementById(
        "nonFragmentedModeFromBatch",
      );

      fragmentedRadio.disabled = false;
      nonFragmentedRadio.disabled = false;
      fragmentedRadio.checked = true; // Default: phân mảnh

      handleFragmentationModeChangeFromBatch(); // Update help text

      console.log(`🆓 Batch has no packages - both modes available`);
    }
  } catch (error) {
    console.error("Error checking batch fragmentation mode:", error);
    // Nếu lỗi, mặc định cho phép cả hai mode
  }
}

// ✅ HANDLE FRAGMENTATION MODE CHANGE FROM BATCH MODAL
function handleFragmentationModeChangeFromBatch() {
  const selectedMode = document.querySelector(
    'input[name="fragmentationModeFromBatch"]:checked',
  );
  const isFragmented = selectedMode ? selectedMode.value === "true" : true;

  const helpText = document.getElementById("fragmentationHelpFromBatch");

  if (isFragmented) {
    helpText.innerHTML = `
      <i class="fas fa-info-circle me-1"></i>
      Phân mảnh: Mỗi tài liệu chia thành 2 mảnh (Header + Content)
    `;
  } else {
    helpText.innerHTML = `
      <i class="fas fa-info-circle me-1"></i>
      Không phân mảnh: Cooperator chỉnh sửa toàn bộ tài liệu PDF
    `;
  }

  // ✅ CẬP NHẬT MAX COUNT NẾU CẦN
  const itemCountInput = document.getElementById("itemCountFromBatch");
  if (itemCountInput) {
    // Trigger validation lại
    itemCountInput.dispatchEvent(new Event("input"));
  }

  console.log(
    `🔀 Fragmentation mode changed to: ${
      isFragmented ? "Fragmented" : "Non-Fragmented"
    }`,
  );
}

// ✅ LOAD COOPERATORS FOR PACKAGE CREATION
async function loadCooperatorsForPackage() {
  try {
    const response = await fetch("/api/cooperators");
    const data = await response.json();

    if (data.success) {
      availableCooperators = data.cooperators;

      if (availableCooperators.length === 0) {
        showToast(
          "Bạn chưa có cooperator nào. Vui lòng liên hệ admin.",
          "warning",
        );
      }
    } else {
      throw new Error(data.error);
    }
  } catch (error) {
    showToast("Lỗi khi tải danh sách cooperator: " + error.message, "danger");
  }
}

// ✅ RENDER COOPERATOR CHECKBOXES IN MODAL
function renderCooperatorCheckboxesModal(filter = "") {
  const container = document.getElementById("cooperatorCheckboxList");
  if (!container) return;
  const filtered = availableCooperators.filter(
    (c) =>
      !filter ||
      c.full_name.toLowerCase().includes(filter.toLowerCase()) ||
      c.username.toLowerCase().includes(filter.toLowerCase()),
  );
  container.innerHTML =
    filtered.length === 0
      ? '<div class="text-muted text-center py-2" style="font-size:0.85em;">Không tìm thấy cooperator</div>'
      : filtered
          .map(
            (c) => `
      <label class="d-flex align-items-center px-2 py-1 gap-2" style="cursor:pointer;border-radius:4px;${selectedCooperatorIdsModal.includes(c.id) ? "background:#e8f4fd;" : ""}" onmouseenter="this.style.background='#f0f0f0'" onmouseleave="this.style.background='${selectedCooperatorIdsModal.includes(c.id) ? "#e8f4fd" : ""}'">
        <input type="checkbox" class="cooperator-cb-modal" value="${c.id}" ${selectedCooperatorIdsModal.includes(c.id) ? "checked" : ""} onchange="handleCooperatorCheckboxModal(this)" style="margin:0;" />
        <span style="font-size:0.85em;">${c.full_name} <span class="text-muted">(${c.username})</span></span>
      </label>
    `,
          )
          .join("");
  // Update select all checkbox
  const selectAllCb = document.getElementById("selectAllCooperators");
  if (selectAllCb)
    selectAllCb.checked =
      selectedCooperatorIdsModal.length === availableCooperators.length &&
      availableCooperators.length > 0;
}

function handleCooperatorCheckboxModal(cb) {
  const id = cb.value;
  if (cb.checked) {
    if (!selectedCooperatorIdsModal.includes(id))
      selectedCooperatorIdsModal.push(id);
  } else {
    selectedCooperatorIdsModal = selectedCooperatorIdsModal.filter(
      (x) => x !== id,
    );
  }
  renderCooperatorCheckboxesModal(
    document.getElementById("cooperatorSearchInput")?.value || "",
  );
  updateCooperatorSummaryModal();
}

function toggleAllCooperatorsModal(masterCb) {
  if (masterCb.checked) {
    selectedCooperatorIdsModal = availableCooperators.map((c) => c.id);
  } else {
    selectedCooperatorIdsModal = [];
  }
  renderCooperatorCheckboxesModal(
    document.getElementById("cooperatorSearchInput")?.value || "",
  );
  updateCooperatorSummaryModal();
}

function filterCooperatorsModal(searchText) {
  renderCooperatorCheckboxesModal(searchText);
}

function updateCooperatorSummaryModal() {
  const badge = document.getElementById("cooperatorCountBadge");
  const summary = document.getElementById("selectedCooperatorsSummaryModal");
  if (badge) badge.textContent = `${selectedCooperatorIdsModal.length} đã chọn`;
  if (summary) {
    if (selectedCooperatorIdsModal.length === 0) {
      summary.textContent = "Chưa chọn cooperator nào";
    } else {
      const names = availableCooperators
        .filter((c) => selectedCooperatorIdsModal.includes(c.id))
        .map((c) => c.full_name);
      summary.textContent = names.join(", ");
    }
  }
  // Clear error if cooperators selected
  if (selectedCooperatorIdsModal.length > 0) {
    const errorEl = document.getElementById("packageErrorMessage");
    if (errorEl) errorEl.classList.add("d-none");
  }
}

// ✅ GET AVAILABLE PIECES FOR SPECIFIC BATCH
async function getAvailablePiecesForBatch(batchId) {
  try {
    const response = await fetch("/api/available-pieces-count-by-batches", {
      method: "POST",
      headers: {
        "Content-Type": "application/json",
      },
      body: JSON.stringify({
        batch_mode: "specific",
        selected_batch_ids: [batchId],
      }),
    });

    const data = await response.json();

    if (data.success) {
      return data.count;
    } else {
      throw new Error(data.error);
    }
  } catch (error) {
    console.error("Error getting available pieces:", error);
    return 0;
  }
}

// ✅ GET AVAILABLE DOCUMENTS FOR SPECIFIC BATCH (FOR NON-FRAGMENTED MODE)
async function getAvailableDocumentsForBatch(batchId) {
  try {
    const response = await fetch("/api/available-documents-count-by-batch", {
      method: "POST",
      headers: {
        "Content-Type": "application/json",
      },
      body: JSON.stringify({
        batch_id: batchId,
      }),
    });

    const data = await response.json();

    if (data.success) {
      return data.count;
    } else {
      throw new Error(data.error);
    }
  } catch (error) {
    console.error("Error getting available documents:", error);
    return 0;
  }
}

async function handleCreatePackageFromBatch() {
  const btn = document.getElementById("createPackageFromBatchBtn");
  const form = document.getElementById("createPackageFromBatchForm");

  if (!form.checkValidity()) {
    form.reportValidity();
    return;
  }

  const packageName = document
    .getElementById("packageNameFromBatch")
    .value.trim();
  const itemCount = parseInt(
    document.getElementById("itemCountFromBatch").value,
  );
  const batchId = document.getElementById("selectedBatchIdForPackage").value;

  if (
    !packageName ||
    selectedCooperatorIdsModal.length === 0 ||
    !batchId ||
    itemCount <= 0
  ) {
    showPackageError(
      selectedCooperatorIdsModal.length === 0
        ? "Vui lòng chọn ít nhất 1 cooperator"
        : "Vui lòng điền đầy đủ thông tin",
    );
    return;
  }

  btn.disabled = true;
  btn.innerHTML = '<i class="fas fa-spinner fa-spin me-1"></i>Đang tạo...';

  // ✅ LẤY CHẾ ĐỘ PHÂN MẢNH
  const fragmentationMode = document.querySelector(
    'input[name="fragmentationModeFromBatch"]:checked',
  );
  const isFragmented = fragmentationMode
    ? fragmentationMode.value === "true"
    : true;

  try {
    const response = await fetch("/api/packages", {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify({
        name: packageName,
        cooperator_ids: selectedCooperatorIdsModal,
        item_count: itemCount,
        batch_mode: "specific",
        selected_batch_ids: [batchId],
        is_fragmented: isFragmented,
      }),
    });

    const data = await response.json();

    if (data.success) {
      showPackageSuccess(data.message);
      setTimeout(() => {
        const modal = bootstrap.Modal.getInstance(
          document.getElementById("createPackageFromBatchModal"),
        );
        if (modal) modal.hide();
        if (
          confirm(
            "Gói đã được tạo thành công! Bạn có muốn xem danh sách gói không?",
          )
        ) {
          window.location.href = "/manage-packages-page";
        }
      }, 2000);
    } else {
      showPackageError(data.error || "Lỗi không xác định");
    }
  } catch (error) {
    showPackageError("Lỗi khi tạo gói: " + error.message);
  } finally {
    btn.disabled = false;
    btn.innerHTML = '<i class="fas fa-plus me-1"></i>Tạo gói';
  }
}

async function navigateToNextDocument() {
  const globalPosition =
    (currentPage - 1) * pageSize + currentDocumentIndex + 1;

  // ✅ KIỂM TRA XEM CÓ DOCUMENT TIẾP THEO KHÔNG
  if (globalPosition >= totalBatchDocuments) {
    showToast("Đã đến document cuối cùng", "info");
    return;
  }

  // ✅ KIỂM TRA XEM DOCUMENT TIẾP THEO CÓ TRONG TRANG HIỆN TẠI KHÔNG
  if (currentDocumentIndex + 1 < currentBatchDocuments.length) {
    // Document tiếp theo trong trang hiện tại
    const nextDoc = currentBatchDocuments[currentDocumentIndex + 1];
    await loadAndShowDocument(nextDoc.id, currentDocumentIndex + 1);
  } else {
    // ✅ CẦN CHUYỂN SANG TRANG TIẾP THEO
    if (currentPage < totalPages) {
      await loadNextPageAndNavigate();
    } else {
      showToast("Đã đến document cuối cùng", "info");
    }
  }
}

function showPackageSuccess(message) {
  const successEl = document.getElementById("packageSuccessMessage");
  const errorEl = document.getElementById("packageErrorMessage");
  errorEl.classList.add("d-none");
  successEl.textContent = message;
  successEl.classList.remove("d-none");
}

function showPackageError(message) {
  const successEl = document.getElementById("packageSuccessMessage");
  const errorEl = document.getElementById("packageErrorMessage");
  successEl.classList.add("d-none");
  errorEl.textContent = message;
  errorEl.classList.remove("d-none");
}
// ✅ THÊM VÀO CUỐI SCRIPT SECTION TRONG index.html

// ✅ CẬP NHẬT PHẦN EXTERNAL PROJECTS JAVASCRIPT

// External Projects Variables
let currentExternalProject = null;
let currentDepartment = null;
let currentFolderPath = "";
let selectedExternalFiles = [];
let allExternalProjects = [];

async function selectProject(projectId, projectName) {
  currentExternalProject = { id: projectId, name: projectName };

  // Update UI
  document.getElementById("currentProjectName").textContent = projectName;
  document.getElementById("projectsListSection").style.display = "none";
  document.getElementById("folderBrowserSection").style.display = "block";

  // Reset state
  currentFolderPath = "";
  selectedExternalFiles = [];
  updateBreadcrumb();

  // ✅ LOAD PROJECT ROOT DIRECTLY (NO DEPARTMENT STEP)
  await loadFolderContents("");
}

// ✅ loadFolderContents - SIMPLIFIED (NO DEPARTMENT)
async function loadFolderContents(folderPath) {
  const container = document.getElementById("folderContentsContainer");

  container.innerHTML = `
        <div class="text-center py-4">
            <div class="spinner-border text-primary" role="status"></div>
            <div>Đang tải nội dung thư mục...</div>
        </div>
      `;

  try {
    // ✅ SIMPLIFIED REQUEST - ONLY PROJECT + FOLDER
    const requestData = {
      project_id: currentExternalProject.id,
      folder_path: folderPath,
    };

    const response = await fetch("/api/external/folder-contents", {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify(requestData),
    });

    const data = await response.json();

    if (data.success) {
      currentFolderPath = folderPath;
      renderFolderContents(data.subfolders, data.files, data.full_path);
      updateBreadcrumb();
    } else {
      container.innerHTML = `<div class="alert alert-danger">${data.error}</div>`;
    }
  } catch (error) {
    container.innerHTML = `
          <div class="alert alert-danger">
              Lỗi khi tải thư mục: ${error.message}
          </div>
        `;
  }
}

function renderFolderContents(subfolders, files, fullPath = "") {
  const container = document.getElementById("folderContentsContainer");
  let html = "";

  // ✅ FOLDER ACTIONS - CHỈ GIỮ LẠI NÚT TẠO BATCH
  html += `
        <div class="alert alert-light border p-3 mb-3">
            <div class="d-flex justify-content-between align-items-center">
                <div>
                    <strong>${subfolders.length}</strong> thư mục con •
                    <strong>${files.length}</strong> files PDF
                </div>
                <div>
                    <button class="btn btn-success" onclick="createBatchFromCurrentFolder()" title="Tạo batch từ thư mục này">
                        <i class="fas fa-plus me-2"></i>Tạo batch từ thư mục này
                    </button>
                </div>
            </div>
        </div>
      `;

  // Show path info
  if (fullPath) {
    html += `
            <div class="alert alert-info py-2 mb-3">
                <small><i class="fas fa-server me-1"></i><strong>Đường dẫn:</strong> ${fullPath}</small>
            </div>
        `;
  }

  html += '<div class="list-group">';

  // BACK NAVIGATION
  if (currentFolderPath) {
    html += `
            <div class="list-group-item list-group-item-action d-flex align-items-center"
                 onclick="navigateToParentFolder()" style="cursor: pointer;">
                <i class="fas fa-arrow-up text-secondary me-3"></i>
                <div class="flex-grow-1">
                    <div class="fw-bold">.. (Thư mục cha)</div>
                    <small class="text-muted">Quay lại thư mục trước</small>
                </div>
            </div>
        `;
  }

  // RENDER SUBFOLDERS
  subfolders.forEach((folderName) => {
    html += `
            <div class="list-group-item list-group-item-action d-flex align-items-center"
                 onclick="navigateToSubfolder('${folderName}')" style="cursor: pointer;">
                <i class="fas fa-folder text-warning me-3"></i>
                <div class="flex-grow-1">
                    <div class="fw-bold">${folderName}</div>
                    <small class="text-muted">Thư mục con</small>
                </div>
                <i class="fas fa-chevron-right text-muted"></i>
            </div>
        `;
  });

  // RENDER FILES
  if (files.length > 0) {
    files.forEach((file) => {
      const fileSizeMB = (file.fileSize / 1024 / 1024).toFixed(1);
      const hasVanBanId = file.idVanBan ? "text-success" : "text-warning";
      const vanBanText = file.idVanBan ? `ID: ${file.idVanBan}` : "Chưa có ID";

      html += `
                <div class="list-group-item d-flex align-items-center">
                    <i class="fas fa-file-pdf text-danger me-3"></i>
                    <div class="flex-grow-1">
                        <div class="fw-bold">${file.tenFile}</div>
                        <small class="text-muted">
                            ${fileSizeMB} MB •
                            <span class="${hasVanBanId}">${vanBanText}</span>
                            ${
                              file.department_id
                                ? `• Phòng: ${file.department_id}`
                                : ""
                            }
                        </small>
                    </div>
                </div>
            `;
    });
  } else if (subfolders.length === 0) {
    html += `
            <div class="text-center py-4 text-muted">
                <i class="fas fa-folder-open fa-2x mb-2 d-block"></i>
                Thư mục trống
            </div>
        `;
  }

  html += "</div>";
  container.innerHTML = html;
}

function updateBreadcrumb() {
  const breadcrumb = document.getElementById("folderBreadcrumb");

  let html = `
            <li class="breadcrumb-item">
                <a href="#" onclick="loadFolderContents('')">
                    ${currentExternalProject.name}
                </a>
            </li>
        `;

  if (currentFolderPath) {
    const pathParts = currentFolderPath
      .split("/")
      .filter((part) => part.trim());
    let buildPath = "";

    pathParts.forEach((part, index) => {
      buildPath += (buildPath ? "/" : "") + part;

      if (index === pathParts.length - 1) {
        // Last part - active
        html += `<li class="breadcrumb-item active">${part}</li>`;
      } else {
        // Intermediate parts - clickable
        html += `
                        <li class="breadcrumb-item">
                            <a href="#" onclick="loadFolderContents('${buildPath}')">${part}</a>
                        </li>
                    `;
      }
    });
  }

  breadcrumb.innerHTML = html;
}
function navigateToParentFolder() {
  const pathParts = currentFolderPath.split("/").filter((part) => part.trim());
  pathParts.pop(); // Remove last part
  const parentPath = pathParts.join("/");
  loadFolderContents(parentPath);
}

// ✅ NAVIGATION HELPERS (ĐƠN GIẢN HÓA)
function navigateToSubfolder(folderName) {
  const newPath = currentFolderPath
    ? `${currentFolderPath}/${folderName}`
    : folderName;
  loadFolderContents(newPath);
}
// Toggle File Selection (updated)
function toggleFileSelection(fileId, fileObj) {
  try {
    const existingIndex = selectedExternalFiles.findIndex(
      (f) => f.idFile === fileId,
    );

    if (existingIndex >= 0) {
      // Remove from selection
      selectedExternalFiles.splice(existingIndex, 1);
    } else {
      // Add to selection
      selectedExternalFiles.push(fileObj);
    }

    updateSelectedFilesList();
    updateBatchCreationSection();
  } catch (error) {
    console.error("Error toggling file selection:", error);
  }
}

// Update Batch Creation Section (updated for departments)
function updateBatchCreationSection() {
  const section = document.getElementById("batchCreationSection");
  const batchNameInput = document.getElementById("externalBatchName");
  const totalFilesSpan = document.getElementById("totalSelectedFiles");

  if (selectedExternalFiles.length > 0) {
    section.style.display = "block";

    // ✅ AUTO-GENERATE BATCH NAME VỚI FULL PATH
    const suggestedName = generateExternalBatchNamePreview();

    if (
      !batchNameInput.value ||
      batchNameInput.placeholder.includes("tự động")
    ) {
      batchNameInput.value = suggestedName;
    }

    totalFilesSpan.textContent = selectedExternalFiles.length;
  } else {
    section.style.display = "none";
  }
}
async function createBatchFromExternal() {
  if (selectedExternalFiles.length === 0) {
    showToast("Vui lòng chọn ít nhất một file", "warning");
    return;
  }

  const batchName = document.getElementById("externalBatchName").value.trim();
  if (!batchName) {
    showToast("Vui lòng nhập tên batch", "warning");
    return;
  }

  const btn = document.getElementById("createExternalBatchBtn");
  const originalHtml = btn.innerHTML;

  try {
    // ✅ ENHANCED LOADING STATE
    btn.disabled = true;
    btn.innerHTML = '<i class="fas fa-spinner fa-spin me-1"></i>Đang tạo...';

    // ✅ DISABLE FORM INPUTS
    document.getElementById("externalBatchName").disabled = true;
    const selectedFilesList = document.getElementById("selectedFilesList");

    // ✅ ADD LOADING OVERLAY TO SELECTED FILES
    const loadingOverlay = document.createElement("div");
    loadingOverlay.className =
      "position-absolute w-100 h-100 d-flex align-items-center justify-content-center";
    loadingOverlay.style.cssText =
      "background: rgba(255,255,255,0.9); z-index: 1000; top: 0; left: 0;";
    loadingOverlay.innerHTML = `
          <div class="text-center">
            <div class="spinner-border spinner-border-sm text-success mb-2" role="status"></div>
            <div><small>Đang tạo ${selectedExternalFiles.length} documents...</small></div>
          </div>
        `;

    if (selectedFilesList) {
      selectedFilesList.style.position = "relative";
      selectedFilesList.appendChild(loadingOverlay);
    }

    // ✅ PROGRESS TOAST
    showToast(
      `🚧 Đang tạo batch với ${selectedExternalFiles.length} files...`,
      "info",
    );

    const response = await fetch("/api/external/create-batch", {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify({
        project_id: currentExternalProject.id,
        project_name: currentExternalProject.name,
        selected_files: selectedExternalFiles,
        batch_name: batchName,
        folder_path: currentFolderPath,
      }),
    });

    const data = await response.json();

    if (data.success) {
      // ✅ SUCCESS WITH DETAILED INFO
      const successMsg =
        `✅ ${data.message}\n\n` +
        `📦 Batch: ${data.batch_name}\n` +
        `📄 Documents: ${data.created_documents}\n` +
        `${
          data.failed_files?.length > 0
            ? `❌ Failed: ${data.failed_files.length}\n`
            : ""
        }`;

      showToast(successMsg, "success");

      // Close modal
      bootstrap.Modal.getInstance(
        document.getElementById("externalProjectsModal"),
      ).hide();

      // Refresh batches list
      loadBatches();

      // Show failed files detail if any
      if (data.failed_files && data.failed_files.length > 0) {
        setTimeout(() => {
          console.warn("Failed files:", data.failed_files);
          alert(
            `⚠️ Một số files không thể tạo:\n\n` +
              data.failed_files
                .map((f) => `• ${f.filename}: ${f.error}`)
                .join("\n"),
          );
        }, 2000);
      }
    } else {
      throw new Error(data.error || "Không thể tạo batch");
    }
  } catch (error) {
    console.error("Error creating batch from external:", error);
    showToast("❌ Lỗi: " + error.message, "danger");
  } finally {
    // ✅ RESET ALL STATES
    btn.disabled = false;
    btn.innerHTML = originalHtml;
    document.getElementById("externalBatchName").disabled = false;

    // Remove loading overlay
    const overlay = selectedFilesList?.querySelector(".position-absolute");
    if (overlay) {
      overlay.remove();
    }
  }
}

async function showExternalProjectsModal() {
  // Reset state trước
  currentExternalProject = null;
  currentDepartment = null;
  currentFolderPath = "";
  selectedExternalFiles = [];
  allExternalProjects = [];

  // Show modal
  const modal = new bootstrap.Modal(
    document.getElementById("externalProjectsModal"),
  );
  modal.show();

  // ✅ ĐỢI MODAL HIỂN THỊ HOÀN TOÀN RỒI MỚI LOAD DATA
  const modalElement = document.getElementById("externalProjectsModal");
  modalElement.addEventListener(
    "shown.bs.modal",
    function onModalShown() {
      // Remove listener để tránh gọi lại
      modalElement.removeEventListener("shown.bs.modal", onModalShown);

      // Ensure correct sections are shown
      document.getElementById("projectsListSection").style.display = "block";
      document.getElementById("folderBrowserSection").style.display = "none";

      // ✅ LOAD PROJECTS SAU KHI MODAL ĐÃ HIỂN THỊ
      loadExternalProjects();
    },
    { once: true },
  );
}

async function loadExternalProjects() {
  const container = document.getElementById("projectsListContainer");

  if (!container) {
    console.error("Project list container not found");
    return;
  }

  try {
    // ✅ HIỂN THỊ LOADING STATE
    container.innerHTML = `
          <div class="text-center py-4">
            <div class="spinner-border text-primary" role="status">
              <span class="visually-hidden">Đang tải...</span>
            </div>
            <div class="mt-2">Đang tải danh sách dự án...</div>
          </div>
        `;

    console.log("🔄 Loading external projects...");

    // ✅ CALL API VỚI TIMEOUT
    const controller = new AbortController();
    const timeoutId = setTimeout(() => controller.abort(), 15000); // 15s timeout

    const response = await fetch("/api/external/projects", {
      signal: controller.signal,
      headers: {
        "Content-Type": "application/json",
      },
    });

    clearTimeout(timeoutId);

    if (!response.ok) {
      throw new Error(`HTTP ${response.status}: ${response.statusText}`);
    }

    const data = await response.json();
    console.log("📊 Projects response:", data);

    if (data.success) {
      allExternalProjects = data.projects || [];
      console.log(`✅ Loaded ${allExternalProjects.length} projects`);

      // ✅ DELAY NHẸ ĐỂ USER THẤY LOADING (UX)
      setTimeout(() => {
        renderProjectsList(allExternalProjects);
      }, 300);
    } else {
      throw new Error(data.error || "API trả về lỗi không xác định");
    }
  } catch (error) {
    console.error("❌ Error loading external projects:", error);

    let errorMessage = "Không thể tải danh sách dự án";

    if (error.name === "AbortError") {
      errorMessage = "Timeout: Kết nối quá chậm, vui lòng thử lại";
    } else if (error.message.includes("HTTP")) {
      errorMessage = `Lỗi server: ${error.message}`;
    } else if (error.message.includes("fetch")) {
      errorMessage = "Lỗi kết nối mạng";
    } else {
      errorMessage = error.message;
    }

    container.innerHTML = `
          <div class="alert alert-danger">
            <div class="d-flex align-items-center mb-2">
              <i class="fas fa-exclamation-triangle me-2"></i>
              <strong>Lỗi tải dự án</strong>
            </div>
            <div class="mb-3">${errorMessage}</div>
            <div>
              <button class="btn btn-outline-danger btn-sm" onclick="loadExternalProjects()">
                <i class="fas fa-redo me-1"></i>Thử lại
              </button>
              <button class="btn btn-outline-secondary btn-sm ms-2" onclick="testApiConnection()">
                <i class="fas fa-network-wired me-1"></i>Test kết nối
              </button>
            </div>
          </div>
        `;
  }
}
function renderProjectsList(projects) {
  const container = document.getElementById("projectsListContainer");

  if (!container) {
    console.error("Project list container not found in renderProjectsList");
    return;
  }

  console.log(`🎨 Rendering ${projects.length} projects`);

  if (projects.length === 0) {
    container.innerHTML = `
          <div class="text-center py-5 text-muted">
            <i class="fas fa-folder-open fa-3x mb-3 d-block text-secondary"></i>
            <h6>Không có dự án nào</h6>
            <p class="mb-3">Hệ thống chưa có dự án external hoặc bạn chưa có quyền truy cập.</p>
            <button class="btn btn-outline-primary btn-sm" onclick="loadExternalProjects()">
              <i class="fas fa-refresh me-1"></i>Làm mới
            </button>
          </div>
        `;
    return;
  }

  // ✅ GENERATE HTML VỚI VALIDATION
  const projectsHtml = projects
    .map((project) => {
      // ✅ VALIDATE PROJECT DATA
      const projectName = project.tenDuAn || "Tên dự án không xác định";
      const projectCode = project.maDuAn || "N/A";
      const projectId = project.id || project.ID || "";

      if (!projectId) {
        console.warn("Project missing ID:", project);
        return ""; // Skip projects without ID
      }

      // ✅ SAFE DATE FORMATTING
      let createdDate = "N/A";
      try {
        if (project.createdDate) {
          createdDate = new Date(project.createdDate).toLocaleDateString(
            "vi-VN",
          );
        }
      } catch (e) {
        console.warn("Invalid date for project:", project.createdDate);
      }

      // ✅ SAFE DEPARTMENTS COUNT
      const deptCount = Array.isArray(project.phongBanDTOs)
        ? project.phongBanDTOs.length
        : 0;

      // ✅ ESCAPE QUOTES IN PROJECT NAME
      const safeProjectName = projectName
        .replace(/'/g, "\\'")
        .replace(/"/g, '\\"');

      return `
            <div class="card mb-2 project-card" onclick="selectProject('${projectId}', '${safeProjectName}')">
              <div class="card-body py-2">
                <div class="d-flex justify-content-between align-items-center">
                  <div class="flex-grow-1">
                    <h6 class="mb-1 text-truncate">${projectName}</h6>
                    <small class="text-muted">
                      <i class="fas fa-hashtag"></i> ${projectCode} •
                      <i class="fas fa-calendar"></i> ${createdDate}
                      ${
                        deptCount > 0
                          ? `• <i class="fas fa-building"></i> ${deptCount} phòng ban`
                          : ""
                      }
                    </small>
                  </div>
                  <i class="fas fa-chevron-right text-muted"></i>
                </div>
              </div>
            </div>
          `;
    })
    .filter((html) => html) // Remove empty strings
    .join("");

  if (projectsHtml) {
    container.innerHTML = projectsHtml;

    // ✅ ADD HOVER EFFECTS AFTER RENDERING
    setTimeout(() => {
      document.querySelectorAll(".project-card").forEach((card) => {
        card.addEventListener("mouseenter", function () {
          this.style.backgroundColor = "#f8f9fa";
          this.style.cursor = "pointer";
        });
        card.addEventListener("mouseleave", function () {
          this.style.backgroundColor = "";
        });
      });
    }, 100);

    console.log(`✅ Successfully rendered ${projects.length} projects`);
  } else {
    container.innerHTML = `
          <div class="alert alert-warning">
            <i class="fas fa-exclamation-triangle me-2"></i>
            Có dự án nhưng không thể hiển thị. Vui lòng kiểm tra định dạng dữ liệu.
          </div>
        `;
  }
}

// Filter Projects
function filterProjects() {
  const searchTerm = document
    .getElementById("projectSearchInput")
    .value.toLowerCase();

  const filteredProjects = allExternalProjects.filter(
    (project) =>
      project.tenDuAn.toLowerCase().includes(searchTerm) ||
      project.maDuAn.toLowerCase().includes(searchTerm),
  );

  renderProjectsList(filteredProjects);
}

// Update Selected Files List
function updateSelectedFilesList() {
  const container = document.getElementById("selectedFilesList");
  const countBadge = document.getElementById("selectedFilesCount");
  const clearBtn = document.getElementById("clearSelectionBtn");

  countBadge.textContent = selectedExternalFiles.length;

  if (selectedExternalFiles.length === 0) {
    container.innerHTML = `
          <div class="text-muted text-center py-3">
            <i class="fas fa-file-plus fa-2x mb-2 d-block"></i>
            Chưa chọn file nào
          </div>
        `;
    clearBtn.style.display = "none";
    return;
  }

  clearBtn.style.display = "block";

  container.innerHTML = selectedExternalFiles
    .map(
      (file) => `
        <div class="d-flex align-items-center mb-1 p-1 bg-light rounded">
          <i class="fas fa-file-pdf text-danger me-2"></i>
          <div class="flex-grow-1" style="font-size: 0.8rem;">
            <div class="text-truncate">${file.tenFile}</div>
          </div>
          <button class="btn btn-sm btn-outline-danger btn-sm"
                  onclick="removeSelectedFile('${file.idFile}')" title="Xóa">
            <i class="fas fa-times"></i>
          </button>
        </div>
      `,
    )
    .join("");
}

// Remove Selected File
function removeSelectedFile(fileId) {
  selectedExternalFiles = selectedExternalFiles.filter(
    (f) => f.idFile !== fileId,
  );
  updateSelectedFilesList();
  updateBatchCreationSection();

  // Uncheck checkbox in current folder view
  const checkbox = document.querySelector(
    `input[onchange*="toggleFileSelection('${fileId}'"]`,
  );
  if (checkbox) {
    checkbox.checked = false;
    checkbox
      .closest(".list-group-item")
      .classList.remove("list-group-item-success");
  }
}

// Clear Selected Files
function clearSelectedFiles() {
  selectedExternalFiles = [];
  updateSelectedFilesList();
  updateBatchCreationSection();

  // Uncheck all checkboxes
  document
    .querySelectorAll('#folderContentsContainer input[type="checkbox"]')
    .forEach((checkbox) => {
      checkbox.checked = false;
      checkbox
        .closest(".list-group-item")
        .classList.remove("list-group-item-success");
    });
}

// Back to Projects List
function backToProjectsList() {
  document.getElementById("projectsListSection").style.display = "block";
  document.getElementById("folderBrowserSection").style.display = "none";
  document.getElementById("batchCreationSection").style.display = "none";

  // Reset state
  currentExternalProject = null;
  currentDepartment = null;
  currentFolderPath = "";
  selectedExternalFiles = [];
}

// Helper function to get current folder files (for refresh)
async function getCurrentFolderFiles() {
  try {
    const requestData = {
      project_id: currentExternalProject.id,
      folder_path: currentFolderPath,
    };

    if (currentDepartment) {
      requestData.department_id = currentDepartment.id;
    }

    const response = await fetch("/api/external/folder-contents", {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify(requestData),
    });

    const data = await response.json();
    return data.success ? data.files : [];
  } catch (error) {
    console.error("Error getting current folder files:", error);
    return [];
  }
}
async function createBatchFromCurrentFolder() {
  const suggestedName = generateExternalBatchNamePreview();
  const batchName = prompt("Nhập tên batch:", suggestedName);

  if (!batchName) return;

  const createBtn = document.querySelector(
    'button[onclick="createBatchFromCurrentFolder()"]',
  );
  const originalHtml = createBtn ? createBtn.innerHTML : "";

  try {
    // ✅ SIMPLE LOADING STATE
    if (createBtn) {
      createBtn.disabled = true;
      createBtn.innerHTML =
        '<i class="fas fa-spinner fa-spin me-1"></i>Đang tạo batch...';
    }

    showToast("🚀 Đang quét files và tạo batch...", "info");

    const response = await fetch("/api/external/create-batch-from-folder", {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify({
        project_id: currentExternalProject.id,
        project_name: currentExternalProject.name,
        folder_path: currentFolderPath,
        batch_name: batchName,
      }),
    });

    const data = await response.json();

    if (data.success) {
      showToast(
        `✅ Đã tạo batch "${data.batch_name}" với ${data.created_documents} files!`,
        "success",
      );

      // ✅ SIMPLE CONFIRMATION
      setTimeout(() => {
        if (
          confirm(
            `Batch đã tạo thành công với ${data.created_documents} files!\n\nĐóng modal và xem batch?`,
          )
        ) {
          bootstrap.Modal.getInstance(
            document.getElementById("externalProjectsModal"),
          ).hide();
          loadBatches();
        }
      }, 1000);
    } else {
      showToast("❌ Lỗi: " + data.error, "danger");
    }
  } catch (error) {
    showToast("❌ Lỗi: " + error.message, "danger");
  } finally {
    // ✅ RESET BUTTON STATE
    if (createBtn) {
      createBtn.disabled = false;
      createBtn.innerHTML =
        originalHtml ||
        '<i class="fas fa-plus me-1"></i>Tạo batch từ thư mục này';
    }
  }
}

// ✅ CẬP NHẬT FUNCTION createBatchFromEntireProject
async function createBatchFromEntireProject() {
  const confirmMessage =
    `Tạo batch từ TOÀN BỘ DỰ ÁN "${currentExternalProject.name}"?\n\n` +
    `⚠️ Điều này sẽ quét tất cả file PDF trong tất cả phòng ban của dự án.\n` +
    `Có thể mất thời gian nếu dự án có nhiều file.`;

  if (!confirm(confirmMessage)) {
    return;
  }

  try {
    // ✅ SUGGESTED NAME CHO TOÀN BỘ DỰ ÁN
    const suggestedName = `${currentExternalProject.name}_Toàn bộ dự án`;
    const batchName = prompt("Nhập tên batch:", suggestedName);

    if (!batchName) {
      return;
    }

    showToast("Đang tạo batch từ toàn bộ dự án...", "info");

    const requestData = {
      project_id: currentExternalProject.id,
      project_name: currentExternalProject.name,
      department_id: "", // Empty = toàn bộ dự án
      department_name: "",
      folder_path: "", // Empty = từ root
      batch_name: batchName, // ✅ SỬ DỤNG CUSTOM NAME
    };

    const response = await fetch("/api/external/create-batch-from-folder", {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify(requestData),
    });

    const data = await response.json();

    if (data.success) {
      showToast(`✅ ${data.message}`, "success");

      // Close modal
      bootstrap.Modal.getInstance(
        document.getElementById("externalProjectsModal"),
      ).hide();

      // Refresh batches list
      loadBatches();

      // Show detailed results
      setTimeout(() => {
        alert(
          `Batch "${data.batch_name}" từ toàn bộ dự án đã tạo thành công!\n\n` +
            `📊 Kết quả:\n` +
            `- Quét tổng cộng: ${data.total_scanned_files} files\n` +
            `- Tạo thành công: ${data.created_documents} documents\n` +
            `- Thất bại: ${data.failed_files?.length || 0} files\n\n` +
            `Batch sẽ xuất hiện trong danh sách và có thể bắt đầu trích xuất.`,
        );
      }, 1000);
    } else {
      throw new Error(data.error || "Không thể tạo batch từ toàn bộ dự án");
    }
  } catch (error) {
    showToast("❌ Lỗi: " + error.message, "danger");
  }
}
function generateExternalBatchNamePreview() {
  let nameParts = [currentExternalProject.name];

  if (currentFolderPath) {
    const folderParts = currentFolderPath
      .split("/")
      .filter((part) => part.trim());
    if (folderParts.length > 0) {
      nameParts = nameParts.concat(folderParts);
    } else {
      nameParts.push("Root");
    }
  } else {
    nameParts.push("Toàn bộ dự án");
  }

  return nameParts.join("_");
}
