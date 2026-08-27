// Copyright (c) 2026 DigiText. All rights reserved.

import { invoke } from "@/api/ipc";
import type { V3yq, V3ym, V3yp, V3yk, V3yl, V3yo } from "@/domain/types";

export interface V48i {
  phongBans: V3yl[];
  mangs: V3ym[];
  thbqs: V3yo[];
  ngonNgus: V3yp[];
  loaiVanBans: V3yq[];
}

export function V48j(): Promise<V48i> {
  return invoke<V48i>("catalog_list");
}

export function V48k(Vjx: V3yl): Promise<void> {
  return invoke<void>("phong_ban_upsert", { args: { phongBan: Vjx } });
}

export function V48l(Vjy: string): Promise<void> {
  return invoke<void>("phong_ban_delete", { args: { maDv: Vjy } });
}

export function V48m(Vjz: V3ym): Promise<void> {
  return invoke<void>("mang_upsert", { args: { mang: Vjz } });
}

export function V48n(Vk0: number): Promise<void> {
  return invoke<void>("mang_delete", { args: { mangCvid: Vk0 } });
}

export function V48o(Vk1: V3yo): Promise<void> {
  return invoke<void>("thbq_upsert", { args: { thbq: Vk1 } });
}

export function V48p(Vk2: number): Promise<void> {
  return invoke<void>("thbq_delete", { args: { thbqid: Vk2 } });
}

export function V48q(Vk3: V3yk): Promise<void> {
  return invoke<void>("phong_upsert", { args: { phong: Vk3 } });
}

export function V48r(Vk4: string): Promise<void> {
  return invoke<void>("phong_delete", { args: { maDuAn: Vk4 } });
}

export function V48s(Vk5: V3yp): Promise<void> {
  return invoke<void>("ngon_ngu_upsert", { args: { ngonNgu: Vk5 } });
}

export function V48t(Vk6: string): Promise<void> {
  return invoke<void>("ngon_ngu_delete", { args: { maNgonNgu: Vk6 } });
}

export function V48u(Vk7: V3yq): Promise<void> {
  return invoke<void>("loai_van_ban_upsert", { args: { loaiVanBan: Vk7 } });
}

export function V48v(Vk8: string): Promise<void> {
  return invoke<void>("loai_van_ban_delete", { args: { maLoai: Vk8 } });
}
