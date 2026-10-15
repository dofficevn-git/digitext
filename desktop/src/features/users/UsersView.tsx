// Copyright (c) 2026 DigiText. All rights reserved.
// Quản trị người dùng và dự án: tài khoản (tạo / sửa / đổi mật khẩu / phân quyền / khoá), dự án (tạo, gán quản trị),
// số liệu nhân viên gửi về. Dữ liệu tự cập nhật theo máy chủ, không nhấp nháy khi tải lại.

import { useCallback, useMemo, useState } from "react";
import { V41t } from "@/live/useLive";
import { V460 } from "@/state";
import { V3z0, V3za } from "@/domain/permissions";
import type { V3yx } from "@/domain/permissions";
import type { V3xs } from "@/domain/types";
import { V4ac, V4ad, V4ae, V4af, V4ag, V4ai, V4aj, V4ak, V4al, V4an, V4ao } from "@/api/users";
import type { V4a8, V4a9, V4ah } from "@/api/users";
import { V4r3, type V4r0 } from "@/api/stats";
import { V4r5, V4r6, V4r8, type V4r4 } from "./projectAdmins";
import ProjectPurgeDialog from "./ProjectPurgeDialog";
import { Icon } from "@/components/Icon";
import { Badge, Button, Card, Checkbox, confirm, Dialog, ErrorBanner, Field, FormGrid, Help, IconButton, Input, KpiRow, KpiTile, Menu, notifyError, PageHeader, Select, Table, Tabs, Tip, toast, Toolbar, type Column } from "@/ui";

const n = (v: number) => v.toLocaleString("vi-VN");
const mb = (b: number) => `${(b / 1_048_576).toFixed(1)} MB`;

type Tab = "users" | "projects" | "stats";
type DialogState = null | { kind: "create" } | { kind: "edit"; user: V4a8 } | { kind: "password"; user: V4a8 } | { kind: "perms"; user: V4a8 } | { kind: "project" };

