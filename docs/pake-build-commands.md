# Hướng Dẫn Các Lệnh Build Và Đóng Gói App Pake (Windows, macOS, Linux)

Tài liệu này tổng hợp toàn bộ các lệnh cần thiết để chuẩn bị môi trường, biên dịch CLI và đóng gói (package/build) ứng dụng Desktop từ trang web (ví dụ: Vbee / CloudCamera) trên cả 3 nền tảng: Windows, macOS và Linux.

---

## 1. Yêu Cầu Môi Trường (Prerequisites)

Trước khi chạy bất kỳ lệnh nào, hãy đảm bảo máy tính đã cài đặt các công cụ sau:
1. **Node.js**: Phiên bản `>= 18.0.0`.
2. **pnpm**: Trình quản lý package của Node (khuyên dùng).
3. **Rust Toolchain**: Cài đặt thông qua [rustup.rs](https://rustup.rs/).
4. **Cấu hình Native SDK từng hệ điều hành**:
   * **Windows**: Visual Studio Build Tools (C++ Cài đặt kèm theo MSVC v143 và Windows 10/11 SDK).
   * **macOS**: Xcode Command Line Tools (`xcode-select --install`).
   * **Linux (Ubuntu/Debian)**: Cài đặt các thư viện hệ thống:
     ```bash
     sudo apt update
     sudo apt install -y libwebkit2gtk-4.1-dev build-essential curl wget file libssl-dev libgtk-3-dev libayatana-appindicator3-dev librsvg2-dev
     ```

---

## 2. Các Lệnh Biên Dịch CLI Của Pake

Các lệnh này thực hiện chuẩn bị và biên dịch lõi Pake CLI trước khi dùng nó để đóng gói URL:

```bash
# 1. Cài đặt các thư viện phụ thuộc của Pake
pnpm install

# 2. Biên dịch Pake CLI từ TypeScript sang JavaScript (ra file dist/cli.js)
pnpm run cli:build

# 3. Biên dịch CLI ở chế độ Development (Watch mode - tự động build lại khi sửa file bin/)
pnpm run cli:dev
```

---

## 3. Các Lệnh Đóng Gói Ứng Dụng Từ Trang Web (URL / Localhost)

Sử dụng Pake CLI đã biên dịch ở trên (`dist/cli.js`) để đóng gói trang web thành file cài đặt.

### A. Dành cho Windows (Đóng gói ra `.exe` / `.msi`)

* **Nếu chạy trên Git Bash (Khuyên dùng)**:
  Sử dụng tiền tố `MSYS_NO_PATHCONV=1` để ngăn trình giả lập Bash tự dịch sai đường dẫn `/live` của tham số `--window`:
  ```bash
  MSYS_NO_PATHCONV=1 node dist/cli.js http://localhost:4201 --name CloudCamera --window live=/live --multi-window --show-system-tray
  ```

* **Nếu chạy trên Windows PowerShell**:
  ```powershell
  node dist/cli.js http://localhost:4201 --name CloudCamera --window live=/live --multi-window --show-system-tray
  ```

### B. Dành cho macOS (Đóng gói ra `.app` / `.dmg`)

Chạy trực tiếp trong Terminal của macOS (zsh/bash):
```bash
node dist/cli.js http://localhost:4201 --name CloudCamera --window live=/live --multi-window --show-system-tray
```

### C. Dành cho Linux (Đóng gói ra `.AppImage` / `.deb`)

Chạy trong Terminal Linux:
```bash
node dist/cli.js http://localhost:4201 --name CloudCamera --window live=/live --multi-window --show-system-tray
```

---

## 4. Các Tham Số CLI Quan Trọng Khi Đóng Gói (Vbee / CloudCamera)

* `--name <TênApp>`: Đặt tên cho file cài đặt và ứng dụng khi hiển thị.
* `--window <nhãn>=<đường_dẫn>`: Khai báo router/đường dẫn cho cửa sổ phụ (Ví dụ: `live=/live` để định nghĩa cửa sổ live stream).
* `--multi-window`: Kích hoạt chế độ đa cửa sổ (Bắt buộc phải có để Vbee mở được camera con).
* `--show-system-tray`: Hiển thị icon ứng dụng ở khay hệ thống (System Tray).
* `--width <px>` & `--height <px>`: Thiết lập kích thước cửa sổ mặc định (ví dụ: `--width 1200 --height 800`).

---

## 5. Các Lệnh Build Native Tauri Lõi (Cho nhà phát triển Pake)

Nếu bạn muốn build trực tiếp file mã nguồn Rust/Tauri của Pake (không qua CLI đóng gói URL):

```bash
# Chạy ứng dụng ở chế độ Development (Hot reload cả Rust và Frontend mặc định)
pnpm run dev

# Build ứng dụng cho nền tảng hiện tại (Windows -> exe, Mac -> app, Linux -> AppImage)
pnpm run build

# Build ứng dụng macOS hỗ trợ cả chip Intel và Apple Silicon (Universal Binary)
pnpm run build:mac

# Build ứng dụng dưới dạng Debug để kiểm tra log lỗi hệ thống
pnpm run build:debug
```

---

## 6. Biến Môi Trường Tối Ưu (Tùy chọn)

Nếu quá trình build bị chậm do tải tài nguyên từ máy chủ Rust/Cargo nước ngoài, bạn có thể gán biến môi trường sử dụng kho tải gương (Mirror) trước lệnh chạy:

* **Môi trường Bash (Mac, Linux, Git Bash Windows)**:
  ```bash
  PAKE_USE_CN_MIRROR=1 node dist/cli.js http://localhost:4201 --name CloudCamera --window live=/live --multi-window --show-system-tray
  ```
* **Môi trường PowerShell (Windows)**:
  ```powershell
  $env:PAKE_USE_CN_MIRROR=1; node dist/cli.js http://localhost:4201 --name CloudCamera --window live=/live --multi-window --show-system-tray
  ```
