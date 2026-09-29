// Copyright (c) 2026 DigiText. All rights reserved.
import { V460 } from "@/state";
import { V3z8 } from "@/domain/permissions";
import V461 from "@/features/shell/Topbar";
import HomeView from "@/features/home/HomeView";
import Sidebar from "@/features/shell/Sidebar";
import V47t from "@/features/shell/StatusBar";
import DriveView from "@/features/drive/DriveView";
import V4bk from "@/features/catalog/CatalogView";
import V4lc from "@/features/dossier/HoSoHubView";
import V4ns from "@/features/chinhly/ChinhLyHub";
import V4oe from "@/features/report/ReportHubView";
import V4p9 from "@/features/scan/ScanView";
import V4p4 from "@/features/editor/EditorView";
import V4qm from "@/features/upload/UploadView";
import V4qy from "@/features/sign/SignView";
import V4ra from "@/features/users/UsersView";
import V4rs from "@/features/settings/SettingsView";
import DongGoiView from "@/features/package/DongGoiView";
import WorkManageView from "@/features/work/WorkManageView";
import MyWorkView from "@/features/work/MyWorkView";
import V4t9 from "@/features/submission/SubmissionView";
import V4tk from "@/features/records/RecordsView";
import V4u7 from "@/features/access/AccessView";
import V4ur from "@/features/disposal/DisposalView";
import V4vc from "@/features/infoops/InfoView";
import V4vd from "@/features/infoops/BkView";
import V4wh from "@/features/server/ServerView";
import V4ws from "@/features/upload/TransferDock";
import V4wu from "@/features/shell/dialogs/CreateProjectDialog";
import V4wv from "@/features/shell/dialogs/MembersDialog";
import V4ww from "@/features/shell/dialogs/PreviewLightbox";

export default function V4wx() {
  const { view, dialog, previewDoc: V380, session: V381 } = V460();

  const V382 = !V381 || V3z8(V381.permissions, view);

  return (
    <div className="app-shell">
      <V461 />
      <Sidebar />

      <main className="content">
        {!V382 && (
          <section className="view active">
            <p className="muted p-6">Tài khoản của bạn không có quyền truy cập màn hình này.</p>
          </section>
        )}
        {V382 && view === "home" && <HomeView />}
        {V382 && view === "drive" && <DriveView />}
        {V382 && view === "catalog" && <V4bk />}
        {V382 && view === "dossier" && <V4lc />}
        {V382 && view === "khogia" && <V4ns />}
        {V382 && view === "report" && <V4oe />}
        {V382 && view === "scan" && <V4p9 />}
        {V382 && view === "editor" && <V4p4 />}
        {V382 && view === "upload" && <V4qm />}
        {V382 && view === "sign" && <V4qy />}
        {V382 && view === "users" && <V4ra />}
        {V382 && view === "settings" && <V4rs />}
        {V382 && view === "package" && <DongGoiView />}
        {V382 && view === "submission" && <V4t9 />}
        {V382 && view === "records" && <V4tk />}
        {V382 && view === "access" && <V4u7 />}
        {V382 && view === "disposal" && <V4ur />}
        {V382 && view === "info" && <V4vc />}
        {V382 && view === "bk" && <V4vd />}
        {V382 && view === "server" && <V4wh />}
        {V382 && view === "workManage" && <WorkManageView />}
        {V382 && view === "work" && <MyWorkView />}
      </main>

      <V47t />

      {}
      <V4ws />

      {dialog === "create" && <V4wu />}
      {dialog === "members" && <V4wv />}
      {V380 && <V4ww />}
    </div>
  );
}
