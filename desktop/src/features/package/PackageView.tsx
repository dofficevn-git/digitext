// Copyright (c) 2026 DigiText. All rights reserved.
// Tab "Nâng cao" của Đóng gói: tạo gói với đầy đủ trường thông tin (SIP / AIP / DIP), chọn tài liệu đã nộp trên
// máy chủ theo mã hồ sơ. Danh sách gói xem ở tab Gói tin.

import { useCallback, useMemo, useState } from "react";
import { V41p } from "@/live/useLive";
import { V4ei, type V4eg } from "@/api/docs";
import { isTauri } from "@/api/ipc";
import { V4rv } from "@/api/packaging";
import { V4s2, type V4rx } from "@/api/submissions";
import { V460 } from "@/state";
import { V4sk, V4sn, V4ss, V4su, type V4sj, type V4sm } from "./packageWire";
import { Banner, Button, Card, Field, FormGrid, Input, notifyError, Select, Table, toast, Toolbar, Checkbox, type Column } from "@/ui";

export default function V4sx() {
  const { session } = V460();
  const [form, setForm] = useState<V4sm>(V4sn);
  const [hoSoRef, setHoSoRef] = useState("");
  const [docs, setDocs] = useState<V4eg[]>([]);
  const [sel, setSel] = useState<Set<string>>(new Set());
  const [subs, setSubs] = useState<V4rx[]>([]);
  const [subId, setSubId] = useState("");
  const [sttFrom, setSttFrom] = useState("");
  const [sttTo, setSttTo] = useState("");
  const [busy, setBusy] = useState<"" | "load" | "create">("");
  const [errs, setErrs] = useState<string[]>([]);
  const set = <K extends keyof V4sm>(k: K, v: V4sm[K]) => setForm((f) => ({ ...f, [k]: v }));

  const isSip = form.kind === "sip_hoso" || form.kind === "sip_tailieu";
  const isHoSo = form.kind === "aip_hoso" || form.kind === "sip_hoso";
  const isDip = form.kind === "dip";

  const loadSubs = useCallback(() => {
    V4s2()
      .then((rs) => setSubs(rs.filter((r) => r.status === "confirmed" || r.status === "awaiting_reception")))
      .catch(() => setSubs([]));
  }, []);
  V41p(loadSubs, { topics: ["submissions"], enabled: isSip && isTauri() });

  const napTaiLieu = async () => {
    setBusy("load");
    try {
      const all = await V4ei();
      const ref = hoSoRef.trim();
      const rows = ref ? all.filter((d) => (d.hoSoRef ?? "") === ref) : all;
      setDocs(rows);
      setSel(new Set(rows.map((d) => d.id)));
      if (rows.length === 0) toast.warning(ref ? `Không có tài liệu nào trên máy chủ khớp mã hồ sơ "${ref}".` : "Máy chủ chưa có tài liệu nào trong phạm vi của bạn.");
    } catch (e) {
      notifyError(e, "nạp tài liệu");
    } finally {
      setBusy("");
    }
  };
  const chon = useMemo(() => docs.filter((d) => sel.has(d.id)).map((d) => ({ documentId: d.id, filename: d.filename })), [docs, sel]);

  const taoGoi = async () => {
    const { request, errors } = V4su(form, chon);
    const range = isSip ? V4ss(sttFrom, sttTo) : { range: null, error: null };
    const all = range.error ? [...errors, range.error] : errors;
    setErrs(all);
    if (!request || all.length > 0) return;
    setBusy("create");
    try {
      const row = await V4rv({ projectId: session?.projectId ?? null, hoSoRef: hoSoRef.trim() || null, submissionId: isSip && subId ? subId : null, sttFrom: range.range ? range.range[0] : null, sttTo: range.range ? range.range[1] : null, request });
      toast.success(`Đã tạo yêu cầu đóng gói "${row.label}". Máy chủ đang dựng gói, xem ở tab Gói tin.`);
    } catch (e) {
      notifyError(e, "tạo gói");
    } finally {
      setBusy("");
    }
  };

  const sel3 = (k: keyof V4sm, label: string, opts: [string, string][]) => (
    <Field label={label}>
      <Select value={form[k] as string} onChange={(v) => set(k, v as never)} options={opts.map(([value, l]) => ({ value, label: l }))} />
    </Field>
  );
  const txt = (k: keyof V4sm, label: string, placeholder?: string, hint?: string) => (
    <Field label={label} hint={hint}>
      <Input value={form[k] as string} onChange={(e) => set(k, e.target.value as never)} placeholder={placeholder} />
    </Field>
  );
  const docCols: Column<V4eg>[] = [
    { key: "filename", header: "Tệp", className: "whitespace-normal" },
    { key: "hoSoRef", header: "Mã hồ sơ", width: 150, cell: (d) => d.hoSoRef ?? "—" },
    { key: "pageCount", header: "Trang", align: "right", width: 70, cell: (d) => d.pageCount ?? "—" },
  ];

  return (
    <div className="space-y-4">
      <Card title="Thông tin gói" description="Điền đầy đủ trường theo TT05, dùng khi cần kiểm soát từng giá trị.">
        <FormGrid cols={3}>
          {sel3("kind", "Loại gói", (Object.keys(V4sk) as V4sj[]).map((k) => [k, V4sk[k]]))}
          {txt("fondCode", "Mã phông", "G09")}
          {txt("archiveOrgCode", "Mã cơ quan lưu trữ", "V01")}
          {!isDip && (
            <>
              {txt("fileCode", isHoSo ? "Mã hồ sơ lưu trữ" : "Mã gói tin", "V01.G09.2021.01")}
              {txt("title", "Tiêu đề")}
              {sel3("maintenance", "Thời hạn lưu trữ", [["01", "Vĩnh viễn"], ["02", "70 năm"], ["03", "50 năm"], ["04", "30 năm"], ["05", "20 năm"], ["06", "10 năm"], ["07", "Khác"]])}
              {sel3("mode", "Chế độ sử dụng", [["01", "Công khai"], ["02", "Có điều kiện"], ["03", "Mật"]])}
              {sel3("confidenceLevel", "Mức độ tin cậy", [["01", "Gốc điện tử"], ["02", "Số hoá"], ["03", "Hỗn hợp"]])}
            </>
          )}
          {isHoSo && (
            <>
              {txt("numberOfPaper", "Số lượng tờ")}
              {txt("paperFileCode", "Mã hồ sơ gốc giấy", "V01.01.001")}
            </>
          )}
          {(form.kind === "aip_tailieu" || form.kind === "sip_tailieu") && sel3("source", "Nguồn gốc", [["0", "Văn bản đi"], ["1", "Văn bản đến"]])}
          {isSip && (
            <>
              {txt("submissionAgreement", "Mã yêu cầu nộp", "NL.BNV.2026.01", "Thoả thuận nộp lưu")}
              {txt("zipOrdinal", "Số thứ tự trong lần nộp")}
              <Field label="Lần nộp" hint="Gắn gói vào lần nộp đang mở">
                <Select value={subId} onChange={setSubId} placeholder="Không gắn" options={subs.map((r) => ({ value: r.id, label: `${r.maYeuCau} · ${r.orgName}` }))} />
              </Field>
              <Field label="STT mục lục từ" hint="Để trống thì tự suy">
                <Input value={sttFrom} onChange={(e) => setSttFrom(e.target.value)} placeholder="1" />
              </Field>
              <Field label="STT mục lục đến">
                <Input value={sttTo} onChange={(e) => setSttTo(e.target.value)} placeholder="1" />
              </Field>
            </>
          )}
          {isDip && (
            <>
              {txt("requestId", "Mã yêu cầu khai thác", "YC.2026.0001")}
              {txt("requestDate", "Ngày yêu cầu", "DD/MM/YYYY")}
              {sel3("purpose", "Mục đích", [["01", "Cá nhân"], ["02", "Công vụ"], ["03", "Công vụ đặc biệt"]])}
              <Field label="Kèm PREMIS" inline>
                <Checkbox checked={form.includePremis} onChange={(v) => set("includePremis", v)} />
              </Field>
              {txt("certMaXacThuc", "Mã xác thực lưu trữ", "XT.2026.0001")}
              {txt("certTenLuuTru", "Tên lưu trữ lịch sử")}
              {txt("certNguoiNhanHoTen", "Người nhận: họ tên")}
              {txt("certNguoiNhanDinhDanh", "Người nhận: mã định danh")}
              {txt("certMaLuuTruGoc", "Mã lưu trữ tài liệu gốc")}
              {txt("certSoKyHieuGoc", "Số, ký hiệu tài liệu gốc")}
              {txt("certTenLoai", "Tên loại tài liệu", "Quyết định")}
              {txt("certTrichYeu", "Trích yếu, tiêu đề")}
              {txt("certMucDich", "Mục đích sử dụng")}
              {txt("certThoiHan", "Thời hạn sử dụng")}
              {txt("certSoLuongBan", "Số lượng bản")}
            </>
          )}
        </FormGrid>
      </Card>

      <Card title="Tài liệu trong gói" description="Nạp tài liệu đã nộp trên máy chủ, lọc theo mã hồ sơ nếu cần.">
        <Toolbar className="mb-3">
          <Field label="Mã hồ sơ trên máy chủ" className="w-72">
            <Input size="sm" value={hoSoRef} onChange={(e) => setHoSoRef(e.target.value)} placeholder="Hộp 01/Hồ sơ 001" />
          </Field>
          <Button size="sm" icon="lamMoi" loading={busy === "load"} disabled={!isTauri()} onClick={() => void napTaiLieu()} className="self-end">
            Nạp tài liệu từ máy chủ
          </Button>
          <Toolbar.Spacer />
          <span className="self-end text-sm text-fg-subtle">{chon.length.toLocaleString("vi-VN")} tài liệu đã chọn</span>
        </Toolbar>
        {docs.length > 0 && <Table columns={docCols} rows={docs} getRowId={(d) => d.id} selection={{ selected: sel, onChange: setSel }} maxHeight={320} dense />}
        {errs.length > 0 && (
          <Banner tone="danger" className="mt-3" title="Chưa tạo được gói">
            <ul className="list-disc pl-4">
              {errs.map((e) => (
                <li key={e}>{e}</li>
              ))}
            </ul>
          </Banner>
        )}
        <div className="mt-3">
          <Button variant="primary" icon="goi" loading={busy === "create"} disabled={chon.length === 0} onClick={() => void taoGoi()}>
            Tạo gói ({chon.length.toLocaleString("vi-VN")} tài liệu)
          </Button>
        </div>
      </Card>
    </div>
  );
}
