use std::collections::{HashMap, VecDeque};
use std::hash::Hash;
use std::sync::Arc;
use parking_lot::Mutex;
use crate::frame::VideoFrame;

/// Cấu hình trần bộ nhớ mặc định cho Ring-Buffer Frame Cache (Khẩu quyết tối thượng #1)
/// Mức 1: 1 GB (1024 MB) - Hạn mức tiêu chuẩn khi nạp và phát video dài
pub const DEFAULT_CACHE_BUDGET_BYTES: usize = 1024 * 1024 * 1024; // 1 GB (1024 MB) Mức 1
/// Mức Trần Tối Thượng: 2 GB (2048 MB) - Trần cứng tuyệt đối của toàn bộ ứng dụng Darkworks
pub const MAX_CACHE_BUDGET_BYTES: usize = 2 * 1024 * 1024 * 1024; // 2 GB (2048 MB) Mức Trần

#[derive(Debug, Clone, Copy, Default)]
pub struct CacheStats {
    pub bytes: usize,
    pub budget: usize,
    pub entry_count: usize,
    pub hits: u64,
    pub misses: u64,
}

struct CacheEntry {
    frame: Arc<VideoFrame>,
    bytes: usize,
    last_access: u64,
}

struct Inner<K> {
    map: HashMap<K, CacheEntry>,
    order: VecDeque<(K, u64)>,
    bytes: usize,
    clock: u64,
    hits: u64,
    misses: u64,
}

/// Bộ đệm vòng Ring-Buffer LRU Cache giới hạn cứng theo ngân sách Byte (Byte-Budgeted)
/// Đảm bảo không bao giờ làm tràn RAM hệ thống quá trần cho phép và dọn dẹp mượt mà O(1).
pub struct FrameCache<K: Eq + Hash + Clone = (String, i64)> {
    inner: Mutex<Inner<K>>,
    budget: Mutex<usize>,
}

impl<K: Eq + Hash + Clone> FrameCache<K> {
    /// Khởi tạo Cache với hạn mức dung lượng RAM (bytes)
    pub fn new(budget_bytes: usize) -> Self {
        let budget = budget_bytes.min(MAX_CACHE_BUDGET_BYTES);
        Self {
            inner: Mutex::new(Inner {
                map: HashMap::new(),
                order: VecDeque::new(),
                bytes: 0,
                clock: 0,
                hits: 0,
                misses: 0,
            }),
            budget: Mutex::new(budget),
        }
    }

    /// Lấy khung hình từ cache nếu có sẵn
    pub fn get(&self, key: &K) -> Option<Arc<VideoFrame>> {
        let mut g = self.inner.lock();
        g.clock = g.clock.wrapping_add(1);
        let now = g.clock;

        if let Some(entry) = g.map.get_mut(key) {
            entry.last_access = now;
            let frame = Arc::clone(&entry.frame);
            g.hits += 1;
            g.order.push_back((key.clone(), now));
            Some(frame)
        } else {
            g.misses += 1;
            None
        }
    }

    /// Đưa khung hình vào cache và tự động giải phóng êm ái các frame cũ nhất theo O(1)
    pub fn insert(&self, key: K, frame: Arc<VideoFrame>) {
        let frame_bytes = frame.byte_size();
        let budget = *self.budget.lock();

        let mut to_drop = Vec::new();
        {
            let mut g = self.inner.lock();
            g.clock = g.clock.wrapping_add(1);
            let now = g.clock;

            if let Some(old) = g.map.insert(
                key.clone(),
                CacheEntry {
                    frame,
                    bytes: frame_bytes,
                    last_access: now,
                },
            ) {
                g.bytes = g.bytes.saturating_sub(old.bytes);
                to_drop.push(old.frame);
            }
            g.bytes = g.bytes.saturating_add(frame_bytes);
            g.order.push_back((key, now));

            // Thuật toán LRU Eviction mượt mà O(1):
            // Giải phóng vi mô từng frame xa nhất trong quá khứ khi vượt budget
            while g.bytes > budget && !g.map.is_empty() {
                if let Some((victim_key, stamp)) = g.order.pop_front() {
                    if let Some(entry) = g.map.get(&victim_key) {
                        if entry.last_access == stamp {
                            if let Some(removed) = g.map.remove(&victim_key) {
                                g.bytes = g.bytes.saturating_sub(removed.bytes);
                                to_drop.push(removed.frame);
                            }
                        }
                    }
                } else {
                    break;
                }
            }

            // Tự động dọn dẹp hàng đợi order định kỳ nếu có quá nhiều entry cũ
            if g.order.len() > g.map.len().saturating_mul(4).max(1024) {
                let Inner { map, order, .. } = &mut *g;
                order.retain(|(k, stamp)| {
                    map.get(k).map(|e| e.last_access == *stamp).unwrap_or(false)
                });
            }
        } // Nhả Mutex ngay lập tức tại đây!

        // Giải phóng bộ nhớ RAM các frame cũ hoàn toàn NGOÀI khóa Mutex (Zero-locking latency)
        drop(to_drop);
    }

    /// Xóa một entry cụ thể khỏi cache
    pub fn remove(&self, key: &K) {
        let mut g = self.inner.lock();
        if let Some(entry) = g.map.remove(key) {
            g.bytes = g.bytes.saturating_sub(entry.bytes);
        }
    }

    /// Xóa toàn bộ cache
    pub fn clear(&self) {
        let mut g = self.inner.lock();
        g.map.clear();
        g.order.clear();
        g.bytes = 0;
    }

    /// Số byte bộ nhớ đang chiếm giữ
    pub fn bytes(&self) -> usize {
        self.inner.lock().bytes
    }

    /// Hạn mức RAM budget hiện tại
    pub fn budget(&self) -> usize {
        *self.budget.lock()
    }

    /// Lấy thống kê hiệu năng cache
    pub fn stats(&self) -> CacheStats {
        let g = self.inner.lock();
        let budget = *self.budget.lock();
        CacheStats {
            bytes: g.bytes,
            budget,
            entry_count: g.map.len(),
            hits: g.hits,
            misses: g.misses,
        }
    }

    /// Thay đổi hạn mức RAM budget (tối đa không quá trần 2GB)
    pub fn set_budget(&self, new_budget: usize) {
        let clamped = new_budget.min(MAX_CACHE_BUDGET_BYTES);
        *self.budget.lock() = clamped;
    }
}
