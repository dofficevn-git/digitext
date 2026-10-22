// Copyright (c) 2026 DigiText. All rights reserved.
// Vai trò và quyền (22/10/2026): bộ quyền của từng vai trò là dữ liệu sửa được — tạo người dùng chỉ việc chọn vai trò.
// Trái: danh sách vai trò (hệ thống trước, rồi tuỳ chỉnh). Phải: bảng quyền theo nhóm nghiệp vụ, ô ngoài tầng của vai
// trò gốc bị mờ. Tạo vai trò = tên + vai trò gốc + sao chép bộ quyền từ vai trò khác. Quản trị hệ thống luôn toàn
// quyền nên không sửa được.

import { useCallback, useEffect, useMemo, useState } from "react";
import { V41t } from "@/live/useLive";
import { V460 } from "@/state";
import { V3z0, V3za, type V3yx } from "@/domain/permissions";
import type { V3xs } from "@/domain/types";
import { roleCreate, roleDelete, roleReset, roleUpdate, rolesList, slugRole, type RoleInfo } from "@/api/roles";
import { Icon } from "@/components/Icon";
import { V462 as cn } from "@/lib/utils";
import { Badge, Button, Card, Checkbox, confirm, Dialog, ErrorBanner, Field, FormGrid, Help, Input, notifyError, PageHeader, Select, Skeleton, Textarea, Tip, toast } from "@/ui";

// Phạm vi dữ liệu do vai trò GỐC quyết định, không sửa được bằng bộ quyền: quản trị / nhân viên dự án
// chỉ làm việc trên dự án được gán (máy chủ chặn mọi dự án khác), quản lý và quản trị hệ thống thấy mọi dự án.
const PHAM_VI: Record<string, string> = {
  super_admin: "Phạm vi: mọi dự án",
  manager: "Phạm vi: mọi dự án",
  admin: "Phạm vi: chỉ dự án được gán",
  user: "Phạm vi: chỉ dự án được gán",
  customer: "Phạm vi: dự án được gán, chỉ xem",
};

const n = (v: number) => v.toLocaleString("vi-VN");
const BASE_ORDER: V3xs[] = ["super_admin", "manager", "admin", "user", "customer"];

function sameSet(a: readonly string[], b: readonly string[]) {
  if (a.length !== b.length) return false;
  const s = new Set(a);
  return b.every((x) => s.has(x));
}

