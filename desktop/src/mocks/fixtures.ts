// Copyright (c) 2026 DigiText. All rights reserved.

import type { V3y0, V3xz, V3y1, V3yi } from "@/domain/types";
import type { V41v } from "@/components/Icon";

export const V41x: Record<string, V3y1> = {
  root: { id: "root", type: "root", name: "Kho tài liệu", kids: [] },
};

export const V41y: Record<V3xz, { tone: V3y0; icon: V41v; label: string }> = {
  signed: { tone: "ok", icon: "kySo", label: "Đã ký" },
  uploading: { tone: "info", icon: "taiLen", label: "Đang tải" },
  editing: { tone: "info", icon: "anh", label: "Đang xử lý" },
  flagged: { tone: "warn", icon: "canhBao", label: "Có cờ" },
  scanned: { tone: "gray", icon: "taiLieu", label: "Đã quét" },
};

export const V41z = [
  "Quản trị",
  "Biên tập (quét + xử lý ảnh)",
  "Ký số",
  "Chỉ xem",
] as const;

export const V420: V3yi[] = [];
