# QUY TRÌNH KIỂM THỬ ĐẦU-CUỐI TÍNH NĂNG NODE GRAPH (E2E WORKFLOW)
@project: DARKWORKS | @feature: DAG Node Graph Engine (Phase 3)
@target: Hướng dẫn người dùng tự kiểm thử từng bước trực tiếp trên giao diện Desktop GUI

Tài liệu này cung cấp kịch bản kiểm thử End-to-End (E2E) chi tiết từ A đến Z để bạn có thể tự mình khởi động Darkworks, mở phân vùng **Node Graph (DAG)** và lần lượt kiểm tra toàn bộ 18 Versatile Atomic Nodes, kho 5 Presets mẫu, bộ công cụ ComfyUI, và cơ chế phát hiện lỗi chu trình kín (DAG Cycle Detection).

---

## 🚀 CHUẨN BỊ & KHỞI ĐỘNG ỨNG DỤNG

Mở Terminal tại thư mục gốc của dự án (`/mnt/Data/darkworks`) và chạy lệnh khởi chạy ứng dụng Darkworks Desktop:

```bash
cargo run --bin darkworks
```
*(Nếu muốn tốc độ dựng khung hình mượt mà tối đa 60 FPS, bạn có thể chạy: `cargo run --release --bin darkworks`)*

---

## 📋 CÁC BƯỚC TỰ KIỂM THỬ TRỰC QUAN TRÊN GIAO DIỆN (STEP-BY-STEP)

### BƯỚC 1: MỞ PHÂN VÙNG GIAO DIỆN NODE GRAPH PANEL
1. **Tìm tab Node Graph:**
   - Quan sát góc trên bên trái của cửa sổ ứng dụng (khu vực nằm ngang hàng với tab `Source`, `Effect Controls`, `Audio Clip Mixer`).
   - Nhấp chuột trái vào tab **`Node Graph`**.
2. **Nếu chưa thấy tab Node Graph trên màn hình:**
   - Vào thanh Menu Bar trên cùng: chọn **`Window`** ➔ **`Workspaces`** ➔ **`All Panels`** (hoặc chọn trực tiếp **`Window`** ➔ **`Node Graph`**).
3. **Kết quả kỳ vọng:**
   - Phân vùng Node Graph hiển thị với lưới nền caro xám chuẩn công nghiệp (**Dot-Grid Canvas**).
   - Thanh Toolbar trên cùng hiển thị tiêu đề màu xanh Cyan Neon: **`🌟 Node Graph (DAG)`** kèm các nút chức năng: `➕ Add Node`, `📂 Load Preset`, `🟩 Add Visual Group`, `💾 Export Preset`, `⚡ Validate DAG`, `🔄 Reset`, `🗑 Clear All`.
   - Mặc định khởi tạo sẵn Preset mẫu **News Ticker Banner** với các node đã được nối dây và đóng khung Backdrop Group.

---

### BƯỚC 2: TEST NẠP 1-CLICK KHO 5 BUILT-IN PRESETS MẪU
Trên thanh Toolbar, nhấp chuột vào menu **`📂 Load Preset`** và lần lượt bấm kiểm tra 5 presets sau:

- [ ] **1. Preset `1. 📰 News Ticker Banner`:**
  - Nhấp chọn `1. 📰 News Ticker Banner`.
  - **Quan sát canvas:** Xuất hiện cụm node gồm `Shape Generator` (RoundedRectangle bo góc 10px làm nền xanh), `Text Generator` (tiêu đề chạy chữ), `Time Driver` (lái cuộn ngang theo thời gian), `Blend` (hòa trộn Normal), và `Final Output`.
  - Có khung **Visual Group** màu tím nhạt bao bọc quanh cụm node.
  - Bấm nút **`⚡ Validate DAG`** trên toolbar ➔ Nhìn sang góc phải toolbar: Báo dòng chữ xanh: `✔ DAG Hợp lệ (4 bước tuần tự, 2 tầng song song)`.

- [ ] **2. Preset `2. 🌊 Continuous Audio Waveform Line (60 FPS)`:**
  - Nhấp chọn `2. 🌊 Audio Waveform Line (60 FPS)`.
  - **Quan sát canvas:** Xuất hiện `Shape Generator` (Line 4px), `Distort / Lens` (chế độ RippleWave), `Audio Reactor` (lấy dải tần Bass), `Time Driver`, và `Blend` (chế độ Screen).
  - Bấm **`⚡ Validate DAG`** ➔ Xác nhận đồ thị hợp lệ, không có chu trình.

- [ ] **3. Preset `3. 🎯 Spotlight Invert`:**
  - Nhấp chọn `3. 🎯 Spotlight Invert`.
  - **Quan sát canvas:** Xuất hiện `Video Input (V1)`, `Shape Generator` (Circle 150px mask), `Color Grade` (bật cờ Invert Colors), và `Blend` (Difference mode).
  - Bấm **`⚡ Validate DAG`** ➔ Xác nhận đồ thị hợp lệ.

