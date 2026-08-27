// Copyright (c) 2026 DigiText. All rights reserved.

import { useCallback, useMemo, useState } from "react";
import { V41p } from "@/live/useLive";
import { V460 } from "@/state";
import V4oq from "@/ui/Seg";
import { V3xq } from "@/api/ipc";
import { V3z0, V3za } from "@/domain/permissions";
import type { V3yx } from "@/domain/permissions";
import type { V3xs } from "@/domain/types";
import { V4al, V4ad, V4ai, V4ak, V4ac, V4ag, V4an, V4ao, V4af, V4aj, V4ae } from "@/api/users";
import type { V4a9, V4a8, V4ah } from "@/api/users";
import { V4r3, type V4r0 } from "@/api/stats";
import { V4r8, V4r5, V4r6, type V4r4 } from "./projectAdmins";
import { Icon } from "@/components/Icon";

const V4r9 = (V2hx: unknown) =>
  V2hx instanceof V3xq || V2hx instanceof Error ? V2hx.message : String(V2hx);

export default function V4ra() {
  const { session: V2hy } = V460();
  const V2hz = V2hy?.role === "super_admin";
  const V2i0 = V2hy?.userId ?? "";

  const [tab, V2i1] = useState<"users" | "projects" | "stats">("users");
  const [users, V2i2] = useState<V4a8[]>([]);
  const [V2i3, V2i4] = useState<V4a9[]>([]);
  const [V2i5, V2i6] = useState<V4r0[]>([]);
  const [V2i7, V2i8] = useState(true);
  const [V2i9, V2ia] = useState("");
  const [V2ib, V2ic] = useState<
    | null
    | { kind: "create" }
    | { kind: "edit"; user: V4a8 }
    | { kind: "password"; user: V4a8 }
    | { kind: "perms"; user: V4a8 }
  >(null);
  const [V2id, V2ie] = useState(false);

  const V2if = useMemo(() => {
    const m = new Map(V2i3.map((p) => [p.id, p.name]));
    return (V2ig: string | null) => (V2ig ? (m.get(V2ig) ?? V2hy?.projectName ?? V2ig) : "—");
  }, [V2i3, V2hy]);

  const reload = useCallback(async () => {
    V2i8(true);
    V2ia("");
    try {
      const [u, p] = await Promise.all([V4ac(), V4ak()]);
      V2i2(u);
      V2i4(p);
    } catch (V2ih) {
      V2ia(V4r9(V2ih));
    } finally {
      V2i8(false);
    }

    try {
      V2i6(await V4r3());
    } catch {
      V2i6([]);
    }
  }, []);

  V41p(reload, { topics: ["users", "projects", "stats"] });

  const V2ii = async (u: V4a8) => {
    if (!u.isActive) {
      try {
        await V4af(u.id, true);
        await reload();
      } catch (V2ij) {
        V2ia(V4r9(V2ij));
      }
      return;
    }
    if (
      !confirm(
        `VsO0IGhp4buHdSBob8OhIHTDoGkga2hv4bqjbiAi${u.username}Ij8gTmfGsOG7nWkgbsOgeSBz4bq9IGtow7RuZyDEkcSDbmcgbmjhuq1wIMSRxrDhu6NjIG7hu69hLg==`,
      )
    )
      return;
    try {
      await V4af(u.id, false);
      await reload();
    } catch (V2ik) {
      V2ia(V4r9(V2ik));
    }
  };

  return (
    <section className="view active">
      <div className="page-head">
        <h2>Quản lý người dùng</h2>
        <span className="sub">
          {V2hz
            ? "VG/DoG4gaOG7hyB0aOG7kW5n"
            : `ROG7sSDDoW4gYuG6oW4gcGjhu6UgdHLDoWNoICg=${V2i3.length})`}
          {" · "}
          {V2hy?.mode === "offline"
            ? "Y+G6p24ga+G6v3QgbuG7kWkgbcOheSBjaOG7pw=="
            : "dHLhu7FjIHR1eeG6v24="}
        </span>
      </div>

      <div className="settings-grid solo">
        <div className="set-body">
          <div className="users-toolbar">
            <V4oq
              value={tab}
              onChange={V2i1}
              options={[
                { value: "users", label: "TmfGsOG7nWkgZMO5bmc=" },
                { value: "projects", label: "ROG7sSDDoW4=" },
                { value: "stats", label: "VGjhu5FuZyBrw6o=" },
              ]}
            />
            <div className="spacer" />
            <button className="btn" onClick={() => void reload()} disabled={V2i7}>
              <Icon name="lamMoi" />
              Tải lại
            </button>
            {tab === "users" && (
              <button className="btn primary" onClick={() => V2ic({ kind: "create" })}>
                + Thêm người dùng
              </button>
            )}
            {tab === "projects" && (
              <button className="btn primary" onClick={() => V2ie(true)}>
                + Tạo dự án
              </button>
            )}
          </div>

          {V2i9 && <div className="login-error">{V2i9}</div>}
          {V2i7 && <p className="muted">Đang tải…</p>}

          {!V2i7 && tab === "users" && (
            <V4rb
              users={users}
              projectName={V2if}
              onEdit={(u) => V2ic({ kind: "edit", user: u })}
              onPassword={(u) => V2ic({ kind: "password", user: u })}
              onPerms={(u) => V2ic({ kind: "perms", user: u })}
              onToggle={(u) => void V2ii(u)}
            />
          )}

          {!V2i7 && tab === "projects" && (
            <V4rc
              projects={V2i3}
              admins={users.filter((u) => u.role === "admin")}
              isSuper={V2hz}
              currentUserId={V2i0}
              onAdd={async (V2il, V2im) => {
                try {
                  await V4an(V2il, V2im);
                  await reload();
                } catch (V2in) {
                  V2ia(V4r9(V2in));
                }
              }}
              onRemove={async (V2io, V2ip) => {
                try {
                  await V4ao(V2io, V2ip);
                  await reload();
                } catch (V2iq) {
                  V2ia(V4r9(V2iq));
                }
              }}
            />
          )}

          {!V2i7 && tab === "stats" && (
            <V4rj stats={V2i5} projects={V2i3} users={users} isSuper={!!V2hz} projectName={V2if} />
          )}
        </div>
      </div>

      {V2ib?.kind === "create" && (
        <V4rm
          mode="create"
          isSuper={!!V2hz}
          projects={V2i3}
          onClose={() => V2ic(null)}
          onDone={async () => {
            V2ic(null);
            await reload();
          }}
        />
      )}
      {V2ib?.kind === "edit" && (
        <V4rm
          mode="edit"
          isSuper={!!V2hz}
          projects={V2i3}
          user={V2ib.user}
          onClose={() => V2ic(null)}
          onDone={async () => {
            V2ic(null);
            await reload();
          }}
        />
      )}
      {V2ib?.kind === "password" && (
        <V4rn user={V2ib.user} onClose={() => V2ic(null)} onDone={() => V2ic(null)} />
      )}
      {V2ib?.kind === "perms" && <V4ro user={V2ib.user} onClose={() => V2ic(null)} />}
      {V2id && (
        <V4rp
          onClose={() => V2ie(false)}
          onDone={async () => {
            V2ie(false);
            await reload();
          }}
        />
      )}
    </section>
  );
}

