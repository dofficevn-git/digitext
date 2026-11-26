// Copyright (c) 2026 DigiText. All rights reserved.
import { StrictMode } from "react";
import { createRoot } from "react-dom/client";
import V4x8 from "@/app/App";
import ErrorBoundary from "@/app/ErrorBoundary";
import { V45z } from "@/state";
import { V3wu } from "@/config";
import "@/styles/theme.css";
import "@/styles/digitext.css";
import "@/styles/responsive.css";
import "@/styles/bo-mau.css";
import "@/styles/nen-toi.css";
import { apBoMau, apNenToi, boMauDaChon, nenToiDaChon } from "@/features/shell/boMau";

document.title = V3wu.appName;
// Gắn bộ màu trước khi vẽ để màn đăng nhập / màn nạp không nháy tím rồi mới đổi.
// Xem trước khi dev: ?bomau=do365|zalo|teams và ?nentoi=1|0 (cùng kiểu với ?view= / ?role=).
const boMauThu = import.meta.env.DEV ? new URLSearchParams(location.search).get("bomau") : null;
apBoMau(boMauThu === "do365" || boMauThu === "zalo" || boMauThu === "teams" ? boMauThu : boMauDaChon());
const nenToiThu = import.meta.env.DEV ? new URLSearchParams(location.search).get("nentoi") : null;
apNenToi(nenToiThu === "1" ? true : nenToiThu === "0" ? false : nenToiDaChon());

createRoot(document.getElementById("root")!).render(
  <StrictMode>
    <ErrorBoundary>
      <V45z>
        <V4x8 />
      </V45z>
    </ErrorBoundary>
  </StrictMode>,
);