export default function V4ra() {
  const { session } = V460();
  const isSuper = session?.role === "super_admin";
  const me = session?.userId ?? "";
  const [tab, setTab] = useState<Tab>("users");
  const [dlg, setDlg] = useState<DialogState>(null);
  const [xoaDuAn, setXoaDuAn] = useState<string | null>(null);
  const [q, setQ] = useState("");
  const [roleF, setRoleF] = useState("");
  const [projF, setProjF] = useState("");

  const load = useCallback(async () => {
    const [users, projects] = await Promise.all([V4ac(), V4ak()]);
    const stats = await V4r3().catch(() => [] as V4r0[]);
    return { users, projects, stats };
  }, []);
  const { data, error, loading, refreshing, reload } = V41t(load, { topics: ["users", "projects", "stats"] });
  const users = useMemo(() => data?.users ?? [], [data]);
  const projects = useMemo(() => data?.projects ?? [], [data]);
  const stats = useMemo(() => data?.stats ?? [], [data]);

  const projectName = useMemo(() => {
    const m = new Map(projects.map((p) => [p.id, p.name]));
    return (id: string | null) => (id ? (m.get(id) ?? session?.projectName ?? id) : "—");
  }, [projects, session]);

  const shownUsers = useMemo(() => {
    const k = q.trim().toLowerCase();
    return users.filter((u) => (!roleF || u.role === roleF) && (!projF || u.projectId === projF) && (!k || u.displayName.toLowerCase().includes(k) || u.username.toLowerCase().includes(k) || (u.email ?? "").toLowerCase().includes(k)));
  }, [users, q, roleF, projF]);

  const toggleActive = async (u: V4a8) => {
    if (u.isActive) {
      const ok = await confirm({ title: `Vô hiệu hoá tài khoản ${u.username}?`, body: "Người này sẽ không đăng nhập được nữa cho tới khi kích hoạt lại.", confirmLabel: "Vô hiệu hoá", danger: true });
      if (!ok) return;
    }
    try {
      await V4af(u.id, !u.isActive);
      toast.success(u.isActive ? `Đã vô hiệu hoá ${u.username}.` : `Đã kích hoạt ${u.username}.`);
      reload();
    } catch (e) {
      notifyError(e, u.isActive ? "vô hiệu hoá tài khoản" : "kích hoạt tài khoản");
    }
  };

  const userCols: Column<V4a8>[] = [
    {
      key: "displayName",
      header: "Tài khoản",
      sortable: true,
      cell: (u) => (
        <div className={u.isActive ? undefined : "opacity-60"}>
          <div className="font-medium text-fg-strong">{u.displayName}</div>
          <div className="text-xs text-fg-subtle">
            {u.username}
            {u.email ? ` · ${u.email}` : ""}
          </div>
        </div>
      ),
    },
    { key: "role", header: "Vai trò", sortable: true, cell: (u) => V3za(u.role), width: 150 },
    { key: "projectId", header: "Dự án", sortable: true, sortValue: (u) => projectName(u.projectId), cell: (u) => projectName(u.projectId) },
    { key: "isActive", header: "Trạng thái", width: 120, cell: (u) => <Badge tone={u.isActive ? "ok" : "gray"}>{u.isActive ? "Hoạt động" : "Đã khoá"}</Badge> },
  ];

  return (
    <section className="view active">
      <PageHeader
        title="Người dùng và dự án"
        description={isSuper ? "Toàn hệ thống." : `Các dự án bạn phụ trách (${n(projects.length)}).`}
        actions={
          <>
            <IconButton label="Tải lại" icon="lamMoi" variant="secondary" loading={refreshing} onClick={reload} />
            {tab === "users" && (
              <Button variant="primary" icon="them" onClick={() => setDlg({ kind: "create" })}>
                Thêm người dùng
              </Button>
            )}
            {tab === "projects" && (
              <Button variant="primary" icon="them" onClick={() => setDlg({ kind: "project" })}>
                Tạo dự án
              </Button>
            )}
          </>
        }
      />
      {session?.mode === "offline" && <ErrorBanner error="Đang ngoại tuyến. Quản lý người dùng cần kết nối máy chủ." className="mb-4" />}
      {error && data && <ErrorBanner error={error} onRetry={reload} className="mb-4" />}

      <Tabs<Tab>
        value={tab}
        onChange={setTab}
        items={[
          { value: "users", label: "Người dùng", count: users.length },
          { value: "projects", label: "Dự án", count: projects.length },
          { value: "stats", label: "Số liệu nhân viên" },
        ]}
      />

      {tab === "users" && (
        <Card padding="none" className="mt-3">
          <div className="border-b border-border-subtle p-3">
            <Toolbar>
              <Input size="sm" prefix="tim" placeholder="Tìm tên, tài khoản, email" value={q} onChange={(e) => setQ(e.target.value)} className="w-64" aria-label="Tìm người dùng" />
              <Select size="sm" value={roleF} onChange={setRoleF} placeholder="Mọi vai trò" options={(["super_admin", "admin", "user", "customer"] as V3xs[]).map((r) => ({ value: r, label: V3za(r) }))} className="w-44" aria-label="Lọc vai trò" />
              {projects.length > 1 && <Select size="sm" value={projF} onChange={setProjF} placeholder="Mọi dự án" options={projects.map((p) => ({ value: p.id, label: `${p.code} · ${p.name}` }))} className="w-56" aria-label="Lọc dự án" />}
            </Toolbar>
          </div>
          <Table
            columns={userCols}
            rows={shownUsers}
            getRowId={(u) => u.id}
            state={{ loading, error: data ? undefined : error, onRetry: reload, empty: users.length === 0 ? { title: "Chưa có tài khoản", action: <Button variant="primary" icon="them" onClick={() => setDlg({ kind: "create" })}>Thêm người dùng</Button> } : { title: "Không có tài khoản khớp bộ lọc" } }}
            rowActions={(u) => (
              <Menu
                size="sm"
                items={[
                  { label: "Sửa thông tin", icon: "sua", onSelect: () => setDlg({ kind: "edit", user: u }) },
                  { label: "Đổi mật khẩu", icon: "khoa", onSelect: () => setDlg({ kind: "password", user: u }) },
                  { label: "Phân quyền", icon: "quanTri", onSelect: () => setDlg({ kind: "perms", user: u }) },
                  "-",
                  { label: u.isActive ? "Vô hiệu hoá" : "Kích hoạt", icon: u.isActive ? "daHuy" : "duyet", danger: u.isActive, onSelect: () => void toggleActive(u), disabled: u.id === me },
                ]}
              />
            )}
          />
        </Card>
      )}

      {tab === "projects" && (
        <ProjectsTab
          projects={projects}
          admins={users.filter((u) => u.role === "admin")}
          isSuper={!!isSuper}
          me={me}
          loading={loading}
          onAdd={async (pid, uid) => {
            try {
              await V4an(pid, uid);
              reload();
            } catch (e) {
              notifyError(e, "gán quản trị dự án");
            }
          }}
          onRemove={async (pid, uid) => {
            try {
              await V4ao(pid, uid);
              reload();
            } catch (e) {
              notifyError(e, "gỡ quản trị dự án");
            }
          }}
          onCreate={() => setDlg({ kind: "project" })}
          onPurge={(pid) => setXoaDuAn(pid)}
        />
      )}

      {tab === "stats" && <StatsTab stats={stats} projects={projects} users={users} isSuper={!!isSuper} projectName={projectName} />}

      {(dlg?.kind === "create" || dlg?.kind === "edit") && (
        <UserDialog
          mode={dlg.kind}
          isSuper={!!isSuper}
          projects={projects}
          user={dlg.kind === "edit" ? dlg.user : undefined}
          onClose={() => setDlg(null)}
          onDone={() => {
            setDlg(null);
            reload();
          }}
        />
      )}
      {dlg?.kind === "password" && <PasswordDialog user={dlg.user} onClose={() => setDlg(null)} />}
      {dlg?.kind === "perms" && <PermsDialog user={dlg.user} onClose={() => setDlg(null)} />}
      {xoaDuAn && <ProjectPurgeDialog projectId={xoaDuAn} onClose={() => setXoaDuAn(null)} onDone={reload} />}
      {dlg?.kind === "project" && (
        <ProjectDialog
          onClose={() => setDlg(null)}
          onDone={() => {
            setDlg(null);
            reload();
          }}
        />
      )}
    </section>
  );
}