function V4rb({
  users,
  projectName: V2ir,
  onEdit: V2is,
  onPassword: V2it,
  onPerms: V2iu,
  onToggle: V2iv,
}: {
  users: V4a8[];
  projectName: (V2iw: string | null) => string;
  onEdit: (u: V4a8) => void;
  onPassword: (u: V4a8) => void;
  onPerms: (u: V4a8) => void;
  onToggle: (u: V4a8) => void;
}) {
  if (users.length === 0) return <p className="muted">Chưa có tài khoản nào.</p>;
  return (
    <div className="tbl-scroll">
      <table className="tbl">
        <thead>
          <tr>
            <th>Tài khoản</th>
            <th>Vai trò</th>
            <th>Dự án</th>
            <th>Trạng thái</th>
            <th />
          </tr>
        </thead>
        <tbody>
          {users.map((u) => (
            <tr key={u.id} className={u.isActive ? undefined : "opacity-55"}>
              <td>
                <b>{u.displayName}</b>
                <small className="muted block">
                  {u.username}
                  {u.email ? ` · ${u.email}` : ""}
                </small>
              </td>
              <td>{V3za(u.role)}</td>
              <td>{V2ir(u.projectId)}</td>
              <td>
                <span className={`badge ${u.isActive ? "ok" : "gray"}`}>
                  {u.isActive ? "SG/huqF0IMSR4buZbmc=" : "xJDDoyBraG/DoQ=="}
                </span>
              </td>
              <td className="whitespace-nowrap text-right">
                <button className="btn sm" onClick={() => V2is(u)}>
                  Sửa
                </button>{" "}
                <button className="btn sm" onClick={() => V2it(u)}>
                  Đổi MK
                </button>{" "}
                <button className="btn sm" onClick={() => V2iu(u)}>
                  Phân quyền
                </button>{" "}
                <button className="btn sm" onClick={() => V2iv(u)}>
                  {u.isActive ? "VsO0IGhp4buHdSBob8Oh" : "S8OtY2ggaG/huqF0"}
                </button>
              </td>
            </tr>
          ))}
        </tbody>
      </table>
    </div>
  );
}

