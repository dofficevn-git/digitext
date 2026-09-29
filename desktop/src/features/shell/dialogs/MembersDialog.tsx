// Copyright (c) 2026 DigiText. All rights reserved.
// Thành viên của một thư mục dự án trong Kho (vai trò theo dự án, chủ sở hữu luôn toàn quyền).

import { useState } from "react";
import { V460 } from "@/state";
import { V41z } from "@/mocks/fixtures";
import type { V3yi } from "@/domain/types";
import { Button, Dialog, IconButton, Input, Select, Table } from "@/ui";

export default function V4wv() {
  const { dialogTitle, closeDialog, members, setMembers } = V460();
  const [list, setList] = useState<V3yi[]>(members);
  const [ten, setTen] = useState("");
  const [vaiTro, setVaiTro] = useState<string>(V41z[1]);
  const roles = V41z.map((r) => ({ value: r, label: r }));

  const doiVaiTro = (i: number, role: string) => setList((d) => d.map((m, j) => (j === i ? { ...m, role } : m)));
  const go = (i: number) => setList((d) => d.filter((_m, j) => j !== i));
  const them = () => {
    const name = ten.trim();
    if (!name) return;
    setList((d) => [...d, { name, email: "", role: vaiTro, av: name[0].toUpperCase(), c: d.length % 5 }]);
    setTen("");
  };
  const luu = () => {
    setMembers(list);
    closeDialog();
  };

  return (
    <Dialog
      open
      onOpenChange={(o) => !o && closeDialog()}
      title="Thành viên"
      description={`${dialogTitle || "Dự án"}. Chủ sở hữu luôn có toàn quyền.`}
      size="md"
      footer={
        <>
          <Button onClick={closeDialog}>Huỷ</Button>
          <Button variant="primary" icon="luu" onClick={luu}>
            Lưu
          </Button>
        </>
      }
    >
      <Table.Root dense>
        <Table.Head>
          <tr>
            <Table.HeadCell>Thành viên</Table.HeadCell>
            <Table.HeadCell width={220}>Vai trò trong dự án</Table.HeadCell>
            <Table.HeadCell width={44}>
              <span className="sr-only">Gỡ</span>
            </Table.HeadCell>
          </tr>
        </Table.Head>
        <Table.Body>
          {list.map((m, i) => (
            <Table.Row key={`${m.name}-${i}`}>
              <Table.Cell>
                <span className="flex items-center gap-2">
                  <span className={`a c${m.c} inline-grid size-7 place-items-center rounded-full text-xs font-semibold`} aria-hidden>
                    {m.av}
                  </span>
                  <span>
                    {m.name}
                    {m.owner && <span className="ml-1 text-xs text-fg-subtle">chủ sở hữu</span>}
                    {m.email && <span className="block text-xs text-fg-subtle">{m.email}</span>}
                  </span>
                </span>
              </Table.Cell>
              <Table.Cell>{m.owner ? <span className="text-sm text-fg-subtle">Toàn quyền</span> : <Select size="sm" value={m.role} onChange={(v) => doiVaiTro(i, v)} options={roles} aria-label={`Vai trò của ${m.name}`} />}</Table.Cell>
              <Table.Cell>{!m.owner && <IconButton label={`Gỡ ${m.name} khỏi dự án`} icon="dong" size="sm" onClick={() => go(i)} />}</Table.Cell>
            </Table.Row>
          ))}
          {list.length === 0 && (
            <Table.FullRow colSpan={3}>
              <p className="px-3 py-3 text-sm text-fg-subtle">Chưa có thành viên nào.</p>
            </Table.FullRow>
          )}
        </Table.Body>
      </Table.Root>
      <div className="mt-3 flex flex-wrap gap-2">
        <Input size="sm" className="min-w-40 flex-1" placeholder="Tên thành viên" value={ten} onChange={(e) => setTen(e.target.value)} onKeyDown={(e) => e.key === "Enter" && them()} aria-label="Tên thành viên mới" />
        <Select size="sm" className="w-56" value={vaiTro} onChange={setVaiTro} options={roles} aria-label="Vai trò" />
        <Button size="sm" icon="them" onClick={them} disabled={!ten.trim()}>
          Thêm
        </Button>
      </div>
    </Dialog>
  );
}
