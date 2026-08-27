// Copyright (c) 2026 DigiText. All rights reserved.

import { useCallback, useState } from "react";
import { V41p } from "@/live/useLive";
import { V460 } from "@/state";
import { isTauri } from "@/api/ipc";
import { V40w, type V40v } from "@/api/archive";
import { V4ny, V4o0, V4nz, V4nx } from "@/api/report";
import V4o9 from "./ReportView";
import { V4o4, V4o5, V4o6, V4o3 } from "./reportLogic";

function V4oa() {
  return (
    <V4o9
      title="VOG7lW5nIGvhur90IHPhu5EgaG/DoQ=="
      subtitle="U+G7kSBo4buTIHPGoS9QREYgxJHDoyBz4buRIGhvw6EgdsOgIHPhu5EgdHJhbmcgdGhlbyBraOG7lSBnaeG6pXkgKHF1eSDEkeG7lWkgQTQpIOKAlCBnb20gdGhlbyBwaMO0bmcu"
      columns={V4o3}
      fetchRows={V4nx}
      sheetName="TongKetSoHoa"
      fileBase="tong-ket-so-hoa"
    />
  );
}

function V4ob() {
  return (
    <V4o9
      title="VOG7lW5nIGvhur90IHRoZW8gbmjDom4gdmnDqm4="
      subtitle="S2jhu5FpIGzGsOG7o25nIGPDtG5nIHZp4buHYyBxdXkgduG7gSBuZ8aw4budaSDEkcSDbmcgbmjhuq1wOiBo4buTIHPGoSwgdsSDbiBi4bqjbiwgdHJhbmcgcXXDqXQsIHRoYW8gdMOhYy4="
      columns={V4o5}
      fetchRows={V4o0}
      sheetName="TongKetNhanVien"
      fileBase="tong-ket-nhan-vien"
    />
  );
}

function V4oc() {
  const [V22d, V22e] = useState<V40v[]>([]);
  const [maDa, V22f] = useState<string>("");
  const V22g = useCallback(() => {
    V40w()
      .then(V22e)
      .catch(() => {});
  }, []);
  V41p(V22g, { topics: ["catalog", "hoso"], enabled: isTauri() });
  return (
    <V4o9
      title="VOG7lW5nIGvhur90IGNo4buJbmggbMO9"
      subtitle="SOG7mXAgdsOgIGjhu5Mgc8ahIHbEqW5oIHZp4buFbiAvIGPDsyB0aOG7nWkgaOG6oW4sIGLDs+KAk3Thuq1wIGjhu5Mgc8ahIGxv4bqhaSDigJQgZ29tIHRoZW8gcGjDsm5nIGJhbi4="
      columns={V4o4}
      fetchRows={() => V4ny(maDa || undefined)}
      sheetName="TongKetChinhLy"
      fileBase="tong-ket-chinh-ly"
      filters={
        <select className="inp max-w-65" value={maDa} onChange={(V22h) => V22f(V22h.target.value)}>
          <option value="">— Tất cả phông —</option>
          {V22d.map((p) => (
            <option key={p.maDuAn} value={p.maDuAn}>
              {p.tenDuAn}
            </option>
          ))}
        </select>
      }
    />
  );
}

function V4od() {
  const [V22i, V22j] = useState<string>("");
  const [V22k, V22l] = useState<string>("");
  return (
    <V4o9
      title="VOG7lW5nIGvhur90IHBo4bqnbiBt4buBbQ=="
      subtitle="SOG7mXAvaOG7kyBzxqEgdsSpbmggdmnhu4VuICYgY8OzIHRo4budaSBo4bqhbiwgYsOz4oCTdOG6rXAgbG/huqFpIOKAlCBnb20gdGhlbyBwaMO0bmcsIGzhu41jIHRoZW8gbmfDoHkga2jhu59pIHThuqFvLg=="
      columns={V4o6}
      fetchRows={() => V4nz(V22i || undefined, V22k || undefined)}
      sheetName="TongKetPhanMem"
      fileBase="tong-ket-phan-mem"
      filters={
        <>
          <label className="muted text-sm">
            Từ ngày
            <input
              className="inp ml-1.5"
              type="date"
              value={V22i}
              onChange={(V22m) => V22j(V22m.target.value)}
            />
          </label>
          <label className="muted text-sm ml-2.5">
            Đến ngày
            <input
              className="inp ml-1.5"
              type="date"
              value={V22k}
              onChange={(V22n) => V22l(V22n.target.value)}
            />
          </label>
        </>
      }
    />
  );
}

export default function V4oe() {
  const { reportTab: V22o } = V460();
  if (V22o === "chinhly") return <V4oc />;
  if (V22o === "nhanvien") return <V4ob />;
  if (V22o === "phanmem") return <V4od />;
  return <V4oa />;
}
