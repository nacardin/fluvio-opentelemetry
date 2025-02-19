use opentelemetry::trace::TracerProvider as _;
use opentelemetry_sdk::trace::SdkTracerProvider;
use tracing::{error, info, span};
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::Registry;

use fluvio_opentelemetry::FluvioExporter;

fn main() {
    let span_exporter = FluvioExporter;

    // Create a new OpenTelemetry trace pipeline that prints to stdout
    let provider = SdkTracerProvider::builder()
        .with_simple_exporter(span_exporter.clone())
        .build();
    let tracer = provider.tracer("readme_example");

    // Create a tracing layer with the configured tracer
    let telemetry = tracing_opentelemetry::layer().with_tracer(tracer);

    // Use the tracing subscriber `Registry`, or any other subscriber
    // that impls `LookupSpan`
    let subscriber = Registry::default().with(telemetry);

    // Trace executed code
    tracing::subscriber::with_default(subscriber, || {
        // Spans will be sent to the configured OpenTelemetry exporter
        let root = span!(tracing::Level::TRACE, "app_start", work_units = 2);
        let _enter = root.enter();

        error!("This event will be logged in the root span.");

        info!(attr1 = "value1", attr2 = "value2", "event1")
    });
}
