mod transform;

use futures_util::future::BoxFuture;
use opentelemetry_sdk::{
    error::OTelSdkResult,
    trace::{SpanData, SpanExporter},
};

use fluvio::{Fluvio, TopicProducerPool};

pub struct FluvioExporter {
    fluvio_topic: String,
    fluvio_producer: TopicProducerPool,
}

impl std::fmt::Debug for FluvioExporter {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("FluvioExporter")
            .field("fluvio_topic", &self.fluvio_topic)
            .finish()
    }
}

impl FluvioExporter {
    pub async fn create_async(topic: &str) -> anyhow::Result<Self> {
        let fluvio = Fluvio::connect().await?;
        let fluvio_producer = fluvio.topic_producer(topic).await?;

        Ok(Self {
            fluvio_topic: topic.to_owned(),
            fluvio_producer,
        })
    }
    pub fn create(topic: &str) -> anyhow::Result<Self> {
        fluvio_future::task::run_block_on(async { Self::create_async(topic).await })
    }
}

impl SpanExporter for FluvioExporter {
    fn export(&mut self, _spans: Vec<SpanData>) -> BoxFuture<'static, OTelSdkResult> {
        use prost::Message;
        use transform::SpanDataProto;

        let mut buf = Vec::new();
        for span in _spans {
            let span_proto = SpanDataProto::from(span);
            span_proto.encode(&mut buf).unwrap();
            self.fluvio_producer
                .send(self.fluvio_topic.clone(), buf.clone())
                .await
                .unwrap();
            buf.clear();
        }
        Box::pin(futures_util::future::ready(Ok(())))
    }
}
