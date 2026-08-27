// Copyright (c) 2026 DigiText. All rights reserved.

import { V460 } from "@/state";
import Banner from "./Banner";
import { Icon } from "@/components/Icon";

export default function V4hd() {
  const { goto } = V460();
  return (
    <>
      <Banner
        tone="info"
        text="S8O9IHPhu5EgUEFkRVMgdHLDqm4gZGVza3RvcCBkw7luZyBVU0IgdG9rZW4gcXVhIG3DoG4g4oCcS8O9IHPhu5HigJ0uIELhuqVtIG7DunQgYsOqbiBkxrDhu5tpIMSR4buDIG3hu58gaMOgbmcgxJHhu6NpIGvDvSBz4buRLg=="
      />
      <div className="py-6 px-1">
        <button className="btn primary" onClick={() => goto("sign")}>
          <Icon name="kySo" />
          Mở màn Ký số
        </button>
      </div>
    </>
  );
}
