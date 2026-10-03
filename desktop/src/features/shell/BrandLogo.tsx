// Logo DIGITEXT (bàn phím + hai bàn tay) lấy từ hệ thống cũ D:\Digitext.2025\Digitext\digitexticon.ico.
import logo from "@/assets/digitext-logo.png";

export default function BrandLogo({ className }: { className?: string }) {
  return <img src={logo} alt="" aria-hidden draggable={false} className={className} />;
}