function V4rc({
  projects: V2ix,
  admins: V2iy,
  isSuper: V2iz,
  currentUserId: V2j0,
  onAdd: V2j1,
  onRemove: V2j2,
}: {
  projects: V4a9[];
  admins: V4a8[];
  isSuper: boolean;
  currentUserId: string;
  onAdd: (V2j3: string, V2j4: string) => void;
  onRemove: (V2j5: string, V2j6: string) => void;
}) {
  if (V2ix.length === 0)
    return <p className="muted">Chưa có dự án nào — bấm “Tạo dự án” để bắt đầu.</p>;
  const V2j7 = (V2j8: string) => {
    const a = V2iy.find((x) => x.id === V2j8);
    return a ? `${a.displayName} (${a.username})` : V2j8;
  };
  return (
    <div className="tbl-scroll">
      <table className="tbl">
        <thead>
          <tr>
            <th>Mã</th>
            <th>Tên dự án</th>
            <th>Admin phụ trách</th>
            <th>Trạng thái</th>
          </tr>
        </thead>
        <tbody>
          {V2ix.map((p) => {
            const V2j9: V4r4 = {
              currentUserId: V2j0,
              isSuper: V2iz,
              adminIds: p.adminIds,
              isProjectAdmin: p.adminIds.includes(V2j0),
            };
            const V2ja = V4r8(V2iy, p.adminIds);
            return (
              <tr key={p.id}>
                <td>
                  <b>{p.code}</b>
                </td>
                <td>{p.name}</td>
                <td>
                  <div className="admin-chips">
                    {p.adminIds.length === 0 && <span className="muted">— chưa gán —</span>}
                    {p.adminIds.map((V2jb) => {
                      const V2jc = V4r6(V2j9, V2jb);
                      return (
                        <span key={V2jb} className="chip">
                          {V2j7(V2jb)}
                          <button
                            type="button"
                            className="x"

                            disabled={V2jc !== null}
                            title={V2jc ?? `R+G7oSA=${V2j7(V2jb)}IGto4buPaSBk4buxIMOhbg==`}
                            onClick={() => V2j2(p.id, V2jb)}
                          >
                            ×
                          </button>
                        </span>
                      );
                    })}
                  </div>
                  {V4r5(V2j9) && V2ja.length > 0 && (
                    <select
                      className="role-sel"
                      value=""
                      onChange={(V2jd) => V2jd.target.value && V2j1(p.id, V2jd.target.value)}
                    >
                      <option value="">+ thêm admin…</option>
                      {V2ja.map((a) => (
                        <option key={a.id} value={a.id}>
                          {a.displayName} ({a.username})
                        </option>
                      ))}
                    </select>
                  )}
                </td>
                <td>
                  <span className={`badge ${p.isActive ? "ok" : "gray"}`}>
                    {p.isActive ? "SG/huqF0IMSR4buZbmc=" : "xJDDs25n"}
                  </span>
                </td>
              </tr>
            );
          })}
        </tbody>
      </table>
    </div>
  );
}

const V4rd = (n: number) => n.toLocaleString("vi-VN");
const V4re = (b: number) => `${(b / 1_048_576).toFixed(1)} MB`;

interface V4rf {
  users: number;
  pages: number;
  hoSo: number;
  pdf: number;
  files: number;
  bytesUp: number;
}
const V4rg = (): V4rf => ({ users: 0, pages: 0, hoSo: 0, pdf: 0, files: 0, bytesUp: 0 });
function V4rh(t: V4rf, s: V4r0): V4rf {
  t.users += 1;
  t.pages += s.pagesScanned;
  t.hoSo += s.hoSoCount;
  t.pdf += s.pdfCount;
  t.files += s.filesUploaded;
  t.bytesUp += s.bytesUploaded;
  return t;
}

