// Copyright (c) 2026 DigiText. All rights reserved.

export type V4ta = "kiem_tra" | "nhat_ky_sao_luu" | "sao_luu" | "su_co" | "thu_nop_truc_tiep";

export interface V4tb {
  key: string;
  label: string;
  kind: "text" | "number";
  optional?: boolean;
  multiline?: boolean;
}

export interface V4tc {
  key: string;
  label: string;
  optional?: boolean;
}

export interface V4td {
  key: string;
  label: string;

  simple?: boolean;
  columns: V4tc[];
  minRows: number;
}

export interface V4te {
  kind: V4ta;
  label: string;

  legal: string;
  fields: V4tb[];
  lists: V4td[];
}

const t = (V2u0: string, V2u1: string, V2u2 = false): V4tb => ({
  key: V2u0,
  label: V2u1,
  kind: "text",
  optional: V2u2,
});
const n = (V2u3: string, V2u4: string): V4tb => ({ key: V2u3, label: V2u4, kind: "number" });

export const V4tf: V4te[] = [
  {
    kind: "kiem_tra",
    label: "QmnDqm4gYuG6o24ga2nhu4NtIHRyYSB0w6BpIGxp4buHdSBsxrB1IHRy4buvIHPhu5E=",
    legal: "TeG7pWMgMSBQaOG7pSBs4bulYyBWSUkg4oCUIMSQaeG7gXUgMjk=",
    fields: [
      t("donVi", "xJDGoW4gduG7iyBraeG7g20gdHJh"),
      t("diaDiem", "xJDhu4thIMSRaeG7g20="),
      t("ngay", "TmfDoHkgbOG6rXAgKEREL01NL1lZWVkp"),
      t("phamVi", "UGjhuqFtIHZpIGtp4buDbSB0cmEgKHBow7RuZyAvIGNo4buNbiBt4bqrdSk="),
      n("tongSoTep", "VOG7lW5nIHPhu5EgdOG7h3Aga2nhu4NtIHRyYQ=="),
      n("soDat", "U+G7kSB04buHcCDEkeG6oXQ="),
      n("soLoi", "U+G7kSB04buHcCBs4buXaQ=="),
      t("phuongAnKhacPhuc", "UGjGsMahbmcgw6FuIGto4bqvYyBwaOG7pWMgKMSQaeG7gXUgMjkuNC5nKQ==", true),
      t("ketLuan", "S+G6v3QgbHXhuq1u", true),
      t("nguoiLap", "TmfGsOG7nWkgbOG6rXAgYmnDqm4gYuG6o24="),
      t("thuTruong", "VGjhu6cgdHLGsOG7n25nIMSRxqFuIHbhu4s="),
    ],
    lists: [
      {
        key: "thanhPhan",
        label: "VGjDoG5oIHBo4bqnbiB0aGFtIGdpYQ==",
        simple: true,
        columns: [],
        minRows: 1,
      },
      {
        key: "tepLoi",
        label: "RGFuaCBt4bulYyB04buHcCBs4buXaSAoxJBp4buBdSAyOS40LsSRKQ==",
        columns: [
          { key: "tenTep", label: "VMOqbiB04buHcA==" },
          { key: "diaChi", label: "xJDhu4thIGNo4buJIGzGsHUgdHLhu68=" },
          { key: "moTaLoi", label: "TcO0IHThuqMgbOG7l2k=", optional: true },
        ],
        minRows: 0,
      },
    ],
  },
  {
    kind: "nhat_ky_sao_luu",
    label: "Tmjhuq10IGvDvSBzYW8gbMawdQ==",
    legal: "TeG7pWMgMiBQaOG7pSBs4bulYyBWSUkg4oCUIMSQaeG7gXUgMzA=",
    fields: [
      t("donVi", "xJDGoW4gduG7iw=="),
      t("ky", "S+G7syBnaGkgbmjhuq10IGvDvQ=="),
      t("nguoiLap", "TmfGsOG7nWkgbOG6rXA="),
    ],
    lists: [
      {
        key: "entries",
        label: "Q8OhYyBs4bqnbiBzYW8gbMawdQ==",
        columns: [
          { key: "thoiGian", label: "VGjhu51pIGdpYW4=" },
          { key: "loai", label: "TG/huqFpIChnaWEgdMSDbmcvxJHhuqd5IMSR4bunKQ==" },
          { key: "nguonDuLieu", label: "Tmd14buTbiBk4buvIGxp4buHdQ==" },
          { key: "phuongTien", label: "UGjGsMahbmcgdGnhu4du" },
          { key: "ketQua", label: "S+G6v3QgcXXhuqM=" },
          { key: "nguoiThucHien", label: "TmfGsOG7nWkgdGjhu7FjIGhp4buHbg==" },
        ],
        minRows: 1,
      },
    ],
  },
  {
    kind: "sao_luu",
    label: "QmnDqm4gYuG6o24gc2FvIGzGsHU=",
    legal: "TeG7pWMgMyBQaOG7pSBs4bulYyBWSUkg4oCUIMSQaeG7gXUgMzA=",
    fields: [
      t("donVi", "xJDGoW4gduG7iw=="),
      t("diaDiem", "xJDhu4thIMSRaeG7g20="),
      t("ngay", "TmfDoHkgbOG6rXAgKEREL01NL1lZWVkp"),
      t("loaiSaoLuu", "TG/huqFpIHNhbyBsxrB1ICjEkGnhu4F1IDMwLjEp"),
      t("nguonDuLieu", "Tmd14buTbiBk4buvIGxp4buHdSAoQ1NETCArIG3DoyBuZ3Xhu5NuIOKAlCAzMC4zLmMp"),
      t("phuongTien", "UGjGsMahbmcgdGnhu4duIGzGsHUgdHLhu68="),
      t("moTaPhuongTien", "TcO0IHThuqMgcGjGsMahbmcgdGnhu4du", true),
      t("ketQuaKiemTra", "S+G6v3QgcXXhuqMga2nhu4NtIHRyYSBzYXUgc2FvIGzGsHUgKDMwLjMuZCk="),
      t("noiBaoQuan", "TsahaSBi4bqjbyBxdeG6o24gcGjGsMahbmcgdGnhu4duICgzMC4zLsSRKQ=="),
      t("nguoiThucHien", "TmfGsOG7nWkgdGjhu7FjIGhp4buHbg=="),
      t("nguoiKiemTra", "TmfGsOG7nWkga2nhu4NtIHRyYQ=="),
    ],
    lists: [],
  },
  {
    kind: "su_co",
    label: "QmnDqm4gYuG6o24geOG7rSBsw70gc+G7sSBj4buRIHbDoCBwaOG7pWMgaOG7k2k=",
    legal: "TeG7pWMgNCBQaOG7pSBs4bulYyBWSUkg4oCUIMSQaeG7gXUgMzE=",
    fields: [
      t("donVi", "xJDGoW4gduG7iw=="),
      t("diaDiem", "xJDhu4thIMSRaeG7g20="),
      t("ngay", "TmfDoHkgbOG6rXAgKEREL01NL1lZWVkp"),
      t("thoiDiemSuCo", "VGjhu51pIMSRaeG7g20geOG6o3kgcmEgc+G7sSBj4buR"),
      t("moTaSuCo", "TcO0IHThuqMgc+G7sSBj4buR"),
      t("nguyenNhan", "Tmd1ecOqbiBuaMOibiAocGjhuqduIGPhu6luZyAvIHBo4bqnbiBt4buBbSDigJQgMzEuMSk="),
      t(
        "nguonPhucHoi",
        "Tmd14buTbiBwaOG7pWMgaOG7k2kgKGLhuqNuIHNhbyBsxrB1IGfhuqduIG5o4bqldCDigJQgMzEuNCk=",
      ),
      t("phuongPhapPhucHoi", "UGjGsMahbmcgcGjDoXAgcGjhu6VjIGjhu5Np"),
      t("ketQuaKiemTra", "S+G6v3QgcXXhuqMga2nhu4NtIHRyYSBzYXUgcGjhu6VjIGjhu5NpICgzMS41KQ=="),
      t("nguoiXuLy", "TmfGsOG7nWkgeOG7rSBsw70gc+G7sSBj4buR"),
      t("phuTrach", "UGjhu6UgdHLDoWNoIMSRxqFuIHbhu4s="),
    ],
    lists: [
      {
        key: "bienPhap",
        label: "Qmnhu4duIHBow6FwIGPDoWNoIGx5IC8ga2jhuq9jIHBo4bulYyAoMzEuMiwgMzEuMyk=",
        simple: true,
        columns: [],
        minRows: 1,
      },
    ],
  },
  {
    kind: "thu_nop_truc_tiep",
    label: "QmnDqm4gYuG6o24gdGh1IG7hu5lwIChnaWFvIG5o4bqtbiB0cuG7sWMgdGnhur9wKQ==",
    legal: "xJBp4buBdSAyMi4yLsSR",
    fields: [
      t("maDangKy", "TcOjIMSRxINuZyBrw70gecOqdSBj4bqndSAobuG6v3UgY8OzKQ==", true),
      t("maCoQuan", "TcOjIGPGoSBxdWFuLCB04buVIGNo4bupYyBu4buZcA=="),
      t("tenCoQuan", "VMOqbiBjxqEgcXVhbiwgdOG7lSBjaOG7qWMgbuG7mXA="),
      n("soLanNop", "U+G7kSBs4bqnbiBu4buZcA=="),
      n(
        "tongDangKy",
        "VOG7lW5nIHPhu5EgaOG7kyBzxqEsIHTDoGkgbGnhu4d1IG7hu5lwIHRoZW8gxJHEg25nIGvDvQ==",
      ),
      n("tongGiaoNhan", "VOG7lW5nIHPhu5EgaOG7kyBzxqEsIHTDoGkgbGnhu4d1IGdpYW8gbmjhuq1u"),
      t("tongDungLuong", "VOG7lW5nIHPhu5EgZHVuZyBsxrDhu6NuZyAodmQgMSwyIEdCKQ=="),
      t("thoiGianGiaoNhan", "VGjhu51pIGdpYW4gZ2lhbyBuaOG6rW4gKEREL01NL1lZWVkp"),
      t("nguoiGiao", "TmfGsOG7nWkgZ2lhbw=="),
      t("nguoiNhan", "TmfGsOG7nWkgbmjhuq1u"),
      t("ghiChu", "R2hpIGNow7oga2jDoWM=", true),
    ],
    lists: [
      {
        key: "thietBi",
        label:
          "VGhp4bq/dCBi4buLIGzGsHUgdHLhu68gYsOgbiBnaWFvICh0w6xuaCB0cuG6oW5nLCBtw7QgdOG6oyDigJQgMjIuMi7EkSk=",
        columns: [
          { key: "ten", label: "VGhp4bq/dCBi4buL" },
          { key: "tinhTrang", label: "VMOsbmggdHLhuqFuZw==" },
          { key: "moTa", label: "TcO0IHThuqMgY2hpIHRp4bq/dA==", optional: true },
        ],
        minRows: 1,
      },
    ],
  },
];