// ---------------- Dự án
function ProjectsTab({ projects, admins, isSuper, me, loading, onAdd, onRemove, onCreate, onPurge }: { projects: V4a9[]; admins: V4a8[]; isSuper: boolean; me: string; loading: boolean; onAdd: (pid: string, uid: string) => void; onRemove: (pid: string, uid: string) => void; onCreate: () => void; onPurge: (pid: string) => void }) {
  const nameOf = (id: string) => {
    const a = admins.find((x) => x.id === id);
    return a ? `${a.displayName} (${a.username})` : id;
  };
  const cols: Column<V4a9>[] = [
    { key: "code", header: "Mã", width: 110, sortable: true, cell: (p) => <span className="font-semibold text-fg-strong">{p.code}</span> },
    { key: "name", header: "Tên dự án", sortable: true },
    {
      key: "admins",
      header: "Quản trị phụ trách",
      className: "whitespace-normal",
      cell: (p) => {
        const ctx: V4r4 = { currentUserId: me, isSuper, adminIds: p.adminIds, isProjectAdmin: p.adminIds.includes(me) };
        const addable = V4r8(admins, p.adminIds);
        return (
          <div className="flex flex-wrap items-center gap-1.5">
            {p.adminIds.length === 0 && <span className="text-fg-subtle">Chưa gán</span>}
            {p.adminIds.map((id) => {
              const why = V4r6(ctx, id);
              return (
                <span key={id} className="inline-flex items-center gap-1 rounded-full border border-border bg-gray-50 pl-2 pr-0.5 text-sm">
                  {nameOf(id)}
                  <Tip text={why ?? "Gỡ khỏi dự án"}>
                    <button type="button" className="grid size-5 place-items-center rounded-full text-fg-subtle hover:bg-gray-200 hover:text-fg disabled:opacity-40" disabled={why !== null} aria-label={`Gỡ ${nameOf(id)} khỏi dự án`} onClick={() => onRemove(p.id, id)}>
                      <Icon name="dong" size={12} />
                    </button>
                  </Tip>
                </span>
              );
            })}
            {V4r5(ctx) && addable.length > 0 && (
              <Select size="sm" value="" onChange={(v) => v && onAdd(p.id, v)} placeholder="Thêm quản trị…" options={addable.map((a) => ({ value: a.id, label: `${a.displayName} (${a.username})` }))} className="w-48" aria-label="Thêm quản trị" />
            )}
          </div>
        );
      },
    },
    { key: "isActive", header: "Trạng thái", width: 110, cell: (p) => <Badge tone={p.isActive ? "ok" : "gray"}>{p.isActive ? "Hoạt động" : "Đóng"}</Badge> },
    ...(isSuper
      ? [
          {
            key: "purge",
            header: "",
            width: 56,
            align: "right" as const,
            cell: (p: V4a9) => <Menu items={[{ label: "Xoá dữ liệu dự án…", icon: "xoa", danger: true, onSelect: () => onPurge(p.id) }]} />,
          },
        ]
      : []),
  ];
  return (
    <Card padding="none" className="mt-3">
      <Table columns={cols} rows={projects} getRowId={(p) => p.id} state={{ loading, empty: { title: "Chưa có dự án", action: <Button variant="primary" icon="them" onClick={onCreate}>Tạo dự án</Button> } }} />
    </Card>
  );
}

