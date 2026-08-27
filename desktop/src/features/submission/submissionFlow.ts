// Copyright (c) 2026 DigiText. All rights reserved.

import type { V3yx } from "@/domain/permissions";
import type { V4s1, V4s5 } from "@/api/submissions";

export const V4t4: Record<string, string> = {
  registered: "xJDDoyDEkcSDbmcga8O9IChjaOG7nSB4w6FjIG5o4bqtbik=",
  confirmed: "xJDDoyB4w6FjIG5o4bqtbiDigJQgxJFhbmcgbuG7mXAgZ8OzaQ==",
  refused: "VOG7qyBjaOG7kWkgdGh1",
  awaiting_reception: "Q2jhu50gdGnhur9wIG5o4bqtbg==",
  received: "xJDDoyB0aeG6v3Agbmjhuq1uIOKAlCDEkWFuZyB44butIGzDvQ==",
  approved: "xJDDoyBwaMOqIGR1eeG7h3QgKFNJUOKGkkFJUCk=",
  completed: "SG/DoG4gdGjDoG5o",
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
        label: "xJDhu5NuZyDDvSB0aHUgKMSQaeG7gXUgMjEuMik=",
        perm: "submission.review",
      },
      {
        action: "refuse",
        label: "VOG7qyBjaOG7kWkgdGh1ICjEkGnhu4F1IDIxLjMp",
        perm: "submission.review",
        requiresNote: true,
      },
    ],
    confirmed: [
      {
        action: "reconcile",
        label: "xJDhu5FpIGNoaeG6v3UgTeG7pWMgbOG7pWMgKMSQaeG7gXUgMjIuMy5kKQ==",
        perm: "submission.view",
      },
    ],
    awaiting_reception: [
      {
        action: "receive",
        label: "VGnhur9wIG5o4bqtbiAoxJBp4buBdSAyMyk=",
        perm: "submission.review",
      },
      { action: "reconcile", label: "xJDhu5FpIGNoaeG6v3UgbOG6oWk=", perm: "submission.view" },
    ],
    received: [
      {
        action: "approve",
        label: "UGjDqiBkdXnhu4d0ICsgY2h1eeG7g24gU0lQ4oaSQUlQICjEkGnhu4F1IDI1KQ==",
        perm: "submission.approve",
      },
    ],
    approved: [
      {
        action: "complete",
        label: "SG/DoG4gdGjDoG5oIHRodSBu4buZcCAoxJBp4buBdSAyNS41KQ==",
        perm: "submission.approve",
      },
    ],
  };
  return (V2rg[status] ?? []).filter((a) => V2rf.includes(a.perm));
}

export function V4t7(V2rh: V4s1): string[] {
  const V2ri: string[] = [];
  if (!V2rh.orgName.trim()) V2ri.push("VGhp4bq/dSB0w6puIGPGoSBxdWFuLCB04buVIGNo4bupYyBu4buZcC4=");
  if (!V2rh.orgCode.trim()) V2ri.push("VGhp4bq/dSBtw6MgY8ahIHF1YW4gbuG7mXAu");
  if (!V2rh.fondCode.trim()) V2ri.push("VGhp4bq/dSBtw6MgcGjDtG5nLg==");
  if (V2rh.items.length === 0)
    V2ri.push(
      "TeG7pWMgbOG7pWMgY+G6p24gdOG7kWkgdGhp4buDdSAwMSBo4buTIHPGoSAoxJBp4buBdSAyMC4zLmspLg==",
    );
  const V2rj = new Set<string>();
  for (const V2rk of V2rh.items) {
    if (!V2rk.maHoSo.trim() || !V2rk.tieuDe.trim()) {
      V2ri.push(`RMOybmcgTeG7pWMgbOG7pWMgdGhp4bq/dSBtw6MgaOG7kyBzxqEgaG/hurdjIHRpw6p1IMSR4buBLg==`);
      break;
    }
  }
  for (const V2rl of V2rh.items) {
    const V2rm = V2rl.maHoSo.trim();
    if (V2rm && V2rj.has(V2rm)) {
      V2ri.push(
        `TcOjIGjhu5Mgc8ahIHRyw7luZyBs4bq3cCB0cm9uZyBN4bulYyBs4bulYzog${V2rm}ICjEkGnhu4F1IDI0LjEuYSku`,
      );
    }
    V2rj.add(V2rm);
  }
  return V2ri;
}
