INSERT INTO cl_ma_tt05 (id, project_id, nhom, ma, ten, so_tt) VALUES
    ('00000000-0000-4000-8000-00000c050101', NULL, 'che_do_su_dung', '01', 'Công khai', 1),
    ('00000000-0000-4000-8000-00000c050102', NULL, 'che_do_su_dung', '02', 'Sử dụng có điều kiện', 2),
    ('00000000-0000-4000-8000-00000c050103', NULL, 'che_do_su_dung', '03', 'Mật', 3),
    ('00000000-0000-4000-8000-00000c050201', NULL, 'muc_do_tin_cay', '01', 'Gốc điện tử', 1),
    ('00000000-0000-4000-8000-00000c050202', NULL, 'muc_do_tin_cay', '02', 'Số hoá', 2),
    ('00000000-0000-4000-8000-00000c050203', NULL, 'muc_do_tin_cay', '03', 'Hỗn hợp', 3),
    ('00000000-0000-4000-8000-00000c050301', NULL, 'tinh_trang_vat_ly', '01', 'Tốt', 1),
    ('00000000-0000-4000-8000-00000c050302', NULL, 'tinh_trang_vat_ly', '02', 'Bình thường', 2),
    ('00000000-0000-4000-8000-00000c050303', NULL, 'tinh_trang_vat_ly', '03', 'Hỏng', 3),
    ('00000000-0000-4000-8000-00000c050401', NULL, 'tinh_trang_du_phong', '01', 'Đã dự phòng', 1),
    ('00000000-0000-4000-8000-00000c050402', NULL, 'tinh_trang_du_phong', '02', 'Chưa dự phòng', 2)
ON CONFLICT DO NOTHING;
