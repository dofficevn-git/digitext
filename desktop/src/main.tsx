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
import { apBoMau, boMauDaChon } from "@/features/shell/boMau";

document.title = V3wu.appName;
// Gắn bộ màu trước khi vẽ để màn đăng nhập / màn nạp không nháy tím rồi mới đổi.
// Xem trước khi dev: ?bomau=do365|zalo|teams (cùng kiểu với ?view= / ?role=).
const boMauThu = import.meta.env.DEV ? new URLSearchParams(location.search).get("bomau") : null;
apBoMau(boMauThu === "do365" || boMauThu === "zalo" || boMauThu === "teams" ? boMauThu : boMauDaChon());

createRoot(document.getElementById("root")!).render(
  <StrictMode>
    <ErrorBoundary>
      <V45z>
        <V4x8 />
      </V45z>
    </ErrorBoundary>
  </StrictMode>,
);