function V4ri({
  label: V2je,
  value: V2jf,
  accent: V2jg,
}: {
  label: string;
  value: string;
  accent?: boolean;
}) {
  return (
    <div
      className={`card m-0 min-w-30 grow shrink basis-[130px] px-3.5 py-2.5${
        V2jg ? " border-brand-500" : ""
      }`}
    >
      <div className={`text-2xl leading-tight font-bold${V2jg ? " text-brand-500" : ""}`}>
        {V2jf}
      </div>
      <div className="muted text-sm mt-0.5">{V2je}</div>
    </div>
  );
}

function V4rj({
  stats: V2jh,
  projects: V2ji,
  users,
  isSuper: V2jj,
  projectName: V2jk,
}: {
  stats: V4r0[];
  projects: V4a9[];
  users: V4a8[];
  isSuper: boolean;
  projectName: (V2jl: string | null) => string;
}) {
  if (V2jh.length === 0)
    return (
      <p className="muted">
        Chưa có số liệu nào. Nhân viên gửi số liệu ở Cài đặt → “Máy chủ &amp; tải lên” → “Gửi số
        liệu về quản trị”, hoặc gửi PDF ở màn “Tải lên”.
      </p>
    );

  const V2jm = V2jh.reduce(V4rh, V4rg());

  return (
    <>
      {}
      <div className="flex gap-2.5 flex-wrap mb-4">
        <V4ri
          label={V2jj ? "TmjDom4gdmnDqm4gKHRvw6BuIEhUKQ==" : "TmjDom4gdmnDqm4="}
          value={V4rd(V2jm.users)}
        />
        <V4ri label="VHJhbmcgcXXDqXQ=" value={V4rd(V2jm.pages)} />
        <V4ri label="SOG7kyBzxqE=" value={V4rd(V2jm.hoSo)} />
        <V4ri label="UERGIMSRw7NuZyBnw7Np" value={V4rd(V2jm.pdf)} />
        <V4ri label="RmlsZSDEkcOjIGfhu61pIHbhu4E=" value={V4rd(V2jm.files)} accent />
        <V4ri label="RHVuZyBsxrDhu6NuZyDEkcOjIGfhu61p" value={V4re(V2jm.bytesUp)} />
      </div>

      {}
      {V2jj && <V4rk stats={V2jh} projects={V2ji} users={users} />}

      {}
      <h3 className="mt-1 mx-0 mb-2">Chi tiết theo nhân viên</h3>
      <div className="tbl-scroll">
        <table className="tbl">
          <thead>
            <tr>
              <th>Nhân viên</th>
              <th>Dự án</th>
              <th className="num">Trang quét</th>
              <th className="num">Hồ sơ</th>
              <th className="num">PDF</th>
              <th className="num">File gửi</th>
              <th className="num">DL gửi</th>
              <th>Cập nhật</th>
            </tr>
          </thead>
          <tbody>
            {V2jh.map((s) => (
              <tr key={s.userId}>
                <td>
                  <b>{s.displayName}</b>
                  <small className="muted block">{s.username}</small>
                </td>
                <td>{V2jk(s.projectId)}</td>
                <td className="num">{V4rd(s.pagesScanned)}</td>
                <td className="num">{s.hoSoCount}</td>
                <td className="num">{s.pdfCount}</td>
                <td className="num">
                  <b>{s.filesUploaded}</b>
                </td>
                <td className="num">{V4re(s.bytesUploaded)}</td>
                <td>
                  <span className="muted">
                    {s.reportedAt ? new Date(s.reportedAt).toLocaleString("vi-VN") : "—"}
                  </span>
                </td>
              </tr>
            ))}
            <tr className="font-semibold">
              <td colSpan={2}>Tổng ({V4rd(V2jm.users)} người)</td>
              <td className="num">{V4rd(V2jm.pages)}</td>
              <td className="num">{V2jm.hoSo}</td>
              <td className="num">{V2jm.pdf}</td>
              <td className="num">{V2jm.files}</td>
              <td className="num">{V4re(V2jm.bytesUp)}</td>
              <td />
            </tr>
          </tbody>
        </table>
      </div>
    </>
  );
}

