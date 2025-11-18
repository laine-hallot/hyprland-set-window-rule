#[derive(Debug, Clone)]
pub struct TotallyNotClientRegion {
    pub at: (i16, i16),
    pub size: (i16, i16),
    pub monitor: Option<String>,
}
