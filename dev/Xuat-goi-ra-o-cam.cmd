@echo off
rem Bam dup de xuat goi tin tu kho may chu ra o cam truc tiep vao may nay.
rem Khong go lenh: chi tra loi hai cau hoi roi cho. Chay lai duoc, tep da co thi bo qua.
setlocal
cd /d "%~dp0.."

echo ===============================================
echo   XUAT GOI TIN RA O CAM VAO MAY CHU
echo ===============================================
echo.
echo  Cac o dang co tren may:
wmic logicaldisk get caption,volumename,freespace 2>nul | findstr /r "[A-Z]:"
echo.

set "DICH="
set /p DICH=Thu muc dich (vd F:\Goi-AIP), Enter = F:\Goi-AIP :
if "%DICH%"=="" set "DICH=F:\Goi-AIP"

echo.
echo  Loai goi:  1 = AIP ho so (bao quan)   2 = SIP ho so (nop luu)   3 = tat ca
set "L="
set /p L=Chon 1 / 2 / 3, Enter = 1 :
if "%L%"=="" set "L=1"
set "LOAI=--loai aip_hoso"
if "%L%"=="2" set "LOAI=--loai sip_hoso"
if "%L%"=="3" set "LOAI="

echo.
echo  Dich : %DICH%
echo  Loai : %LOAI%
echo.
python -I -X utf8 dev\xuat-goi.py --thu-muc "%DICH%" %LOAI% --luong 8

echo.
echo ===============================================
pause