// ---------------- Số liệu nhân viên
interface Tot {
  users: number;
  pages: number;
  hoSo: number;
  pdf: number;
  files: number;
  bytesUp: number;
}
const tot0 = (): Tot => ({ users: 0, pages: 0, hoSo: 0, pdf: 0, files: 0, bytesUp: 0 });
function add(t: Tot, s: V4r0): Tot {
  t.users += 1;
  t.pages += s.pagesScanned;
  t.hoSo += s.hoSoCount;
  t.pdf += s.pdfCount;
  t.files += s.filesUploaded;
  t.bytesUp += s.bytesUploaded;
  return t;
}

function StatsTab({ stats, projects, users, isSuper, projectName }: { stats: V4r0[]; projects: V4a9[]; users: V4a8[]; isSuper: boolean; projectName: (id: string | null) => string }) {
  const sum = stats.reduce(add, tot0());
  const cols: Column<V4r0>[] = [
    { key: "displayName", header: "Nhân viên", sortable: true, cell: (s) => <div><div className="font-medium">{s.displayName}</div><div className="text-xs text-fg-subtle">{s.username}</div></div> },
    { key: "projectId", header: "Dự án", cell: (s) => projectName(s.projectId), sortValue: (s) => projectName(s.projectId), sortable: true },
    { key: "pagesScanned", header: "Trang quét", align: "right", sortable: true, cell: (s) => n(s.pagesScanned) },
    { key: "hoSoCount", header: "Hồ sơ", align: "right", sortable: true },
    { key: "pdfCount", header: "PDF", align: "right", sortable: true },
    { key: "filesUploaded", header: "Tệp đã gửi", align: "right", sortable: true, cell: (s) => <b>{n(s.filesUploaded)}</b> },
    { key: "bytesUploaded", header: "Dung lượng", align: "right", sortable: true, cell: (s) => mb(s.bytesUploaded) },
    { key: "reportedAt", header: "Cập nhật", sortable: true, cell: (s) => <span className="text-fg-subtle">{s.reportedAt ? new Date(s.reportedAt).toLocaleString("vi-VN") : "—"}</span> },
  ];
  const byAdmin = useMemo(() => {
    if (!isSuper) return [];
    const NONE = "__none__";
    const pAdmins = new Map(projects.map((p) => [p.id, p.adminIds]));
    const aName = new Map(users.filter((u) => u.role === "admin" || u.role === "super_admin").map((u) => [u.id, `${u.displayName} (${u.username})`]));
    const g = new Map<string, { name: string; projects: Set<string>; t: Tot }>();
    for (const s of stats) {
      const ids = s.projectId ? (pAdmins.get(s.projectId) ?? []) : [];
      for (const id of ids.length ? ids : [NONE]) {
        let row = g.get(id);
        if (!row) {
          row = { name: id === NONE ? "Chưa gán quản trị" : (aName.get(id) ?? "Quản trị đã xoá"), projects: new Set(), t: tot0() };
          g.set(id, row);
        }
        if (s.projectId) row.projects.add(s.projectId);
        add(row.t, s);
      }
    }
    return [...g.values()].sort((a, b) => b.t.files - a.t.files);
  }, [isSuper, projects, users, stats]);

  return (
    <div className="mt-3 space-y-4">
      {stats.length === 0 ? (
        <Card>
          <p className="text-sm text-fg-muted">Chưa có số liệu. Nhân viên gửi số liệu về ở Cài đặt › Dữ liệu, hoặc số liệu tự về khi họ đưa tệp lên máy chủ.</p>
        </Card>
      ) : (
        <>
          <KpiRow>
            <KpiTile label={isSuper ? "Nhân viên (toàn hệ thống)" : "Nhân viên"} value={sum.users} />
            <KpiTile label="Trang quét" value={sum.pages} />
            <KpiTile label="Hồ sơ" value={sum.hoSo} />
            <KpiTile label="PDF" value={sum.pdf} />
            <KpiTile label="Tệp đã gửi" value={sum.files} tone="info" />
            <KpiTile label="Dung lượng" value={mb(sum.bytesUp)} />
          </KpiRow>
          {isSuper && byAdmin.length > 0 && (
            <Card title="Theo quản trị phụ trách" description="Dự án có nhiều quản trị được tính cho từng người, nên tổng các dòng có thể lớn hơn tổng chung." padding="none">
              <Table
                columns={[
                  { key: "name", header: "Quản trị", cell: (g) => <b>{g.name}</b> },
                  { key: "projects", header: "Dự án", align: "right", cell: (g) => g.projects.size },
                  { key: "users", header: "Nhân viên", align: "right", cell: (g) => g.t.users },
                  { key: "pages", header: "Trang quét", align: "right", cell: (g) => n(g.t.pages) },
                  { key: "hoSo", header: "Hồ sơ", align: "right", cell: (g) => g.t.hoSo },
                  { key: "files", header: "Tệp đã gửi", align: "right", cell: (g) => <b>{n(g.t.files)}</b> },
                  { key: "bytes", header: "Dung lượng", align: "right", cell: (g) => mb(g.t.bytesUp) },
                ]}
                rows={byAdmin}
                getRowId={(g) => g.name}
              />
            </Card>
          )}
          <Card title="Theo nhân viên" padding="none">
            <Table columns={cols} rows={stats} getRowId={(s) => s.userId} />
          </Card>
        </>
      )}
    </div>
  );
}

