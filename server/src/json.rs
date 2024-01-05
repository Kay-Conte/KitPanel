use foxhole::{http::Version, IntoResponse, Response};
use models::ToJson;

pub struct Json<T>(pub T);

impl<T> IntoResponse for Json<T>
where
    T: ToJson,
{
    fn response(self) -> foxhole::systems::RawResponse {
        let body = self.0.to_json().into_bytes();

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
