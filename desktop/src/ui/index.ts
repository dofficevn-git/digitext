// Copyright (c) 2026 DigiText. All rights reserved.
// Bộ thành phần giao diện DigiText. Màn nghiệp vụ import từ đây, không import thẳng components/ui/*.

export { default as Button, buttonStyles, type ButtonProps } from "./Button";
export { default as IconButton } from "./IconButton";
export { default as Spinner } from "./Spinner";
export { default as Tip } from "./Tooltip";
export { default as Field, FormGrid, useFieldProps } from "./Field";
export { default as Input, Textarea, Select, Checkbox, Switch, type SelectOption } from "./Input";
export { default as PageHeader } from "./PageHeader";
export { default as Toolbar } from "./Toolbar";
export { default as Card } from "./Card";
export { default as Tabs, TabPanel, type TabItem } from "./Tabs";
export { default as Stepper, type StepItem } from "./Stepper";
export { default as KpiTile, KpiRow } from "./KpiTile";
export { default as EmptyState } from "./EmptyState";
export { default as Skeleton } from "./Skeleton";
export { default as Kbd } from "./Kbd";
export { default as Help } from "./Help";
export { default as Banner, ErrorBanner, type BannerTone } from "./Banner";
export { default as Menu, type MenuEntry, type MenuItem } from "./Menu";
export { default as Badge } from "./Badge";
export { default as Seg } from "./Seg";
export { default as Dialog } from "./Dialog";
export { default as Table, useTableSort, compareValues, collator, type Column, type SortState, type TableState } from "./Table";
export { default as Pagination, usePagination, pageSlice } from "./Pagination";
export { toast, notifyError, Toaster } from "./toast";
export { confirm, prompt, type ConfirmOptions, type PromptOptions } from "./dialogs/confirmStore";
export { default as ConfirmHost } from "./dialogs/ConfirmHost";
export { describeError, errorText, type UserError } from "@/lib/errors";
