// Copyright (c) 2026 DigiText. All rights reserved.
import { useState } from "react";
import { V460 } from "@/state";
import { V41z } from "@/mocks/fixtures";
import type { V3yi } from "@/domain/types";
import { Icon } from "@/components/Icon";

export default function V4wv() {
  const { dialogTitle: V37b, closeDialog: V37c, members: V37d, setMembers: V37e } = V460();

  const [V37f, V37g] = useState<V3yi[]>(V37d);
  const [V37h, V37i] = useState("");
  const [V37j, V37k] = useState<string>(V41z[1]);

  const V37l = (i: number, V37m: string) =>
    V37g((d) => d.map((m, V37n) => (V37n === i ? { ...m, role: V37m } : m)));
  const V37o = (i: number) => V37g((d) => d.filter((_V37p, V37q) => V37q !== i));
  const V37r = () => {
    const name = V37h.trim();
    if (!name) return;
    V37g((d) => [
      ...d,
      {
        name,
        email: name.toLowerCase().replace(/\s+/g, ".") + "@tn.gov.vn",
        role: V37j,
        av: name[0].toUpperCase(),
        c: d.length % 5,
      },
    ]);
    V37i("");
  };
  const V37s = () => {
    V37e(V37f);
    V37c();
  };

  return (
    <div className="overlay" onClick={V37c}>
      <div className="dialog wide" onClick={(V37t) => V37t.stopPropagation()}>
        <h3>
          Quản lý thành viên — <span>{V37b || "ROG7sSDDoW4="}</span>
        </h3>
        <p className="muted -mt-1.5 mx-0 mb-3.5 text-sm">
          Phân quyền theo dự án. Chủ sở hữu luôn có toàn quyền.
        </p>
        <div className="tbl-scroll">
          <table className="tbl">
            <thead>
              <tr>
                <th>Thành viên</th>
                <th>Vai trò trong dự án</th>
                <th />
              </tr>
            </thead>
            <tbody>
              {V37f.map((m, i) => (
                <tr key={i}>
                  <td>
                    <div className="mem-cell">
                      <span className={`a c${m.c}`}>{m.av}</span>
                      <div>
                        {m.name}
                        {m.owner && <span className="owner-tag"> (chủ sở hữu)</span>}
                        <small>{m.email}</small>
                      </div>
                    </div>
                  </td>
                  <td>
                    {m.owner ? (
                      <span className="muted">Toàn quyền</span>
                    ) : (
                      <select
                        className="role-sel"
                        value={m.role}
                        onChange={(V37u) => V37l(i, V37u.target.value)}
                      >
                        {V41z.map((r) => (
                          <option key={r}>{r}</option>
                        ))}
                      </select>
                    )}
                  </td>
                  <td>
                    {!m.owner && (
                      <span
                        className="rm-mem"
                        title="R+G7oSBraOG7j2kgZOG7sSDDoW4="
                        onClick={() => V37o(i)}
                      >
                        <Icon name="dong" size={13} />
                      </span>
                    )}
                  </td>
                </tr>
              ))}
            </tbody>
          </table>
        </div>
        <div className="add-member">
          <input
            placeholder="KyBUaMOqbSB0aGVvIHTDqm4gLyBlbWFpbOKApg=="
            value={V37h}
            onChange={(V37v) => V37i(V37v.target.value)}
            onKeyDown={(V37w) => V37w.key === "Enter" && V37r()}
          />
          <select className="role-sel" value={V37j} onChange={(V37x) => V37k(V37x.target.value)}>
            {V41z.map((r) => (
              <option key={r}>{r}</option>
            ))}
          </select>
          <button className="btn" onClick={V37r}>
            Thêm
          </button>
        </div>
        <div className="actions">
          <button className="btn" onClick={V37c}>
            Đóng
          </button>
          <button className="btn primary" onClick={V37s}>
            Lưu thay đổi
          </button>
        </div>
      </div>
    </div>
  );
}