function V4rk({
  stats: V2jn,
  projects: V2jo,
  users,
}: {
  stats: V4r0[];
  projects: V4a9[];
  users: V4a8[];
}) {
  const V2jp = "__none__";
  const V2jq = new Map(V2jo.map((p) => [p.id, p.adminIds]));
  const V2jr = new Map(
    users
      .filter((u) => u.role === "admin" || u.role === "super_admin")
      .map((u) => [u.id, `${u.displayName} (${u.username})`]),
  );

  const V2js = new Map<string, { name: string; projects: Set<string>; t: V4rf }>();
  let V2jt = false;
  for (const s of V2jn) {
    const V2ju = s.projectId ? (V2jq.get(s.projectId) ?? []) : [];
    if (V2ju.length > 1) V2jt = true;
    const V2jv = V2ju.length > 0 ? V2ju : [V2jp];
    for (const V2jw of V2jv) {
      let g = V2js.get(V2jw);
      if (!g) {
        g = {
          name:
            V2jw === V2jp
              ? "4oCUIGNoxrBhIGfDoW4gYWRtaW4g4oCU"
              : (V2jr.get(V2jw) ?? "QWRtaW4gxJHDoyB4b8Oh"),
          projects: new Set(),
          t: V4rg(),
        };
        V2js.set(V2jw, g);
      }
      if (s.projectId) g.projects.add(s.projectId);
      V4rh(g.t, s);
    }
  }
  const V2jx = [...V2js.values()].sort((a, b) => b.t.files - a.t.files);

  return (
    <div className="mb-5">
      <h3 className="mt-1 mx-0 mb-2">Tổng hợp theo admin</h3>
      {V2jt && (
        <p className="muted mt-0 mx-0 mb-2">
          Có dự án do nhiều admin cùng phụ trách — số liệu tính cho từng người, nên tổng các dòng
          lớn hơn tổng toàn hệ.
        </p>
      )}
      <div className="tbl-scroll">
        <table className="tbl">
          <thead>
            <tr>
              <th>Admin phụ trách</th>
              <th className="num">Dự án</th>
              <th className="num">Nhân viên</th>
              <th className="num">Trang quét</th>
              <th className="num">Hồ sơ</th>
              <th className="num">File gửi</th>
              <th className="num">DL gửi</th>
            </tr>
          </thead>
          <tbody>
            {V2jx.map((g) => (
              <tr key={g.name}>
                <td>
                  <b>{g.name}</b>
                </td>
                <td className="num">{g.projects.size}</td>
                <td className="num">{g.t.users}</td>
                <td className="num">{V4rd(g.t.pages)}</td>
                <td className="num">{g.t.hoSo}</td>
                <td className="num">
                  <b>{g.t.files}</b>
                </td>
                <td className="num">{V4re(g.t.bytesUp)}</td>
              </tr>
            ))}
          </tbody>
        </table>
      </div>
    </div>
  );
}

const V4rl: V3xs[] = ["super_admin", "admin", "user", "customer"];