- [ ] **4. Preset `4. ⚡ Audio Beat Invert Flash`:**
  - Nhấp chọn `4. ⚡ Audio Beat Invert Flash`.
  - **Quan sát canvas:** Xuất hiện `Video Input`, `Shape Generator` (Solid Matte), `Audio Reactor` (Bass beat), và `Blend` (Difference mode).

- [ ] **5. Preset `5. ✨ Cinematic Soft Glow`:**
  - Nhấp chọn `5. ✨ Cinematic Soft Glow`.
  - **Quan sát canvas:** Xuất hiện `Video Input`, `Blur / Sharpen` (chế độ BokehBloom, radius 15.0), và `Blend` (chế độ Screen bloom).

---

### BƯỚC 3: TEST DỌN SẠCH CANVAS & THÊM NODE MỚI (18 ATOMIC NODES)
1. Bấm nút **`🗑 Clear All`** trên toolbar để xóa sạch canvas. Thanh trạng thái báo: `Đã dọn sạch đồ thị`.
2. Mở menu **`➕ Add Node`** và thử thêm lần lượt các Node đại diện từ 6 phân khu:
   - **Phân khu 1 (Nguồn phát - Generators):**
     - Bấm chọn `🎬 Video Input` ➔ Node xuất hiện trên canvas với cổng ra `Output (Texture)`.
     - Bấm chọn `🔤 Text Generator` ➔ Node hiển thị ô nhập Text, slider font size, tracking.
     - Bấm chọn `🟩 Shape / Solid Matte` ➔ Node hiển thị chọn shape (Rectangle, Circle, Line).
     - Bấm chọn `⏱️ Time Driver` ➔ Node đếm thời gian với tốc độ speed và mode (Linear, SineLoop, PingPong).
   - **Phân khu 2 (Chỉnh màu & Quang học):**
     - Bấm chọn `🎨 Color Grade (Invert/B&W)` ➔ Node có các slider Exposure, Contrast, Saturation, và 2 checkbox: `[✓] Invert Colors`, `[✓] Black & White`.
     - Bấm chọn `💧 Blur / Sharpen` ➔ Node có dropdown chọn 5 loại blur (Gaussian, Box, Directional, BokehBloom, UnsharpMask) và slider radius.
     - Bấm chọn `🌀 Distort / Lens` ➔ Dropdown 6 kiểu biến dạng (RippleWave, Vignette, FishEye, LensDistortion, ChromaticAberration).
     - Bấm chọn `🟢 Ultra Keyer` ➔ Bộ tách phông xanh kèm thanh trượt Similarity và Spill Suppress.
   - **Phân khu 3 (Biến hình & Hòa trộn):**
     - Bấm chọn `📐 Transform 2D (Flip H/V)` ➔ Sliders Scale, Position, Rotation, Anchor, và 2 checkbox: `[Flip Horizontal]`, `[Flip Vertical]`.
     - Bấm chọn `✂️ Crop / Mask` ➔ Sliders Top, Bottom, Left, Right và Feather làm mềm viền.
     - Bấm chọn `🎭 Blend / Mix (26 Modes)` ➔ Dropdown chọn 26 chế độ hòa trộn (Normal, Multiply, Screen, Overlay, SoftLight, Difference...) và 3 cổng cắm: `Background`, `Foreground`, `Mask`.
   - **Phân khu 4 (Chuyển cảnh):**
     - Bấm chọn `⚡ Transition` ➔ Dropdown 30 kiểu chuyển cảnh và thanh kéo Progress.
   - **Phân khu 5 (Tín hiệu & Tự động hóa):**
     - Bấm chọn `🔊 Audio Reactor` ➔ Dropdown dải tần (Bass, Mid, High, Full) và multiplier.
     - Bấm chọn `⚡ Action Node` ➔ Có nút bấm `[ ▶ RUN WORKFLOW ]` để kích hoạt tự động hóa.
   - **Phân khu 6 (Mở rộng & ComfyUI):**
     - Bấm chọn `⬛ Black Box (Compound)` ➔ Hộp đen đóng gói subgraph, bộc lộ Exposed Pins.
     - Bấm chọn `💻 Custom WGSL Shader` ➔ Ô text editor nhúng code WGSL trực tiếp.
     - Bấm chọn `📝 Sticky Note` ➔ Thẻ ghi chú màu vàng dán trên canvas.
     - Bấm chọn `● Reroute Dot` ➔ Chấm tròn nhỏ đổi hướng dây nối 90 độ.
     - Bấm chọn `🖥️ Final Output` ➔ Cổng xuất texture cuối cùng ra Viewport.

---

