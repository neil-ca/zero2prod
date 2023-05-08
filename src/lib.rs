pub mod configuration;
pub mod domain;
pub mod email_client;
pub mod routes;
pub mod startup;
pub mod telemetry;
//pub enum Payload<S = Pin<Box<dyn Stream<Item = Result<Bytes, PayloadError>> + 'static, Global>>> {
//None,
//H1 {
//payload: Payload,
//},
//H2 {
//payload: Payload,
//},
//Stream {
//payload: S,
//},
//}