function V4rm({
  mode: V2jy,
  isSuper: V2jz,
  projects: V2k0,
  user,
  onClose: V2k1,
  onDone: V2k2,
}: {
  mode: "create" | "edit";
  isSuper: boolean;
  projects: V4a9[];
  user?: V4a8;
  onClose: () => void;
  onDone: () => void | Promise<void>;
}) {
  const [V2k3, V2k4] = useState(user?.username ?? "");
  const [V2k5, V2k6] = useState(user?.displayName ?? "");
  const [V2k7, V2k8] = useState(user?.email ?? "");
  const [V2k9, V2ka] = useState("");

  const [V2kb, V2kc] = useState<V3xs>(user?.role ?? "user");

  const [V2kd, V2ke] = useState(user?.projectId ?? (!V2jz && V2k0.length === 1 ? V2k0[0].id : ""));
  const [V2kf, V2kg] = useState(false);
  const [V2kh, V2ki] = useState("");

  const V2kj: V3xs = V2jz ? V2kb : "user";

  const V2kk = V2jy === "create" && (V2kj === "user" || (V2jz && V2kj === "admin"));

  const V2kl = async () => {
    if (V2kf) return;
    V2kg(true);
    V2ki("");
    try {
      if (V2jy === "create") {
        const V2km = await V4ad({
          username: V2k3.trim(),
          password: V2k9,
          displayName: V2k5.trim(),
          email: V2k7.trim() || null,
          role: V2kj,
          projectId: V2kk ? V2kd || null : null,
        });

        if (V2jz && V2kj === "admin" && V2kd) {
          await V4an(V2kd, V2km.id);
        }
      } else if (user) {
        await V4ae({
          id: user.id,
          displayName: V2k5.trim(),
          email: V2k7.trim() || null,
        });
      }
      await V2k2();
    } catch (V2kn) {
      V2ki(V4r9(V2kn));
    } finally {
      V2kg(false);
    }
  };

  return (
    <div className="overlay" onClick={V2k1}>
      <div className="dialog" onClick={(V2ko) => V2ko.stopPropagation()}>
        <h3>
          {V2jy === "create" ? "VGjDqm0gbmfGsOG7nWkgZMO5bmc=" : `U+G7rWE6IA==${user?.username}`}
        </h3>

        <div className="field">
          <label>Tên đăng nhập</label>
          <input
            className="inp"
            value={V2k3}
            disabled={V2jy === "edit"}
            onChange={(V2kp) => V2k4(V2kp.target.value)}
          />
        </div>
        <div className="field">
          <label>Tên hiển thị</label>
          <input className="inp" value={V2k5} onChange={(V2kq) => V2k6(V2kq.target.value)} />
        </div>
        <div className="field">
          <label>Email (tuỳ chọn)</label>
          <input className="inp" value={V2k7} onChange={(V2kr) => V2k8(V2kr.target.value)} />
        </div>

        {V2jy === "create" && (
          <div className="field">
            <label>Mật khẩu (≥ 8 ký tự)</label>
            <input
              className="inp"
              type="password"
              value={V2k9}
              onChange={(V2ks) => V2ka(V2ks.target.value)}
            />
          </div>
        )}

        {V2jy === "create" && V2jz && (
          <div className="field">
            <label>Vai trò</label>
            <select
              className="role-sel"
              value={V2kb}
              onChange={(V2kt) => V2kc(V2kt.target.value as V3xs)}
            >
              {V4rl.map((r) => (
                <option key={r} value={r}>
                  {V3za(r)}
                </option>
              ))}
            </select>
          </div>
        )}

        {V2jy === "create" && V2kk && (
          <div className="field">
            <label>Dự án (tuỳ chọn)</label>
            {V2k0.length === 0 ? (
              <span className="muted">
                Chưa có dự án nào — tài khoản sẽ được tạo mà không gắn dự án.
              </span>
            ) : (
              <select
                className="role-sel"
                value={V2kd}
                onChange={(V2ku) => V2ke(V2ku.target.value)}
              >
                <option value="">— không gắn dự án —</option>
                {V2k0.map((p) => (
                  <option key={p.id} value={p.id}>
                    {p.code} · {p.name}
                  </option>
                ))}
              </select>
            )}
          </div>
        )}

        {V2kh && <div className="login-error">{V2kh}</div>}

        <div className="actions">
          <button className="btn" onClick={V2k1}>
            Huỷ
          </button>
          <button
            className="btn primary"
            onClick={() => void V2kl()}
            disabled={
              V2kf || !V2k5.trim() || (V2jy === "create" && (!V2k3.trim() || V2k9.length < 8))
            }
          >
            {V2kf
              ? "xJBhbmcgbMawdeKApg=="
              : V2jy === "create"
                ? "VOG6oW8gdMOgaSBraG/huqNu"
                : "TMawdQ=="}
          </button>
        </div>
      </div>
    </div>
  );
}

