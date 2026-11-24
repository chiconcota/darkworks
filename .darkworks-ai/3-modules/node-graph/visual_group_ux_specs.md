# 📑 MA TRẬN & ĐẶC TẢ TÍNH NĂNG VISUAL GROUP (BACKDROP BOX) CHO DARKWORKS NLE

> **Mục tiêu:** Xây dựng hệ thống Visual Group (Khung phân vùng đồ thị) đạt chuẩn chuyên nghiệp của các phần mềm đồ thị hàng đầu thế giới (ComfyUI, DaVinci Resolve Fusion, Blender Shader Editor, Unreal Engine Blueprints), mang lại trải nghiệm biên tập video trực quan, gọn gàng và mượt mà nhất cho người dựng phim.

---

## 1. BẢNG MA TRẬN TÍNH NĂNG VISUAL GROUP (UX/UI FEATURE MATRIX)

Dưới đây là danh sách phân loại chi tiết các tính năng cần thiết cho Visual Group, được sắp xếp theo 4 trụ cột nghiệp vụ UX/UI và phân tầng mức độ ưu tiên:
- **P0 (Cốt lõi):** Tính năng thiết yếu phải có trong quy trình làm việc cơ bản hàng ngày.
- **P1 (Chuyên nghiệp):** Nâng cao hiệu suất biên tập cho Project quy mô lớn (20 - 100+ Nodes).
- **P2 (Nâng cao / Power-User):** Tự động hóa, tối ưu layout thông minh và A/B Testing.

