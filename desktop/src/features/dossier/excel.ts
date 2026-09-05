// Copyright (c) 2026 DigiText. All rights reserved.

import type { WorkBook } from "xlsx";
import { V4bf, V4b7 } from "@/domain/catalogImport";
import { isTauri } from "@/api/ipc";
import { V4dz } from "@/api/exportFile";
import type { V4bm, V4cf } from "@/api/dossier";
import { V4dg, type V4d8, V4db } from "./logic";

async function V4e1(Vzq: WorkBook, Vzr: string): Promise<void> {
  const Vzs = await import("xlsx");
  if (isTauri()) {
    const Vzt = Vzs.write(Vzq, { type: "array", bookType: "xlsx" }) as ArrayBuffer;
    await V4dz(new Uint8Array(Vzt), Vzr, "xlsx");
  } else {
    Vzs.writeFile(Vzq, Vzr);
  }
}

export const V4e2 = [
  "Mảng",
  "ID Hồ Sơ",
  "Mã Hồ Sơ",
  "Số Tạm",
  "Tiêu Đề",
  "THBQ",
  "Số Tờ",
  "Ngày BĐ",
  "Ngày KT",
  "T.Thừa",
  "T.Thái",
  "Tài Liệu",
  "Chức Năng",
];

export async function V4e3(
  Vzu: Record<string, unknown>[],
  Vzv: string,
  Vzw: string,
): Promise<void> {
  const Vzx = await import("xlsx");
  const Vzy = Vzx.utils.json_to_sheet(Vzu, { skipHeader: Vzu.length === 0 });
  const Vzz = Vzx.utils.book_new();
  Vzx.utils.book_append_sheet(Vzz, Vzy, Vzv);
  await V4e1(Vzz, Vzw);
}

export interface V4e4 {
  name: string;
  aoa: (string | number)[][];

  merges?: string[];
}

export async function V4e5(V100: V4e4[], V101: string): Promise<void> {
  const V102 = await import("xlsx");
  const V103 = V102.utils.book_new();
  const V104 = V100.length > 0 ? V100 : [{ name: "Sheet1", aoa: [] as (string | number)[][] }];
  for (const s of V104) {
    const V105 = V102.utils.aoa_to_sheet(s.aoa);
    if (s.merges?.length) {
      V105["!merges"] = s.merges.map((r) => V102.utils.decode_range(r));
    }

    const name = (s.name || "Sheet").slice(0, 31);
    V102.utils.book_append_sheet(V103, V105, name);
  }
  await V4e1(V103, V101);
}

export async function V4e6(): Promise<void> {
  const V106: Record<string, unknown> = {
    Mảng: "",
    "ID Hồ Sơ": "",
    "Mã Hồ Sơ": "",
    "Số Tạm": "",
    "Tiêu Đề": "",
    THBQ: "",
    "Số Tờ": "",
    "Ngày BĐ": "",
    "Ngày KT": "",
    "T.Thừa": "",
    "T.Thái": "",
    "Tài Liệu": "",
    "Chức Năng": "",
  };
  await V4e3([V106], "MauHoSo", "mau-lap-ho-so.xlsx");
}

export async function V4e7(): Promise<void> {
  const V107: Record<string, unknown> = {};
  for (const h of V4db) V107[h] = "";
  await V4e3([V107], "MauVanBan", "mau-nhap-van-ban.xlsx");
}

export async function V4e8(
  file: File,
  V108: V4bm[] = [],
): Promise<{ items: V4cf[]; skipped: number; issues: V4d8[] }> {
  const V109 = await V4b7(file);
  const name = V4bf(V109);
  const V10a = V109.sheets.find((s) => s.name === name) ?? V109.sheets[0];
  if (!V10a) return { items: [], skipped: 0, issues: [] };
  return V4dg(V10a.rows, V108);
}
