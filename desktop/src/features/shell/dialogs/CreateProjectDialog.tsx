// Copyright (c) 2026 DigiText. All rights reserved.
// Tạo thư mục con trong Kho tài liệu (mở từ menu Tạo của Kho).

import { useState } from "react";
import { V460 } from "@/state";
import { Button, Dialog, Field, Input } from "@/ui";

export default function V4wu() {
  const { dialogTitle, closeDialog, createFolder } = V460();
  const [name, setName] = useState("");
  const [err, setErr] = useState<string | undefined>();
  const tao = () => {
    const t = name.trim();
    if (!t) {
      setErr("Nhập tên thư mục.");
      return;
    }
    if (/[\\/:*?"<>|]/.test(t)) {
      setErr("Tên không được chứa các ký tự \\ / : * ? \" < > |");
      return;
    }
    createFolder(t);
    closeDialog();
  };
  return (
    <Dialog
      open
      onOpenChange={(o) => !o && closeDialog()}
      title={dialogTitle || "Tạo thư mục"}
      size="sm"
      footer={
        <>
          <Button onClick={closeDialog}>Huỷ</Button>
          <Button variant="primary" icon="them" onClick={tao}>
            Tạo
          </Button>
        </>
      }
    >
      <form
        onSubmit={(e) => {
          e.preventDefault();
          tao();
        }}
      >
        <Field label="Tên thư mục" required error={err}>
          <Input
            value={name}
            autoFocus
            placeholder="Ví dụ: Hộp 01"
            onChange={(e) => {
              setName(e.target.value);
              setErr(undefined);
            }}
          />
        </Field>
      </form>
    </Dialog>
  );
}