| STT | Trụ Cột Nghiệp Vụ | Tên Tính Năng (Feature) | Mô Tả Trải Nghiệm Người Dùng (UX/UI Spec) | Mức Ưu Tiên | Trạng Thái Hiện Tại |
| :---: | :--- | :--- | :--- | :---: | :---: |
| **01** | **Tương tác Thao tác (Interaction)** | **Gom nhóm nhanh (`Ctrl + G`)** | Quét chọn nhiều node ➔ Bấm `Ctrl + G` tự động tính Bounding Box + 35px padding bao quanh các node với tên mặc định `Group {N}`. | **P0** | ✅ **Đã hoàn thành** |
| **02** | **Tương tác Thao tác (Interaction)** | **Kéo di chuyển đồng bộ (Header Drag)** | Nhấp giữ chuột vào Header bar để kéo cả Group và tất cả Node nằm bên trong di chuyển cùng lúc. | **P0** | ✅ **Đã hoàn thành** |
| **03** | **Tương tác Thao tác (Interaction)** | **Co giãn kích thước (Corner Resize)** | Kéo tay cầm góc dưới phải (`ResizeNwSe`) để mở rộng/thu hẹp khung Group bao thêm các Node. | **P0** | ✅ **Đã hoàn thành** |
| **04** | **Tương tác Thao tác (Interaction)** | **Rã nhóm (`Ctrl + Shift + G`)** | Phím tắt hủy Group: Xóa bỏ khung Group nhưng **giữ nguyên toàn bộ Node và dây nối**. | **P0** | ✅ **Đã hoàn thành** |
| **05** | **Tương tác Thao tác (Interaction)** | **Co giãn 4 cạnh (Edge Resizing)** | Rê chuột vào 4 viền (Trái, Phải, Trên, Dưới) hiện con trỏ chỉnh biên độ viền linh hoạt. | **P1** | ⏳ Đề xuất P1 |
| **06** | **Tương tác Thao tác (Interaction)** | **Căn vừa tự động (Auto-Fit to Nodes)** | Nhấp đúp vào tay cầm hoặc Menu "Fit" ➔ Khung Group tự động co/dãn vừa khít các Node bên trong. | **P1** | ✅ **Đã hoàn thành** |
| **07** | **Tương tác Thao tác (Interaction)** | **Di chuyển tách rời (Alt + Drag)** | Giữ phím `Alt` khi kéo Header ➔ Chỉ di chuyển khung viền Group mà không kéo theo các Node con. | **P1** | ⏳ Đề xuất P1 |
| **08** | **Tương tác Thao tác (Interaction)** | **Tự động bắt thành viên (Drop-in Auto-capture)** | Kéo thả 1 Node từ ngoài vào ranh giới Group ➔ Node tự động trở thành thành viên của Group. | **P2** | ⏳ Đề xuất P2 |
| **09** | **Nhận Diện Trực Quan (Visuals)** | **Đổi tên trực tiếp (Inline Rename)** | Nhấp đúp vào Header bar (hoặc bấm `F2`) để gõ đổi tên trực tiếp, bấm Enter để lưu, Esc để hủy. | **P0** | ✅ **Đã hoàn thành** |
| **10** | **Nhận Diện Trực Quan (Visuals)** | **Bảng màu xoay vòng & Bảng chọn màu** | Tự động sinh màu khi tạo; Menu chuột phải cho phép chọn trực tiếp 6 màu ComfyUI dịu mắt. | **P0** | ✅ **Đã hoàn thành** |
| **11** | **Nhận Diện Trực Quan (Visuals)** | **Cỡ chữ thích ứng Zoom (LOD Title)** | Khi zoom nhỏ (< 40%), chữ tiêu đề Group phóng to tương đối để người dùng dễ đọc tổng quan sơ đồ. | **P1** | ⏳ Đề xuất P1 |
| **12** | **Nhận Diện Trực Quan (Visuals)** | **Dòng ghi chú phụ (Description / Subtitle)** | Cho phép thêm 1 dòng ghi chú nhỏ bên dưới tiêu đề (Ví dụ: *"Khử ồn âm thanh + Đồng bộ Lip-sync"*). | **P1** | ⏳ Đề xuất P1 |
| **13** | **Nhận Diện Trực Quan (Visuals)** | **Khóa nhóm (Lock / Freeze Group)** | Nút icon 🔒 trên Header & Context Menu: Khóa vị trí, tránh bấm nhầm làm lệch bố cục sơ đồ. | **P1** | ✅ **Đã hoàn thành** |
| **14** | **Nhận Diện Trực Quan (Visuals)** | **Nhóm lồng nhau (Nested Groups)** | Hỗ trợ Group lớn bao quanh Group nhỏ (phân cấp Z-Index), kéo Group cha thì Group con di chuyển theo. | **P2** | ⏳ Đề xuất P2 |
| **15** | **Điều Khiển Quy Trình (Pipeline)** | **Bypass / Mute cả cụm (A/B Preview)** | Nút bật/tắt (Toggle 👁) trên Header: Tắt hoặc Bật tức thì toàn bộ hiệu ứng của cụm Node bên trong. | **P1** | ⏳ Đề xuất P2 |
| **16** | **Điều Khiển Quy Trình (Pipeline)** | **Thu gọn nhóm (Collapse to Compact Node)** | Bấm nút thu gọn ➔ Gom 10 Node thành 1 Node hộp đen nhỏ, chỉ giữ lại các chân In/Out nối ra ngoài. | **P2** | ⏳ Đề xuất P2 |
| **17** | **Điều Khiển Quy Trình (Pipeline)** | **Lưu thành Preset tái sử dụng** | Nhấp chuột phải chọn "Save as Preset" ➔ Lưu trọn vẹn cụm Node + Group thành file `.preset.json`. | **P1** | ⏳ Đề xuất P1 |
| **18** | **Quản Lý & Điều Hướng (Navigation)** | **Focus vào Group (Outliner Click)** | Bấm vào tên Group trong bảng quản lý ➔ Viewport tự động Pan/Zoom mượt mà căn giữa Group đó. | **P1** | ⏳ Đề xuất P1 |
| **19** | **Quản Lý & Điều Hướng (Navigation)** | **Menu chuột phải đầy đủ (Context Menu)** | Chuột phải vào Header hiện menu: Đổi tên, Đổi màu, Auto-fit, Khóa vị trí, Rã nhóm, Xóa. | **P0** | ✅ **Đã hoàn thành** |
| **20** | **Quản Lý & Điều Hướng (Navigation)** | **Hộp thoại quản lý nhóm an toàn** | Hộp thoại quản lý nhóm: Cho phép xóa khung giữ node ("Rã nhóm") hoặc xóa sạch cụm. | **P0** | ✅ **Đã hoàn thành** |

