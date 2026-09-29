// Copyright (c) 2026 DigiText. All rights reserved.
// Hiển thị yêu cầu confirm()/prompt() từ confirmStore. Gắn một lần ở gốc ứng dụng.

import { useEffect, useMemo, useState } from "react";
import {
  AlertDialog,
  AlertDialogContent,
  AlertDialogDescription,
  AlertDialogFooter,
  AlertDialogHeader,
  AlertDialogTitle,
} from "@/components/ui/alert-dialog";
import Button from "../Button";
import Field from "../Field";
import Input, { Select, Textarea } from "../Input";
import { finish, subscribe, type Request } from "./confirmStore";

export default function ConfirmHost() {
  const [req, setReq] = useState<Request | null>(null);
  useEffect(() => subscribe(setReq), []);
  return (
    <AlertDialog open={!!req} onOpenChange={(open) => !open && req && finish(req.id, req.kind === "confirm" ? false : null)}>
      <AlertDialogContent>{req && (req.kind === "confirm" ? <ConfirmBody req={req} /> : <PromptBody req={req} />)}</AlertDialogContent>
    </AlertDialog>
  );
}

function ConfirmBody({ req }: { req: Extract<Request, { kind: "confirm" }> }) {
  const { opts } = req;
  const [typed, setTyped] = useState("");
  const gate = opts.typeToConfirm;
  const ok = !gate || typed.trim() === gate;
  return (
    <form
      onSubmit={(e) => {
        e.preventDefault();
        if (ok) finish(req.id, true);
      }}
    >
      <AlertDialogHeader>
        <AlertDialogTitle>{opts.title}</AlertDialogTitle>
        <AlertDialogDescription className={opts.body ? "whitespace-pre-line" : "sr-only"}>{opts.body ?? opts.title}</AlertDialogDescription>
      </AlertDialogHeader>
      {gate && (
        <div className="mt-3">
          <Field label={`Gõ "${gate}" để xác nhận`}>
            <Input value={typed} onChange={(e) => setTyped(e.target.value)} autoFocus autoComplete="off" />
          </Field>
        </div>
      )}
      <AlertDialogFooter className="mt-4">
        <Button type="button" onClick={() => finish(req.id, false)} autoFocus={!gate}>
          {opts.cancelLabel ?? "Huỷ"}
        </Button>
        <Button type="submit" variant={opts.danger ? "danger" : "primary"} disabled={!ok}>
          {opts.confirmLabel ?? (opts.danger ? "Xoá" : "Đồng ý")}
        </Button>
      </AlertDialogFooter>
    </form>
  );
}

function PromptBody({ req }: { req: Extract<Request, { kind: "prompt" }> }) {
  const { opts } = req;
  const [v, setV] = useState(opts.initial ?? "");
  const [touched, setTouched] = useState(false);
  const err = useMemo(() => {
    if (opts.required && !v.trim()) return "Chưa nhập giá trị.";
    return opts.validate?.(v) ?? null;
  }, [v, opts]);
  const type = opts.type ?? "text";
  return (
    <form
      onSubmit={(e) => {
        e.preventDefault();
        setTouched(true);
        if (!err) finish(req.id, v);
      }}
    >
      <AlertDialogHeader>
        <AlertDialogTitle>{opts.title}</AlertDialogTitle>
        <AlertDialogDescription className={opts.body ? "" : "sr-only"}>{opts.body ?? opts.title}</AlertDialogDescription>
      </AlertDialogHeader>
      <div className="mt-3">
        <Field label={opts.label ?? "Giá trị"} error={touched ? err : undefined} required={opts.required}>
          {type === "textarea" ? (
            <Textarea value={v} onChange={(e) => setV(e.target.value)} placeholder={opts.placeholder} autoFocus rows={4} />
          ) : type === "select" ? (
            <Select value={v} onChange={setV} options={opts.options ?? []} placeholder={opts.placeholder ?? "— Chọn —"} autoFocus />
          ) : (
            <Input type={type === "number" ? "number" : type === "date" ? "date" : "text"} value={v} onChange={(e) => setV(e.target.value)} placeholder={opts.placeholder} autoFocus autoComplete="off" />
          )}
        </Field>
      </div>
      <AlertDialogFooter className="mt-4">
        <Button type="button" onClick={() => finish(req.id, null)}>
          {opts.cancelLabel ?? "Huỷ"}
        </Button>
        <Button type="submit" variant={opts.danger ? "danger" : "primary"} disabled={touched && !!err}>
          {opts.confirmLabel ?? "Đồng ý"}
        </Button>
      </AlertDialogFooter>
    </form>
  );
}