export function V4tg(V2u5: V4ta): V4te {
  const t = V4tf.find((x) => x.kind === V2u5);
  if (!t) throw new Error(`a2jDtG5nIGPDsyBt4bqrdTog${V2u5}`);
  return t;
}

export interface V4th {
  payload: Record<string, unknown> | null;
  errors: string[];
}

export function V4ti(
  template: V4te,
  V2u6: Record<string, string>,
  V2u7: Record<string, Record<string, string>[]>,
): V4th {
  const V2u8: string[] = [];
  const V2u9: Record<string, unknown> = {};

  for (const f of template.fields) {
    const V2ua = (V2u6[f.key] ?? "").trim();
    if (!V2ua) {
      if (!f.optional) V2u8.push(`VGhp4bq/dSAi${f.label}".`);
      continue;
    }
    if (f.kind === "number") {
      const V2ub = Number(V2ua);
      if (!Number.isFinite(V2ub) || V2ub < 0) {
        V2u8.push(`"${f.label}IiBwaOG6o2kgbMOgIHPhu5Ega2jDtG5nIMOibS4=`);
        continue;
      }
      V2u9[f.key] = V2ub;
    } else {
      V2u9[f.key] = V2ua;
    }
  }

  for (const l of template.lists) {
    const V2uc = (V2u7[l.key] ?? []).filter((r) => Object.values(r).some((v) => (v ?? "").trim()));
    if (V2uc.length < l.minRows) {
      V2u8.push(`"${l.label}IiBj4bqnbiB04buRaSB0aGnhu4N1IA==${l.minRows}IGTDsm5nLg==`);
      continue;
    }
    if (l.simple) {
      V2u9[l.key] = V2uc.map((r) => (r.value ?? "").trim()).filter(Boolean);
      if ((V2u9[l.key] as string[]).length < l.minRows) {
        V2u8.push(`"${l.label}IiBj4bqnbiB04buRaSB0aGnhu4N1IA==${l.minRows}IGTDsm5nLg==`);
      }
      continue;
    }
    const V2ud: Record<string, unknown>[] = [];
    for (const [i, row] of V2uc.entries()) {
      const V2ue: Record<string, unknown> = {};
      for (const c of l.columns) {
        const v = (row[c.key] ?? "").trim();
        if (!v) {
          if (!c.optional) V2u8.push(`"${l.label}IiBkw7JuZyA=${i + 1}OiB0aGnhur91ICI=${c.label}".`);
          continue;
        }
        V2ue[c.key] = v;
      }
      V2ud.push(V2ue);
    }
    V2u9[l.key] = V2ud;
  }

  return V2u8.length > 0 ? { payload: null, errors: V2u8 } : { payload: V2u9, errors: [] };
}