export default function RolesView() {
  const { session } = V460();
  const myRole = session?.role ?? "user";
  const toanHeThong = myRole === "super_admin" || myRole === "manager";
  const coSua = toanHeThong && (session?.permissions ?? []).includes("user.grant_permissions");
  const load = useCallback(() => rolesList(), []);
  const { data, error, loading, reload } = V41t(load, { topics: ["users"], errorText: () => "Không tải được danh sách vai trò." });
  const roles = useMemo(() => data ?? [], [data]);
  const [chon, setChon] = useState<string>("");
  const role = roles.find((r) => r.code === chon) ?? roles[0];
  useEffect(() => {
    if (!chon && roles.length) setChon(roles[0].code);
  }, [roles, chon]);

  const [name, setName] = useState("");
  const [desc, setDesc] = useState("");
  const [perms, setPerms] = useState<V3yx[]>([]);
  const [busy, setBusy] = useState(false);
  const [tao, setTao] = useState(false);
  useEffect(() => {
    if (!role) return;
    setName(role.name);
    setDesc(role.description ?? "");
    setPerms(role.permissions);
  }, [role]);

  const khoa = !role || !coSua || role.code === "super_admin" || (myRole === "manager" && (role.baseRole === "super_admin" || role.baseRole === "manager"));
  const daDoi = !!role && (name.trim() !== role.name || (desc.trim() || "") !== (role.description ?? "") || !sameSet(perms, role.permissions));
  const allowed = useMemo(() => new Set<V3yx>(role?.allowed ?? []), [role]);
  const permSet = useMemo(() => new Set<V3yx>(perms), [perms]);

  const toggle = (p: V3yx, on: boolean) => setPerms((cur) => (on ? (cur.includes(p) ? cur : [...cur, p]) : cur.filter((x) => x !== p)));
  const toggleGroup = (items: V3yx[], on: boolean) => setPerms((cur) => {
    const ok = items.filter((p) => allowed.has(p));
    return on ? Array.from(new Set([...cur, ...ok])) : cur.filter((p) => !ok.includes(p));
  });

  const luu = async () => {
    if (!role || khoa || !daDoi) return;
    setBusy(true);
    try {
      await roleUpdate(role.code, { name: name.trim(), description: desc.trim() || null, permissions: perms.filter((p) => allowed.has(p)) });
      toast.success(`Đã lưu vai trò «${name.trim()}». Có hiệu lực khi người dùng đăng nhập lại.`);
      reload();
    } catch (e) {
      notifyError(e, "lưu vai trò");
    } finally {
      setBusy(false);
    }
  };
  const khoiPhuc = async () => {
    if (!role || !role.isSystem) return;
    const ok = await confirm({ title: `Khôi phục bộ quyền mặc định của «${role.name}»?`, body: "Mọi thay đổi trên vai trò này sẽ bị bỏ. Quyền cấp / chặn riêng của từng người vẫn giữ.", confirmLabel: "Khôi phục" });
    if (!ok) return;
    setBusy(true);
    try {
      await roleReset(role.code);
      toast.success("Đã khôi phục bộ quyền mặc định.");
      reload();
    } catch (e) {
      notifyError(e, "khôi phục vai trò");
    } finally {
      setBusy(false);
    }
  };
  const xoa = async () => {
    if (!role || role.isSystem) return;
    const ok = await confirm({ title: `Xoá vai trò «${role.name}»?`, body: role.userCount > 0 ? `Còn ${n(role.userCount)} tài khoản đang dùng — đổi vai trò cho họ trước.` : "Vai trò tuỳ chỉnh này sẽ bị xoá.", confirmLabel: "Xoá", danger: true });
    if (!ok || role.userCount > 0) return;
    setBusy(true);
    try {
      await roleDelete(role.code);
      toast.success("Đã xoá vai trò.");
      setChon("");
      reload();
    } catch (e) {
      notifyError(e, "xoá vai trò");
    } finally {
      setBusy(false);
    }
  };

  return (
    <section className="view active">
      <PageHeader
        title="Vai trò và quyền"
        description="Mỗi vai trò có sẵn một bộ quyền; tạo tài khoản chỉ cần chọn vai trò. Quyền cấp / chặn riêng cho từng người vẫn làm ở Người dùng và dự án."
        actions={
          coSua && (
            <Button variant="primary" icon="them" onClick={() => setTao(true)}>
              Tạo vai trò
            </Button>
          )
        }
      />
      {error && !data && <ErrorBanner error={error} onRetry={reload} className="mb-4" />}
      <div className="grid gap-4 lg:grid-cols-12">
        <Card padding="none" className="lg:col-span-4" title="Vai trò">
          {loading && !data ? (
            <div className="p-3">
              <Skeleton.Rows rows={5} cols={1} />
            </div>
          ) : (
            <ul className="divide-y divide-border-subtle">
              {roles.map((r) => (
                <li key={r.code}>
                  <button type="button" className={cn("flex w-full items-start gap-3 px-4 py-2.5 text-left hover:bg-surface-hover", role?.code === r.code && "bg-brand-50")} onClick={() => setChon(r.code)} aria-current={role?.code === r.code ? "true" : undefined}>
                    <span className="min-w-0 flex-1">
                      <span className="flex items-center gap-2">
                        <span className="font-medium">{r.name}</span>
                        {r.isSystem ? <Badge tone="gray">Hệ thống</Badge> : <Badge tone="info">Tuỳ chỉnh</Badge>}
                      </span>
                      <span className="block text-xs text-fg-subtle">
                        {r.isSystem ? r.code : `${r.code} · nền ${r.baseRoleName}`} · {n(r.permissions.length)} quyền · {n(r.userCount)} tài khoản
                      </span>
                    </span>
                    <Icon name="sau" size={14} className="mt-1 flex-none text-fg-subtle" />
                  </button>
                </li>
              ))}
            </ul>
          )}
        </Card>

        <div className="space-y-4 lg:col-span-8">
          {role && (
            <Card
              title={
                <span className="flex flex-wrap items-center gap-2">
                  {role.name}
                  <Badge tone="gray">Nền: {role.baseRoleName}</Badge>
                  <Badge tone={role.baseRole === "admin" || role.baseRole === "user" ? "warn" : "gray"}>{PHAM_VI[role.baseRole]}</Badge>
                  {khoa && role.code === "super_admin" && <Badge tone="ok">Luôn toàn quyền</Badge>}
                </span>
              }
              description={role.isSystem ? "Vai trò hệ thống: sửa được bộ quyền, không xoá được. Khôi phục mặc định để về bộ quyền ban đầu." : "Vai trò tuỳ chỉnh: bộ quyền không vượt tầng của vai trò gốc (ô mờ)."}
              actions={
                <span className="flex items-center gap-2">
                  {role.isSystem && !khoa && (
                    <Button size="sm" icon="hoanTac" onClick={() => void khoiPhuc()} disabled={busy}>
                      Khôi phục mặc định
                    </Button>
                  )}
                  {!role.isSystem && !khoa && (
                    <Button size="sm" variant="danger" icon="xoa" onClick={() => void xoa()} disabled={busy}>
                      Xoá
                    </Button>
                  )}
                  {!khoa && (
                    <Button size="sm" variant="primary" icon="luu" loading={busy} disabled={!daDoi} onClick={() => void luu()}>
                      Lưu
                    </Button>
                  )}
                </span>
              }
            >
              {!khoa && (
                <FormGrid cols={2} className="mb-4">
                  <Field label="Tên vai trò" required>
                    <Input value={name} onChange={(e) => setName(e.target.value)} />
                  </Field>
                  <Field label="Mô tả" hint="Hiện khi chọn vai trò lúc tạo tài khoản">
                    <Input value={desc} onChange={(e) => setDesc(e.target.value)} />
                  </Field>
                </FormGrid>
              )}
              <div className="mb-2 flex items-center justify-between text-sm text-fg-muted">
                <span>
                  {n(permSet.size)} / {n(role.allowed.length)} quyền được chọn
                </span>
                {!khoa && (
                  <span className="flex gap-2">
                    <Button size="sm" variant="ghost" onClick={() => setPerms(role.allowed)}>
                      Chọn hết
                    </Button>
                    <Button size="sm" variant="ghost" onClick={() => setPerms([])}>
                      Bỏ hết
                    </Button>
                  </span>
                )}
              </div>
              <div className="grid gap-3 md:grid-cols-2">
                {V3z0.map((g) => {
                  const items = g.perms.map((i) => i.perm);
                  const ok = items.filter((p) => allowed.has(p));
                  const on = ok.filter((p) => permSet.has(p)).length;
                  return (
                    <div key={g.key} className="rounded-lg border border-border-subtle p-3">
                      <div className="mb-2 flex items-center gap-2">
                        {!khoa && ok.length > 0 ? (
                          <Checkbox checked={on === ok.length && ok.length > 0} onChange={(v) => toggleGroup(items, v)} text={g.label} />
                        ) : (
                          <span className="text-sm font-medium">{g.label}</span>
                        )}
                        <span className="ml-auto text-xs text-fg-subtle tabular-nums">
                          {on}/{ok.length}
                        </span>
                      </div>
                      <ul className="space-y-1">
                        {g.perms.map((it) => {
                          const duoc = allowed.has(it.perm);
                          const checked = permSet.has(it.perm);
                          const row = (
                            <li key={it.perm} className={cn("flex items-center gap-2 text-sm", !duoc && "opacity-45")}>
                              <Checkbox checked={checked} disabled={khoa || !duoc} onChange={(v) => toggle(it.perm, v)} text={it.label} />
                              <code className="ml-auto text-[11px] text-fg-subtle">{it.perm}</code>
                            </li>
                          );
                          return duoc ? row : <Tip key={it.perm} text={`Vượt tầng của vai trò gốc ${role.baseRoleName}`}>{row}</Tip>;
                        })}
                      </ul>
                    </div>
                  );
                })}
              </div>
            </Card>
          )}
          <Help title="Cách quyền được tính">
            <p>Quyền của một người = bộ quyền của vai trò, cộng quyền cấp riêng, trừ quyền chặn riêng (hai mục sau ở Người dùng và dự án › Phân quyền). Vai trò gốc quyết định tầng và phạm vi: Quản trị dự án và Nhân viên luôn gắn một dự án; Quản lý và Quản trị hệ thống thấy mọi dự án. Đổi bộ quyền có hiệu lực ngay trên máy chủ; máy trạm nhận khi người dùng đăng nhập lại hoặc làm mới phiên.</p>
          </Help>
        </div>
      </div>
      {tao && <TaoVaiTroDialog roles={roles} myRole={myRole} onClose={() => setTao(false)} onDone={(code) => { setTao(false); setChon(code); reload(); }} />}
    </section>
  );
}

