-- 5/11/2026: cho cac goi that bai vi quet virus chay lai sau khi nang gioi han ClamAV
-- (StreamMaxLength 100 MB -> 2000 MB). Chi dung den cac goi loi do clamd, khong dung goi khac.
begin;

select count(*) as se_chay_lai
from packages
where status = 'failed'
  and error like 'quét virus lỗi:%';

update packages
   set status      = 'queued',
       error       = null,
       attempts    = 0,
       locked_by   = null,
       lease_until = null,
       virus_scan  = null,
       updated_at  = now()
 where status = 'failed'
   and error like 'quét virus lỗi:%';

select status, count(*) from packages group by 1 order by 2 desc;

commit;
