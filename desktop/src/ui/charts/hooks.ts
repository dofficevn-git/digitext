// Copyright (c) 2026 DigiText. All rights reserved.
// Hook dùng chung cho biểu đồ: đo bề rộng phần tử, chỉ số đang hover / chọn bằng phím.

import { useCallback, useEffect, useRef, useState, type RefObject } from "react";

/** Đo bề rộng phần tử (ResizeObserver), 0 khi chưa gắn. */
export function useWidth<E extends HTMLElement>(): [RefObject<E>, number] {
  const ref = useRef<E>(null);
  const [w, setW] = useState(0);
  useEffect(() => {
    const el = ref.current;
    if (!el) return;
    const ro = new ResizeObserver((es) => {
      for (const e of es) setW(Math.floor(e.contentRect.width));
    });
    ro.observe(el);
    setW(Math.floor(el.getBoundingClientRect().width));
    return () => ro.disconnect();
  }, []);
  return [ref, w];
}

/** Chỉ số đang hover/chọn bằng phím trong một dãy n điểm. */
export function useHoverIndex(n: number) {
  const [i, setI] = useState<number | null>(null);
  const step = useCallback((dir: -1 | 1) => setI((cur) => (n <= 0 ? null : cur == null ? (dir > 0 ? 0 : n - 1) : Math.max(0, Math.min(n - 1, cur + dir)))), [n]);
  const clear = useCallback(() => setI(null), []);
  useEffect(() => {
    if (i != null && i >= n) setI(n > 0 ? n - 1 : null);
  }, [n, i]);
  return { i, setI, step, clear };
}
