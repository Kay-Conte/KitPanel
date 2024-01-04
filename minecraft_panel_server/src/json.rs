use foxhole::{IntoResponse, Response, http::Version};
use serde::Serialize;

#[derive(Serialize)]
pub struct Json<T>(pub T);

impl<T> IntoResponse for Json<T>
where
    T: Serialize,
{
    fn response(self) -> foxhole::systems::RawResponse {
        let body = serde_json::to_string(&self.0)
            .expect("Failed to serialize response objecct")
            .into_bytes();

        let size = body.len();

        Response::builder()
            .version(Version::HTTP_11)
            .status(200)
            .header("content-type", "text/json")
            .header("content-length", format!("{}", size))
            .body(body)
            .expect("Failed to convert object to response")
    }
}
