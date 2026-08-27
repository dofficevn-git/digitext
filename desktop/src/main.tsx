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

document.title = V3wu.appName;

createRoot(document.getElementById("root")!).render(
  <StrictMode>
    <ErrorBoundary>
      <V45z>
        <V4x8 />
      </V45z>
    </ErrorBoundary>
  </StrictMode>,
);