---

## 2. BẢNG SO SÁNH VỚI CÁC PHẦN MỀM ĐỒ THỊ CHUYÊN NGHIỆP

| Tính Năng (Capability) | Darkworks (Hiện Tại) | ComfyUI | DaVinci Resolve (Underlay) | Blender (Frame Node) | Unreal Engine (Comment Box) |
| :--- | :---: | :---: | :---: | :---: | :---: |
| **Phím tắt gom nhanh** | `Ctrl + G` | Menu / Canvas | `Alt + Underlay` | `Ctrl + J` | `C` |
| **Kéo Header mang theo Node** | ✅ Có | ✅ Có | ✅ Có | ✅ Có | ✅ Có |
| **Đổi màu nền (Color Coding)** | ✅ Tự động xoay vòng | ✅ Bảng 10 màu | ✅ Bảng màu Resolve | ✅ Tự chọn mã màu | ✅ Full RGB Picker |
| **Thu phóng kích thước** | ✅ Kéo góc dưới phải | ✅ Kéo góc | ✅ Kéo 4 góc | ✅ Tự động co theo node | ✅ Kéo 4 góc |
| **Sửa tên trực tiếp trên Header** | ⏳ Qua Popup Dialog | ✅ Inline Text | ✅ Inline Text | ✅ F2 / Sidebar | ✅ Inline Text |
| **Khóa vị trí (Lock Mode)** | ⏳ Chưa có | ❌ Không | ❌ Không | ❌ Không | ✅ Có |
| **Bypass / Mute toàn bộ Node** | ⏳ Chưa có | ✅ Mute Group | ❌ Phải chọn từng node | ✅ Mute Node | ❌ Không |
| **Thu gọn thành Node con** | ⏳ (Có Black Node) | ❌ Không | ❌ Không | ✅ Group Node (`Ctrl + G`) | ✅ Collapse Nodes |
| **Tự động co vừa khít (Auto-Fit)** | ⏳ Chưa có | ❌ Không | ❌ Không | ✅ Tự co giãn | ❌ Không |

---

## 3. CHI TIẾT TRẢI NGHIỆM NGƯỜI DÙNG ĐỀ XUẤT (DETAILED UX/UI DESIGN)

### 3.1. Giao diện Thanh Header Group (Header Bar Layout)
Thanh Header của Group được thiết kế với chiều cao chuẩn **30px**, nền bán trong suốt đậm hơn thân Group:
```text
+-------------------------------------------------------------------------------+
| [📁 Icon] [Tiêu đề Group (Double click sửa)]     [🔒] [👁 Bypass] [🎨] [✖ Xóa] |
+-------------------------------------------------------------------------------+
|                                                                               |
|       [ Node A ]  ══════>  [ Node B ]  ══════>  [ Node C ]                    |
|                                                                               |
|                                                                      [◢ Grip] |
+-------------------------------------------------------------------------------+
```

- **Phía bên trái:**
  - Icon thư mục `📁` nhận diện nhóm.
  - Tiêu đề nhóm in đậm (`RichText::strong()`), nhấp đúp để đổi tên trực tiếp (Inline Edit).
- **Phía bên phải (Cụm điều khiển mini):**
  - Icon `🎨`: Nhấp để mở bảng 6 màu nhanh (Quick Palette Swatches).
  - Icon `👁`: Bật/Tắt bypass toàn bộ hiệu ứng của nhóm (rất hữu ích cho người dựng color grade / visual FX).
  - Icon `🔒`: Khóa nhóm, ngăn di chuyển hoặc thay đổi ngoài ý muốn.
  - Icon `✖`: Xóa nhóm (hiển thị hộp thoại hỏi xóa khung hay xóa toàn bộ).

