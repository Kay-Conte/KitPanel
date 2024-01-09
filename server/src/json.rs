use foxhole::{http::Version, IntoResponse, Response, action::RawResponse, Action, resolve::{Resolve, ResolveGuard}};
use models::{FromJson, ToJson};

pub struct Json<T>(pub T);

impl<T> IntoResponse for Json<T>
where
    T: ToJson,
{
    fn response(self) -> RawResponse {
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

impl<'a, T> Resolve<'a> for Json<T>
where
    T: 'a + FromJson,
{
    type Output = Json<T>;

    fn resolve(
        ctx: &'a foxhole::RequestState,
        _path_iter: &mut foxhole::PathIter,
    ) -> ResolveGuard<Self::Output> {
        let Ok(body) = String::from_utf8(ctx.request.body().get().to_vec()) else {
            return ResolveGuard::None;
        };

        T::from_json(body).map(|t| Json(t)).into()

    }
}