function V4rn({
  user,
  onClose: V2kv,
  onDone: V2kw,
}: {
  user: V4a8;
  onClose: () => void;
  onDone: () => void;
}) {
  const [V2kx, V2ky] = useState("");
  const [V2kz, V2l0] = useState("");
  const [V2l1, V2l2] = useState(false);
  const [V2l3, V2l4] = useState("");
  const [V2l5, V2l6] = useState(false);

  const V2l7 = async () => {
    if (V2l1) return;
    if (V2kx !== V2kz) {
      V2l4("SGFpIGzhuqduIG5o4bqtcCBt4bqtdCBraOG6qXUga2jDtG5nIGto4bubcC4=");
      return;
    }
    V2l2(true);
    V2l4("");
    try {
      await V4ag(user.id, V2kx);
      V2l6(true);
      setTimeout(V2kw, 700);
    } catch (V2l8) {
      V2l4(V4r9(V2l8));
    } finally {
      V2l2(false);
    }
  };

  return (
    <div className="overlay" onClick={V2kv}>
      <div className="dialog" onClick={(V2l9) => V2l9.stopPropagation()}>
        <h3>Đặt lại mật khẩu — {user.username}</h3>
        <p className="muted -mt-1.5 text-sm">
          Đổi mật khẩu sẽ đăng xuất mọi phiên đang hoạt động của tài khoản này.
        </p>
        <div className="field">
          <label>Mật khẩu mới (≥ 8 ký tự)</label>
          <input
            className="inp"
            type="password"
            value={V2kx}
            onChange={(V2la) => V2ky(V2la.target.value)}
          />
        </div>
        <div className="field">
          <label>Nhập lại</label>
          <input
            className="inp"
            type="password"
            value={V2kz}
            onChange={(V2lb) => V2l0(V2lb.target.value)}
          />
        </div>
        {V2l3 && <div className="login-error">{V2l3}</div>}
        {V2l5 && <div className="muted">Đã đổi mật khẩu</div>}
        <div className="actions">
          <button className="btn" onClick={V2kv}>
            Huỷ
          </button>
          <button
            className="btn primary"
            onClick={() => void V2l7()}
            disabled={V2l1 || V2kx.length < 8}
          >
            {V2l1 ? "xJBhbmcgbMawdeKApg==" : "xJDhurd0IGzhuqFp"}
          </button>
        </div>
      </div>
    </div>
  );
}

function V4ro({ user, onClose: V2lc }: { user: V4a8; onClose: () => void }) {
  const [data, V2ld] = useState<V4ah | null>(null);
  const [V2le, V2lf] = useState<Set<V3yx>>(new Set());
  const [V2lg, V2lh] = useState(true);
  const [V2li, V2lj] = useState(false);
  const [V2lk, V2ll] = useState("");
  const [V2lm, V2ln] = useState(false);

  const V2lo = useCallback(async () => {
    V2lh(true);
    V2ll("");
    try {
      const d = await V4ai(user.id);
      V2ld(d);
      V2lf(new Set(d.effective));
    } catch (V2lp) {
      V2ll(V4r9(V2lp));
    } finally {
      V2lh(false);
    }
  }, [user.id]);
  V41p(V2lo, { topics: ["users"] });

  const V2lq = useMemo(() => new Set<V3yx>(data?.roleDefaults ?? []), [data]);
  const V2lr = useMemo(() => new Set<V3yx>(data?.grantable ?? []), [data]);

  const V2ls = (p: V3yx) => {
    if (!V2lr.has(p) && !V2lq.has(p)) return;
    V2lf((V2lt) => {
      const V2lu = new Set(V2lt);
      if (V2lu.has(p)) V2lu.delete(p);
      else V2lu.add(p);
      return V2lu;
    });
  };

  const V2lv = useMemo(() => [...V2le].filter((p) => !V2lq.has(p)), [V2le, V2lq]);
  const V2lw = useMemo(() => [...V2lq].filter((p) => !V2le.has(p)), [V2le, V2lq]);

  const V2lx = useMemo(() => {
    if (!data) return false;
    const V2ly = new Set(data.effective);
    if (V2ly.size !== V2le.size) return true;
    for (const p of V2le) if (!V2ly.has(p)) return true;
    return false;
  }, [data, V2le]);

  const V2lz = async () => {
    if (V2li || !data) return;
    V2lj(true);
    V2ll("");
    try {
      const d = await V4aj(user.id, V2lv, V2lw);
      V2ld(d);
      V2lf(new Set(d.effective));
      V2ln(true);
      setTimeout(() => V2ln(false), 3000);
    } catch (V2m0) {
      V2ll(V4r9(V2m0));
    } finally {
      V2lj(false);
    }
  };

  return (
    <div className="overlay" onClick={V2lc}>
      <div className="dialog max-w-150" onClick={(V2m1) => V2m1.stopPropagation()}>
        <h3>Phân quyền — {user.username}</h3>
        <p className="muted -mt-1.5 text-sm">
          Vai trò <b>{V3za(user.role)}</b> cấp sẵn bộ quyền mặc định. Tích để CẤP thêm, bỏ tích để
          GỠ từng chức năng. Ô mờ là quyền bạn không được phép cấp. Quyền mới có hiệu lực khi người
          đó <b>đăng nhập lại</b> (hoặc bấm “Kiểm tra kết nối” trong Cài đặt).
        </p>

        {V2lg && <p className="muted">Đang tải quyền…</p>}

        {!V2lg && data && (
          <div className="max-h-[52vh] overflow-y-auto my-2 mx-0 pr-1">
            {V3z0.map((g) => (
              <div key={g.key} className="mb-2.5">
                <div className="font-semibold text-sm my-1 mx-0">{g.label}</div>
                {g.perms.map(({ perm: V2m2, label: V2m3 }) => {
                  const V2m4 = V2le.has(V2m2);
                  const V2m5 = V2lq.has(V2m2);
                  const V2m6 = V2lr.has(V2m2) || V2m5;
                  return (
                    <label
                      key={V2m2}
                      className={`flex items-center gap-2 py-0.5${
                        V2m6 ? " cursor-pointer" : " cursor-not-allowed opacity-50"
                      }`}
                    >
                      <input
                        type="checkbox"
                        checked={V2m4}
                        disabled={!V2m6}
                        onChange={() => V2ls(V2m2)}
                      />
                      <span className="text-sm">{V2m3}</span>
                      <small className="muted text-xs">{V2m2}</small>
                      {V2m4 && !V2m5 && (
                        <span className="badge ok ml-auto">
                          <Icon name="them" />
                          cấp thêm
                        </span>
                      )}
                      {!V2m4 && V2m5 && (
                        <span className="badge err ml-auto">
                          <Icon name="bo" />
                          đã gỡ
                        </span>
                      )}
                    </label>
                  );
                })}
              </div>
            ))}
          </div>
        )}

        {V2lk && <div className="login-error">{V2lk}</div>}
        {V2lm && (
          <div className="badge ok inline-block">
            <Icon name="duyet" />
            Đã lưu quyền — áp dụng khi người dùng đăng nhập lại
          </div>
        )}
        {!V2lg && data && (
          <div className="muted text-sm">
            {V2lv.length} quyền cấp thêm · {V2lw.length} quyền gỡ bỏ so với mặc định vai trò
          </div>
        )}

        <div className="actions">
          <button
            className="btn"
            onClick={() => data && V2lf(new Set(data.roleDefaults))}
            disabled={!data || V2li}
          >
            Về mặc định vai trò
          </button>
          <button className="btn" onClick={V2lc}>
            Đóng
          </button>
          <button
            className="btn primary"
            onClick={() => void V2lz()}
            disabled={V2li || V2lg || !V2lx}
          >
            {V2li ? "xJBhbmcgbMawdeKApg==" : V2lm ? "xJDDoyBsxrB1" : "TMawdSBxdXnhu4Fu"}
          </button>
        </div>
      </div>
    </div>
  );
}