// ---------------- Hộp thoại người dùng
const ROLES: V3xs[] = ["super_admin", "admin", "user", "customer"];

function UserDialog({ mode, isSuper, projects, user, onClose, onDone }: { mode: "create" | "edit"; isSuper: boolean; projects: V4a9[]; user?: V4a8; onClose: () => void; onDone: () => void }) {
  const [username, setUsername] = useState(user?.username ?? "");
  const [displayName, setDisplayName] = useState(user?.displayName ?? "");
  const [email, setEmail] = useState(user?.email ?? "");
  const [password, setPassword] = useState("");
  const [role, setRole] = useState<V3xs>(user?.role ?? "user");
  const [projectId, setProjectId] = useState(user?.projectId ?? (!isSuper && projects.length === 1 ? projects[0].id : ""));
  const [busy, setBusy] = useState(false);
  const [touched, setTouched] = useState(false);
  const effRole: V3xs = isSuper ? role : "user";
  const askProject = mode === "create" && (effRole === "user" || (isSuper && effRole === "admin"));
  const errs = {
    username: mode === "create" && !username.trim() ? "Chưa nhập tên đăng nhập." : undefined,
    displayName: !displayName.trim() ? "Chưa nhập tên hiển thị." : undefined,
    password: mode === "create" && password.length < 8 ? "Mật khẩu cần ít nhất 8 ký tự." : undefined,
  };
  const valid = !errs.username && !errs.displayName && !errs.password;

  const save = async () => {
    setTouched(true);
    if (!valid || busy) return;
    setBusy(true);
    try {
      if (mode === "create") {
        const created = await V4ad({ username: username.trim(), password, displayName: displayName.trim(), email: email.trim() || null, role: effRole, projectId: askProject ? projectId || null : null });
        if (isSuper && effRole === "admin" && projectId) await V4an(projectId, created.id);
        toast.success(`Đã tạo tài khoản ${username.trim()}.`);
      } else if (user) {
        await V4ae({ id: user.id, displayName: displayName.trim(), email: email.trim() || null });
        toast.success("Đã lưu thông tin tài khoản.");
      }
      onDone();
    } catch (e) {
      notifyError(e, mode === "create" ? "tạo tài khoản" : "lưu tài khoản");
    } finally {
      setBusy(false);
    }
  };

  return (
    <Dialog open onOpenChange={(o) => !o && onClose()} title={mode === "create" ? "Thêm người dùng" : `Sửa tài khoản ${user?.username}`} size="sm" footer={<><Button onClick={onClose}>Huỷ</Button><Button variant="primary" loading={busy} onClick={() => void save()}>{mode === "create" ? "Tạo tài khoản" : "Lưu"}</Button></>}>
      <form
        className="space-y-3"
        onSubmit={(e) => {
          e.preventDefault();
          void save();
        }}
      >
        <Field label="Tên đăng nhập" required error={touched ? errs.username : undefined}>
          <Input value={username} disabled={mode === "edit"} onChange={(e) => setUsername(e.target.value)} autoFocus={mode === "create"} autoComplete="off" />
        </Field>
        <Field label="Tên hiển thị" required error={touched ? errs.displayName : undefined}>
          <Input value={displayName} onChange={(e) => setDisplayName(e.target.value)} autoFocus={mode === "edit"} />
        </Field>
        <Field label="Email" hint="Tuỳ chọn">
          <Input type="email" value={email} onChange={(e) => setEmail(e.target.value)} />
        </Field>
        {mode === "create" && (
          <Field label="Mật khẩu" required hint="Ít nhất 8 ký tự" error={touched ? errs.password : undefined}>
            <Input type="password" value={password} onChange={(e) => setPassword(e.target.value)} autoComplete="new-password" />
          </Field>
        )}
        {mode === "create" && isSuper && (
          <Field label="Vai trò">
            <Select value={role} onChange={(v) => setRole(v as V3xs)} options={ROLES.map((r) => ({ value: r, label: V3za(r) }))} />
          </Field>
        )}
        {askProject && (
          <Field label="Dự án" hint={projects.length === 0 ? "Chưa có dự án nào, tài khoản sẽ không gắn dự án." : "Tuỳ chọn"}>
            <Select value={projectId} onChange={setProjectId} placeholder="Không gắn dự án" options={projects.map((p) => ({ value: p.id, label: `${p.code} · ${p.name}` }))} disabled={projects.length === 0} />
          </Field>
        )}
        <button type="submit" className="hidden" />
      </form>
    </Dialog>
  );
}

