// Copyright (c) 2026 DigiText. All rights reserved.

import V4wm from "./CollectStatusWidget";
import V4wp from "./UploadStatusWidget";
import "./uploadWidget.css";

export default function V4ws() {
  return (
    <div className="uw-dock">
      <V4wm />
      <V4wp />
    </div>
  );
}