function TaoVaiTroDialog({ roles, myRole, onClose, onDone }: { roles: RoleInfo[]; myRole: string; onClose: () => void; onDone: (code: string) => void }) {
  const bases: V3xs[] = (myRole === "super_admin" ? ["manager", "admin", "user"] : ["admin", "user"]) as V3xs[];
  const [name, setName] = useState("");
  const [code, setCode] = useState("");
  const [codeTouched, setCodeTouched] = useState(false);
  const [desc, setDesc] = useState("");
  const [base, setBase] = useState<V3xs>("user");
  const [copyFrom, setCopyFrom] = useState<string>("");
  const [busy, setBusy] = useState(false);
  useEffect(() => {
    if (!codeTouched) setCode(slugRole(name));
  }, [name, codeTouched]);
  const nguon = roles.filter((r) => BASE_ORDER.indexOf(r.baseRole) >= BASE_ORDER.indexOf(base));
  const errCode = !/^[a-z][a-z0-9_]{1,39}$/.test(code) ? "Mã: chữ thường, số, gạch dưới, 2–40 ký tự, bắt đầu bằng chữ." : roles.some((r) => r.code === code) ? "Mã đã tồn tại." : undefined;
  const valid = name.trim().length > 0 && !errCode;
  const tao = async () => {
    if (!valid || busy) return;
    setBusy(true);
    try {
      const src = roles.find((r) => r.code === (copyFrom || base));
      const r = await roleCreate({ code, name: name.trim(), description: desc.trim() || null, baseRole: base, permissions: src?.permissions ?? [] });
      toast.success(`Đã tạo vai trò «${r.name}». Chỉnh bộ quyền rồi bấm Lưu.`);
      onDone(r.code);
    } catch (e) {
      notifyError(e, "tạo vai trò");
    } finally {
      setBusy(false);
    }
  };
  return (
    <Dialog
      open
      onOpenChange={(o) => !o && !busy && onClose()}
      title="Tạo vai trò"
      description="Vai trò mới bắt đầu bằng bộ quyền sao chép từ một vai trò có sẵn; sửa tiếp ở bảng quyền sau khi tạo."
      size="md"
      footer={
        <>
          <Button onClick={onClose} disabled={busy}>
            Huỷ
          </Button>
          <Button variant="primary" icon="them" loading={busy} disabled={!valid} onClick={() => void tao()}>
            Tạo vai trò
          </Button>
        </>
      }
    >
      <div className="space-y-3">
        <Field label="Tên vai trò" required>
          <Input value={name} onChange={(e) => setName(e.target.value)} autoFocus placeholder="Ví dụ: Ký số viên" />
        </Field>
        <Field label="Mã" required hint="Dùng trong hệ thống, không đổi được sau khi tạo" error={name && errCode ? errCode : undefined}>
          <Input value={code} onChange={(e) => { setCodeTouched(true); setCode(e.target.value); }} />
        </Field>
        <Field label="Vai trò gốc" required hint="Quyết định tầng và phạm vi dự án; bộ quyền không vượt tầng này">
          <Select value={base} onChange={(v) => { setBase(v as V3xs); setCopyFrom(""); }} options={bases.map((b) => ({ value: b, label: V3za(b) }))} />
        </Field>
        <Field label="Sao chép bộ quyền từ" hint="Mặc định: bộ quyền của vai trò gốc">
          <Select value={copyFrom} onChange={setCopyFrom} placeholder={`Bộ quyền của ${V3za(base)}`} options={nguon.map((r) => ({ value: r.code, label: `${r.name} (${n(r.permissions.length)} quyền)` }))} />
        </Field>
        <Field label="Mô tả" hint="Tuỳ chọn">
          <Textarea value={desc} onChange={(e) => setDesc(e.target.value)} rows={2} />
        </Field>
      </div>
    </Dialog>
  );
}
