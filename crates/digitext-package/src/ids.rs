// Copyright (c) 2026 DigiText. All rights reserved.
use crate::model::PackageKind;
pub trait T9n {
    fn uuid(&mut self) -> String;
}
pub struct Tic;
impl T9n for Tic {
    fn uuid(&mut self) -> String {
        uuid::Uuid::new_v4().to_string().to_uppercase()
    }
}
pub struct Tl1(pub u64);
impl T9n for Tl1 {
    fn uuid(&mut self) -> String {
        self.0 += 1;
        format!("00000000-0000-4000-8000-{:012X}", self.0)
    }
}
pub fn f_2op(r_3yq: &mut dyn T9n) -> String {
    format!("uuid-{}", r_3yq.uuid())
}
pub fn file_id(r_3yr: &mut dyn T9n) -> String {
    format!("ID-{}", r_3yr.uuid())
}
pub fn f_1x2(r_3yu: PackageKind, r_3ys: &str, r_3yt: &mut dyn T9n) -> String {
    if r_3yu.uses_urn_objid() {
        format!("urn:{}:uuid:{}", r_3ys, r_3yt.uuid())
    } else {
        f_2op(r_3yt)
    }
}
pub fn folder_name(r_3yv: &str) -> String {
    r_3yv.replace(':', "_")
}
pub fn f_2sb(r_3yw: PackageKind, r_3yy: &str, r_3yz: Option<u32>) -> String {
    match (r_3yw, r_3yz) {
        (PackageKind::SipHoso | PackageKind::SipTailieu, Some(r_3yx)) => {
            format!("{}_{}.zip", r_3yx, r_3yy)
        }
        _ => format!("{}.zip", folder_name(r_3yy)),
    }
}
