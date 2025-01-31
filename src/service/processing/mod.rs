pub mod processing_service;
pub mod local_nlp;
pub mod pattern_match_processor;

pub use local_nlp::LocalProcessor;
pub use pattern_match_processor::PatternMatchProcessor;
pub use processing_service::ProcessingService;