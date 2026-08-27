// Copyright (c) 2026 DigiText. All rights reserved.

import type { V3yx } from "@/domain/permissions";
import type { V4s1, V4s5 } from "@/api/submissions";

export const V4t4: Record<string, string> = {
  registered: "Đã đăng ký (chờ xác nhận)",
  confirmed: "Đã xác nhận — đang nộp gói",
  refused: "Từ chối thu",
  awaiting_reception: "Chờ tiếp nhận",
  received: "Đã tiếp nhận — đang xử lý",
  approved: "Đã phê duyệt (SIP→AIP)",
  completed: "Hoàn thành",
};

export interface V4t5 {
  action: V4s5 | "approve";
  label: string;

  perm: V3yx;

  requiresNote?: boolean;
}

export function V4t6(status: string, V2rf: V3yx[]): V4t5[] {
  const V2rg: Record<string, V4t5[]> = {
    registered: [
      {
        action: "confirm",
        label: "Đồng ý thu (Điều 21.2)",
        perm: "submission.review",
      },
      {
        action: "refuse",
        label: "Từ chối thu (Điều 21.3)",
        perm: "submission.review",
        requiresNote: true,
      },
    ],
    confirmed: [
      {
        action: "reconcile",
        label: "Đối chiếu Mục lục (Điều 22.3.d)",
        perm: "submission.view",
      },
    ],
    awaiting_reception: [
      {
        action: "receive",
        label: "Tiếp nhận (Điều 23)",
        perm: "submission.review",
      },
      { action: "reconcile", label: "Đối chiếu lại", perm: "submission.view" },
    ],
    received: [
      {
        action: "approve",
        label: "Phê duyệt + chuyển SIP→AIP (Điều 25)",
        perm: "submission.approve",
      },
    ],
    approved: [
      {
        action: "complete",
        label: "Hoàn thành thu nộp (Điều 25.5)",
        perm: "submission.approve",
      },
    ],
  };
  return (V2rg[status] ?? []).filter((a) => V2rf.includes(a.perm));
}

export function V4t7(V2rh: V4s1): string[] {
  const V2ri: string[] = [];
  if (!V2rh.orgName.trim()) V2ri.push("Thiếu tên cơ quan, tổ chức nộp.");
  if (!V2rh.orgCode.trim()) V2ri.push("Thiếu mã cơ quan nộp.");
  if (!V2rh.fondCode.trim()) V2ri.push("Thiếu mã phông.");
  if (V2rh.items.length === 0)
    V2ri.push(
      "Mục lục cần tối thiểu 01 hồ sơ (Điều 20.3.k).",
    );
  const V2rj = new Set<string>();
  for (const V2rk of V2rh.items) {
    if (!V2rk.maHoSo.trim() || !V2rk.tieuDe.trim()) {
      V2ri.push(`Dòng Mục lục thiếu mã hồ sơ hoặc tiêu đề.`);
      break;
    }
  }
  for (const V2rl of V2rh.items) {
    const V2rm = V2rl.maHoSo.trim();
    if (V2rm && V2rj.has(V2rm)) {
      V2ri.push(
        `Mã hồ sơ trùng lặp trong Mục lục: ${V2rm} (Điều 24.1.a).`,
      );
    }
    V2rj.add(V2rm);
  }
  return V2ri;
}
