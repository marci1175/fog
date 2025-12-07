use common::{
    anyhow,
    dependency::DependencyRequest,
    reqwest::{Client, Response, header::USER_AGENT},
    serde_json,
};

pub async fn request_dependency(
    client: Client,
    remote_url: &str,
    dep_name: String,
    version: String,
) -> anyhow::Result<Response>
{
    Ok(client
        .get(remote_url)
        .body(serde_json::to_string(&DependencyRequest {
            name: dep_name,
            version,
        })?)
        .header(USER_AGENT, format!("FDCN({})", env!("CARGO_PKG_VERSION")))
        .send()
        .await?)
}