function V4rp({
  onClose: V2m7,
  onDone: V2m8,
}: {
  onClose: () => void;
  onDone: () => void | Promise<void>;
}) {
  const [V2m9, V2ma] = useState("");
  const [name, V2mb] = useState("");
  const [V2mc, V2md] = useState(false);
  const [V2me, V2mf] = useState("");

  const V2mg = async () => {
    if (V2mc) return;
    V2md(true);
    V2mf("");
    try {
      await V4al(V2m9.trim(), name.trim());
      await V2m8();
    } catch (V2mh) {
      V2mf(V4r9(V2mh));
    } finally {
      V2md(false);
    }
  };

  return (
    <div className="overlay" onClick={V2m7}>
      <div className="dialog" onClick={(V2mi) => V2mi.stopPropagation()}>
        <h3>Tạo dự án</h3>
        <div className="field">
          <label>Mã dự án</label>
          <input
            className="inp"
            value={V2m9}
            placeholder="VD: STC"
            onChange={(V2mj) => V2ma(V2mj.target.value)}
          />
        </div>
        <div className="field">
          <label>Tên dự án</label>
          <input className="inp" value={name} onChange={(V2mk) => V2mb(V2mk.target.value)} />
        </div>
        {V2me && <div className="login-error">{V2me}</div>}
        <div className="actions">
          <button className="btn" onClick={V2m7}>
            Huỷ
          </button>
          <button
            className="btn primary"
            onClick={() => void V2mg()}
            disabled={V2mc || !V2m9.trim() || !name.trim()}
          >
            {V2mc ? "xJBhbmcgdOG6oW/igKY=" : "VOG6oW8gZOG7sSDDoW4="}
          </button>
        </div>
      </div>
    </div>
  );
}