### BƯỚC 4: TEST THAO TÁC NỐI DÂY & PHÁT HIỆN LỖI VÒNG LẶP (DAG CYCLE PROOF)
1. **Nối dây hợp lệ (Valid Routing):**
   - Đặt con trỏ chuột vào socket tròn màu xanh lam **`Output (Texture)`** của node `Video Input`.
   - Nhấn giữ chuột trái và kéo dây cong Bézier sang socket **`Input`** của node `Color Grade`.
   - Tiếp tục kéo từ socket **`Output`** của `Color Grade` vào socket **`Foreground`** của `Blend`.
   - Kéo từ `Blend` vào socket **`Input`** của `Final Output`.
   - Bấm nút **`⚡ Validate DAG`** trên toolbar ➔ Thanh trạng thái báo: **`✔ DAG Hợp lệ`**.
2. **Thử nghiệm tạo vòng lặp vô tận (Cycle Trap Test):**
   - Thử kéo một đường dây từ `Output` của `Blend` cắm ngược trở lại vào `Input` của `Color Grade` (tạo chu trình kín A ➔ B ➔ A).
   - Bấm nút **`⚡ Validate DAG`** trên toolbar.
   - **Kết quả kỳ vọng:** Hệ thống phát hiện ngay lập tức và từ chối xử lý, thanh trạng thái hiển thị thông báo đỏ:  
     `✖ Lỗi phân tích đồ thị: Phát hiện chu trình kín (Cycle Detected) trong DAG`.
3. **Tháo dây (Disconnect):**
   - Kéo đầu dây nối ra khỏi socket và thả ra khoảng trống trên canvas để ngắt kết nối.
   - Bấm lại **`⚡ Validate DAG`** ➔ Trở lại trạng thái hợp lệ.

---

### BƯỚC 5: TEST BỘ CÔNG CỤ TỔ CHỨC CANVAS KIỂU COMFYUI
- [ ] **Test Visual Group (Backdrop Box):**
  - Bấm nút **`🟩 Add Visual Group`** trên toolbar.
  - Một khung hình chữ nhật trong suốt màu tím xuất hiện với tiêu đề `GROUP 1`.
  - Di chuyển các node đặt lọt vào bên trong khung `GROUP 1`.
  - Nhấp giữ chuột vào thanh tiêu đề của `GROUP 1` và kéo đi ➔ Toàn bộ các node con nằm trong khung di chuyển đồng bộ theo (rất tiện lợi khi quản lý đồ thị lớn).
- [ ] **Test Sticky Note (Ghi chú):**
  - Nhấp đúp vào ô nội dung của `Sticky Note`, gõ chữ: `"Hiệu ứng chỉnh màu Cyberpunk Neon"`.
  - Chữ hiển thị rõ nét trên canvas mà không làm ảnh hưởng luồng render.
- [ ] **Test Reroute Dot (Bẻ góc dây nối):**
  - Đặt `Reroute Dot` ở giữa 2 node đặt xa nhau.
  - Nối dây qua Reroute Dot để đường dây bẻ góc ngay ngắn, không bị cắt ngang qua thân node khác.

---

### BƯỚC 6: TEST XUẤT PRESET RA CLIPBOARD & KHÔI PHỤC
1. Sau khi chỉnh sửa các thông số và sắp xếp đồ thị theo ý bạn, bấm nút **`💾 Export Preset`** trên toolbar.
2. Quan sát thanh trạng thái: Hiển thị dòng chữ: **`Đã copy Preset JSON vào Clipboard!`**.
3. Mở một ứng dụng soạn thảo văn bản (như VS Code, Gedit hoặc Kate) và nhấn `Ctrl + V`.
4. **Kiểm tra JSON:** Dữ liệu xuất ra chứa cấu trúc JSON chuẩn gồm tên preset, danh sách nodes với đầy đủ tham số đã chỉnh, và thông tin các Visual Groups.
5. Bấm nút **`🔄 Reset`** trên toolbar ➔ Ứng dụng khôi phục đồ thị về Preset News Ticker mặc định ban đầu.

---

## 🤖 KIỂM THỬ TỰ ĐỘNG BẰNG LỆNH (AUTOMATED HARNESS COMMANDS)

Ngoài việc tự tay thao tác trên giao diện, bạn có thể chạy các lệnh kiểm tra tự động độc lập qua Terminal bất kỳ lúc nào:

1. **Chạy 11 bài kiểm thử tự động của Crate Node Graph:**
   ```bash
   cargo test -p darkworks-node-graph
   ```
   *(Xác nhận toàn bộ 11/11 tests pass: 18 atomic nodes, color invert, transform flips, 5 presets, cycle detection, topological sort, JIT WGSL parser).*

2. **Chạy kiểm toán kích thước file (< 700 dòng):**
   ```bash
   python3 .agents/harness/audit.py
   ```
   *(Xác nhận toàn bộ 10/10 file mã nguồn Darkworks mới đều đạt chuẩn xanh < 700 dòng).*

3. **Chạy bộ lái nghiệm thu tính năng:**
   ```bash
   python3 .agents/harness/drive_feature.py -f all
   ```
