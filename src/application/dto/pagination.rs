use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PageParams {
    pub page: u32,
    pub limit: u32,
}

impl Default for PageParams {
    fn default() -> Self {
        Self { page: 1, limit: 20 }
    }
}

impl PageParams {
    pub fn offset(&self) -> u32 {
        (self.page.saturating_sub(1)) * self.limit
    }

    pub fn clamped_limit(&self) -> u32 {
        self.limit.min(100)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PageMeta {
    pub total: u64,
    pub page: u32,
    pub limit: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Paginated<T> {
    pub data: Vec<T>,
    pub meta: PageMeta,
}

impl<T> Paginated<T> {
    pub fn new(data: Vec<T>, total: u64, params: &PageParams) -> Self {
        Self {
            data,
            meta: PageMeta {
                total,
                page: params.page,
                limit: params.clamped_limit(),
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_page_params() {
        let p = PageParams::default();
        assert_eq!(p.page, 1);
        assert_eq!(p.limit, 20);
    }

    #[test]
    fn offset_first_page_is_zero() {
        assert_eq!(PageParams { page: 1, limit: 20 }.offset(), 0);
    }

    #[test]
    fn offset_second_page() {
        assert_eq!(PageParams { page: 2, limit: 20 }.offset(), 20);
    }

    #[test]
    fn offset_page_zero_saturates_to_zero() {
        assert_eq!(PageParams { page: 0, limit: 10 }.offset(), 0);
    }

    #[test]
    fn clamped_limit_under_100_is_unchanged() {
        assert_eq!(PageParams { page: 1, limit: 50 }.clamped_limit(), 50);
    }

    #[test]
    fn clamped_limit_caps_at_100() {
        assert_eq!(PageParams { page: 1, limit: 200 }.clamped_limit(), 100);
    }

    #[test]
    fn paginated_new_sets_meta() {
        let params = PageParams { page: 3, limit: 10 };
        let p: Paginated<i32> = Paginated::new(vec![1, 2, 3], 50, &params);
        assert_eq!(p.data, vec![1, 2, 3]);
        assert_eq!(p.meta.total, 50);
        assert_eq!(p.meta.page, 3);
        assert_eq!(p.meta.limit, 10);
    }

    #[test]
    fn paginated_new_clamps_limit_in_meta() {
        let params = PageParams { page: 1, limit: 500 };
        let p: Paginated<()> = Paginated::new(vec![], 0, &params);
        assert_eq!(p.meta.limit, 100);
    }
}