### 3.2. Menu Chuột Phải Cho Group (Group Context Menu)
Khi người dùng nhấp chuột phải vào bất kỳ vị trí nào trên thanh Header của Group:
- `✏️ Đổi tên nhóm (Rename)`: Bật chế độ sửa tên trực tiếp.
- `🎨 Đổi màu sắc (Change Color)`: Mở bảng màu nhanh.
- `🎯 Căn vừa Node con (Fit to Content)`: Tự động tính lại bounding box bao khít các node hiện có.
- `📑 Nhân bản nhóm (Duplicate Group)`: Tạo bản sao của cả Group và toàn bộ Node con sang bên cạnh.
- `🔒 Khóa vị trí (Lock Position)`: Bật/Tắt trạng thái cố định.
- `💾 Lưu thành Preset (Export Preset)`: Xuất file `.preset.json` tái sử dụng.
- `🔓 Rã nhóm (Ungroup - Giữ Node)`: Hủy khung nhóm, giữ nguyên Node.
- `🗑 Xóa toàn bộ (Delete Group & Nodes)`: Xóa sạch cả khung và Node bên trong.

### 3.3. Bộ Phím Tắt Tiêu Chuẩn (Standard Hotkeys)
- **`Ctrl + G` (hoặc `Cmd + G`):** Nhóm các Node đang chọn vào Visual Group mới.
- **`Ctrl + Shift + G`:** Rã nhóm (Ungroup) đang chọn (xóa khung, giữ Node).
- **`F2` (khi chọn Group):** Bắt đầu đổi tên nhóm ngay lập tức.
- **`Alt + Kéo chuột trên Header`:** Di chuyển chỉ khung viền Group mà không di chuyển Node bên trong (hữu ích khi muốn căn lại khung bao).

---

## 4. LỘ TRÌNH ĐỀ XUẤT TRIỂN KHAI (IMPLEMENTATION ROADMAP)

Để đảm bảo tuân thủ thiết quân luật mã nguồn của Darkworks (**File < 700 dòng**, **Never Crash**, **RAM < 2GB**), các tính năng trên nên được chia làm 3 đợt:

### Đợt 1: Hoàn thiện Tương tác Cốt lõi (Next Phase Polish)
1. **Inline Edit tên nhóm:** Nhấp đúp vào Header bar để sửa tên trực tiếp như Node (Enter lưu, Esc hủy).
2. **Menu chuột phải trên Group:** Tích hợp `✏️ Đổi tên`, `🎨 Đổi màu`, `🔓 Rã nhóm`, `🗑 Xóa`.
3. **Phím tắt `Ctrl + Shift + G`:** Rã nhóm nhanh.
4. **Auto-Fit:** Nhấp đúp vào viền để tự động co vừa các Node con.

### Đợt 2: Điều khiển Workflow & Thẩm Mỹ (Professional Suite)
1. **Nút Khóa `🔒`:** Chống kéo nhầm sơ đồ.
2. **Nút Bypass `👁`:** Tắt/Bật toàn bộ cụm Node trong Group.
3. **Cỡ chữ thích ứng Zoom (LOD Zoom Text):** Hiển thị rõ tiêu đề ở mọi mức Zoom xa.
4. **Phân vùng lồng nhau (Nested Groups):** Nhóm cha chứa nhóm con.

### Đợt 3: Đóng Gói Cấp Cao (Compound & Subgraph)
1. **Thu gọn thành Compound Node (Black Box):** Đóng gói Group thành 1 Node duy nhất trên Canvas chính.
2. **Xuất/Nạp Preset trực tiếp từ Group:** 1 cú nhấp chuột lưu thành công cụ tái sử dụng trong Source Bin.
