// Copyright (c) 2026 DigiText. All rights reserved.

import { Component } from "react";
import type { ErrorInfo, ReactNode } from "react";
import { V3wu } from "@/config";

interface V4x9 {
  children: ReactNode;
}
interface State {
  error: Error | null;
}

export default class ErrorBoundary extends Component<V4x9, State> {
  state: State = { error: null };

  static getDerivedStateFromError(V3ac: Error): State {
    return { error: V3ac };
  }

  componentDidCatch(V3ad: Error, info: ErrorInfo) {
    console.error(
      `[${V3wu.appName}] lỗi render không bắt được:`,
      V3ad,
      info.componentStack,
    );
  }

  private reload = () => {
    this.setState({ error: null });
    if (typeof window !== "undefined") window.location.reload();
  };

  render() {
    const { error: V3ae } = this.state;
    if (!V3ae) return this.props.children;

    return (
      <div
        role="alert"
        style={{
          minHeight: "100vh",
          display: "grid",
          placeItems: "center",
          padding: 24,
          background: "#1e2b2f",
          color: "#eee",

          fontFamily: '"Segoe UI", -apple-system, BlinkMacSystemFont, sans-serif',
        }}
      >
        <div className="max-w-120 text-center">
          <h1 className="text-xl mb-2" style={{ color: "#fff" }}>
            Đã xảy ra lỗi
          </h1>
          <p className="opacity-80 mb-4">
            Ứng dụng gặp sự cố không mong muốn. Bạn có thể tải lại để tiếp tục.
          </p>
          <pre
            style={{
              textAlign: "left",
              background: "#000",
              color: "#f88",
              padding: 12,
              borderRadius: 8,
              overflow: "auto",
              fontSize: 12,
              marginBottom: 16,
            }}
          >
            {V3ae.message}
          </pre>
          <button className="btn primary" onClick={this.reload}>
            Tải lại ứng dụng
          </button>
        </div>
      </div>
    );
  }
}