function PasswordDialog({ user, onClose }: { user: V4a8; onClose: () => void }) {
  const [p1, setP1] = useState("");
  const [p2, setP2] = useState("");
  const [busy, setBusy] = useState(false);
  const [touched, setTouched] = useState(false);
  const err1 = p1.length < 8 ? "Mật khẩu cần ít nhất 8 ký tự." : undefined;
  const err2 = p2 !== p1 ? "Hai lần nhập không khớp." : undefined;
  const save = async () => {
    setTouched(true);
    if (err1 || err2 || busy) return;
    setBusy(true);
    try {
      await V4ag(user.id, p1);
      toast.success(`Đã đổi mật khẩu cho ${user.username}. Mọi phiên đang đăng nhập của tài khoản này bị đăng xuất.`);
      onClose();
    } catch (e) {
      notifyError(e, "đổi mật khẩu");
    } finally {
      setBusy(false);
    }
  };
  return (
    <Dialog open onOpenChange={(o) => !o && onClose()} title={`Đổi mật khẩu ${user.username}`} description="Đổi mật khẩu sẽ đăng xuất mọi phiên đang hoạt động của tài khoản này." size="sm" footer={<><Button onClick={onClose}>Huỷ</Button><Button variant="primary" loading={busy} onClick={() => void save()}>Đặt mật khẩu</Button></>}>
      <form
        className="space-y-3"
        onSubmit={(e) => {
          e.preventDefault();
          void save();
        }}
      >
        <Field label="Mật khẩu mới" required hint="Ít nhất 8 ký tự" error={touched ? err1 : undefined}>
          <Input type="password" value={p1} onChange={(e) => setP1(e.target.value)} autoFocus autoComplete="new-password" />
        </Field>
        <Field label="Nhập lại" required error={touched ? err2 : undefined}>
          <Input type="password" value={p2} onChange={(e) => setP2(e.target.value)} autoComplete="new-password" />
        </Field>
        <button type="submit" className="hidden" />
      </form>
    </Dialog>
  );
}

