use futures_util::future::BoxFuture;
use opentelemetry_sdk::{
    error::OTelSdkResult,
    trace::{SpanData, SpanExporter},
};

#[derive(Debug, Clone)]
pub struct FluvioExporter;

impl SpanExporter for FluvioExporter {
    fn export(&mut self, _spans: Vec<SpanData>) -> BoxFuture<'static, OTelSdkResult> {
        Box::pin(futures_util::future::ready(Ok(())))
    }
}
