pub mod accumulated_image;
pub mod csv;
pub mod metadata;
pub mod time_frames;
pub type OutputResult<T> = Result<T, Box<dyn std::error::Error>>;