function PermsDialog({ user, onClose }: { user: V4a8; onClose: () => void }) {
  const [sel, setSel] = useState<Set<V3yx> | null>(null);
  const [busy, setBusy] = useState(false);
  const load = useCallback(() => V4ai(user.id), [user.id]);
  const { data, error, loading, setData } = V41t(load, { topics: ["users"] });
  const eff = sel ?? new Set<V3yx>(data?.effective ?? []);
  const defaults = useMemo(() => new Set<V3yx>(data?.roleDefaults ?? []), [data]);
  const grantable = useMemo(() => new Set<V3yx>(data?.grantable ?? []), [data]);
  const granted = [...eff].filter((p) => !defaults.has(p));
  const revoked = [...defaults].filter((p) => !eff.has(p));
  const dirty = !!data && (eff.size !== data.effective.length || data.effective.some((p) => !eff.has(p)));

  const toggle = (p: V3yx) => {
    if (!grantable.has(p) && !defaults.has(p)) return;
    const next = new Set(eff);
    if (next.has(p)) next.delete(p);
    else next.add(p);
    setSel(next);
  };
  const save = async () => {
    if (!data || busy) return;
    setBusy(true);
    try {
      const d: V4ah = await V4aj(user.id, granted, revoked);
      setData(d);
      setSel(null);
      toast.success("Đã lưu quyền. Có hiệu lực khi người dùng đăng nhập lại.");
    } catch (e) {
      notifyError(e, "lưu quyền");
    } finally {
      setBusy(false);
    }
  };

  return (
    <Dialog
      open
      onOpenChange={(o) => !o && onClose()}
      title={`Phân quyền ${user.username}`}
      description={`Vai trò ${V3za(user.role)} có sẵn bộ quyền mặc định. Tích để cấp thêm, bỏ tích để gỡ. Ô mờ là quyền bạn không được cấp.`}
      size="lg"
      footer={
        <>
          <Button onClick={() => data && setSel(new Set(data.roleDefaults))} disabled={!data || busy}>
            Về mặc định vai trò
          </Button>
          <Toolbar.Spacer />
          <span className="self-center text-sm text-fg-subtle">
            {granted.length} cấp thêm · {revoked.length} gỡ
          </span>
          <Button onClick={onClose}>Đóng</Button>
          <Button variant="primary" loading={busy} disabled={!dirty} onClick={() => void save()}>
            Lưu quyền
          </Button>
        </>
      }
    >
      {loading && !data ? (
        <p className="text-sm text-fg-subtle">Đang tải quyền…</p>
      ) : error && !data ? (
        <ErrorBanner error={error} />
      ) : (
        <FormGrid cols={2}>
          {V3z0.map((g) => (
            <section key={g.key}>
              <h4 className="mb-1 text-sm font-semibold text-fg-strong">{g.label}</h4>
              <ul className="space-y-0.5">
                {g.perms.map(({ perm, label }) => {
                  const on = eff.has(perm);
                  const isDef = defaults.has(perm);
                  const can = grantable.has(perm) || isDef;
                  return (
                    <li key={perm} className={can ? undefined : "opacity-50"}>
                      <Field label={<span className="flex items-center gap-1.5 text-fg">{label}{on && !isDef && <Badge tone="ok">cấp thêm</Badge>}{!on && isDef && <Badge tone="err">đã gỡ</Badge>}</span>} inline>
                        <Checkbox checked={on} disabled={!can} onChange={() => toggle(perm)} />
                      </Field>
                    </li>
                  );
                })}
              </ul>
            </section>
          ))}
        </FormGrid>
      )}
      <Help className="mt-4" title="Khi nào quyền có hiệu lực">
        <p>Quyền mới áp dụng khi người dùng đăng nhập lại, hoặc khi họ bấm "Kiểm tra kết nối" trong Cài đặt › Máy chủ.</p>
      </Help>
    </Dialog>
  );
}

