<!-- code-review-graph MCP tools -->
## MCP Tools: code-review-graph

**This project has a knowledge graph. Start with the code-review-graph
MCP tools to narrow scope, then read the source.** The graph is cheaper than scanning files and
gives you structural context (callers, dependents, test coverage) that file search cannot.

### When to use graph tools FIRST

- **Exploring code**: `semantic_search_nodes_tool` or `query_graph_tool` instead of Grep
- **Understanding impact**: `get_impact_radius_tool` instead of manually tracing imports
- **Code review**: `detect_changes_tool` + `get_review_context_tool` instead of reading entire files
- **Finding relationships**: `query_graph_tool` with callers_of/callees_of/imports_of/tests_for
- **Architecture questions**: `get_architecture_overview_tool` + `list_communities_tool`

### Verify in the source

- Narrow scope with the graph, then read the source. Do not change code from graph output alone.
- For any non-trivial change, read the implementation and the relevant tests before concluding.
- Verify the exact source when touching behavior, database logic, migrations, retries, fallbacks,
  recovery, or compatibility code.
- When the graph and the source disagree, the source wins. The graph may be stale or may not
  model that relationship.
- An empty graph result can mean "not indexed" or "not statically visible", not "does not exist".

### Key Tools

| Tool | Use when |
| ------ | ---------- |
| `detect_changes_tool` | Reviewing code changes — gives risk-scored analysis |
| `get_review_context_tool` | Need source snippets for review — token-efficient |
| `get_impact_radius_tool` | Understanding blast radius of a change |
| `get_affected_flows_tool` | Finding which execution paths are impacted |
| `query_graph_tool` | Tracing callers, callees, imports, tests, dependencies |
| `semantic_search_nodes_tool` | Finding functions/classes by name or keyword |
| `get_architecture_overview_tool` | Understanding high-level codebase structure |
| `refactor_tool` | Planning renames, finding dead code |

### Workflow

1. The graph auto-updates on file changes (via hooks).
2. Use `detect_changes_tool` for code review.
3. Use `get_affected_flows_tool` to understand impact.
4. Use `query_graph_tool` pattern="tests_for" to check coverage.
<!-- /code-review-graph MCP tools -->

---

## Dự án này là gì

Hệ thống số hoá & quản lý tài liệu lưu trữ (nghiệp vụ lưu trữ nhà nước VN):
quét → nộp lưu → OCR/KIE → duyệt → đóng gói AIP (METS/PREMIS/EAD/TT02) →
ký số PAdES-LTV → kho WORM → khai thác / tiêu huỷ.

**Bản đồ đầy đủ: [docs/PROJECT-MAP.md](docs/PROJECT-MAP.md) — đọc file đó trước
khi khảo sát code.** Nó mô tả 14 crate Rust, desktop Tauri, pipeline OCR/ML,
bề mặt HTTP, hạ tầng, và các bẫy đã biết.

## ⚠️ Source đã bị obfuscate

Toàn bộ mã trong repo là **output đã obfuscate** của [dev/obfuscate/](dev/obfuscate/):

- Định danh nội bộ bị đổi tên: Rust `f_*` / `T*` / `r_*` / `C_*`, TS `V4*`, Python `p_*` / `q_*`.
  Tên module, đường dẫn file và `pub` API **còn nguyên** — đó là chỗ bám để hiểu code.
- Chuỗi tiếng Việt bị mã hoá base64 (`echo '<chuỗi>' | base64 -d` để đọc; chuỗi
  thường bị cắt mảnh quanh `{placeholder}`).
- Comment/docstring đã bị xoá sạch — đừng đi tìm chú thích, không có.

Đừng "sửa" tên biến khó hiểu thành tên đẹp: chúng là output của công cụ, sửa tay
sẽ lệch với lần obfuscate sau.

## Đang làm dở

Việc dở dang, hai máy, và những file phải mang tay sang máy mới:
**[docs/HANDOFF.md](docs/HANDOFF.md)** — đọc trước khi làm tiếp bất cứ gì
liên quan tới dịch vụ OCR, web KIE, hay fine-tune VietOCR.

## Bẫy đã biết

- **Migrations Postgres ở `migrations/0001`–`0023`**, áp tay theo thứ tự (không có `_sqlx_migrations`
  tự động, không có cache `.sqlx/`). Ghi chú cũ "không có migrations" là sai (sửa 2/11/2026).
- **Không có `query!` macro** — SQL chỉ nổ lúc chạy, không nổ lúc biên dịch.
- **Quyền cài đặt hai nơi**: `digitext-core/src/permission.rs` và
  `desktop/src/domain/permissions.ts`. Sửa một bên phải sửa bên kia. Từ 22/10/2026 bộ quyền
  THẬT của từng vai trò nằm trong bảng `roles` (migration 0022) — code chỉ là mặc định để seed.
- **Graph không phân giải được lời gọi hàm trong Rust** (0 CALLS edge trên 210 file
  `.rs`). Kết quả `impact`/`query` trên `.rs` phải đối chiếu source.
- **Không có test nào cho code sản phẩm.** `crates/`, `desktop/src`, `ml/` đều 0 test;
  3 file test duy nhất là của công cụ obfuscate. Không có lưới an toàn khi sửa —
  đặc biệt ở `sign-verify`, `package`, `wipe`, `disposal`.

## Đây là bản chụp đã làm sạch để giao đi

Test bị gỡ sạch, file `.sql`/migrations bị loại, đúng 1 commit không lịch sử.
Tồn tại một repo gốc có tên thật + comment + test + migrations ở nơi khác.
**Nếu đây là bản giao thì sửa ở đây sẽ bị ghi đè ở lần làm sạch sau** — xác nhận
repo gốc trước khi sửa. Chi tiết ở mục 0 của [docs/PROJECT-MAP.md](docs/PROJECT-MAP.md).