function ProjectDialog({ onClose, onDone }: { onClose: () => void; onDone: () => void }) {
  const [code, setCode] = useState("");
  const [name, setName] = useState("");
  const [busy, setBusy] = useState(false);
  const [touched, setTouched] = useState(false);
  const save = async () => {
    setTouched(true);
    if (!code.trim() || !name.trim() || busy) return;
    setBusy(true);
    try {
      await V4al(code.trim(), name.trim());
      toast.success(`Đã tạo dự án ${code.trim()}.`);
      onDone();
    } catch (e) {
      notifyError(e, "tạo dự án");
    } finally {
      setBusy(false);
    }
  };
  return (
    <Dialog open onOpenChange={(o) => !o && onClose()} title="Tạo dự án" size="sm" footer={<><Button onClick={onClose}>Huỷ</Button><Button variant="primary" loading={busy} onClick={() => void save()}>Tạo dự án</Button></>}>
      <form
        className="space-y-3"
        onSubmit={(e) => {
          e.preventDefault();
          void save();
        }}
      >
        <Field label="Mã dự án" required hint="Ngắn, không dấu, ví dụ STC" error={touched && !code.trim() ? "Chưa nhập mã." : undefined}>
          <Input value={code} onChange={(e) => setCode(e.target.value.toUpperCase())} autoFocus autoComplete="off" />
        </Field>
        <Field label="Tên dự án" required error={touched && !name.trim() ? "Chưa nhập tên." : undefined}>
          <Input value={name} onChange={(e) => setName(e.target.value)} />
        </Field>
        <button type="submit" className="hidden" />
      </form>
    </Dialog>
  );
}
