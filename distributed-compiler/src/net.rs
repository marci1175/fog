use common::{
    anyhow,
    dependency::DependencyRequest,
    reqwest::{
        Client, Response,
        header::{CONTENT_TYPE, USER_AGENT},
    },
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
        .get(format!("{remote_url}/fetch_dependency"))
        .body(serde_json::to_string(&DependencyRequest {
            name: dep_name,
            version,
        })?)
        .header(USER_AGENT, format!("FDCN({})", env!("CARGO_PKG_VERSION")))
        .header(CONTENT_TYPE, "application/json")
        .send()
        .await?)
}

pub async fn request_dependency_information(
    client: Client,
    remote_url: &str,
    dep_name: String,
    version: String,
) -> anyhow::Result<Response>
{
    Ok(client
        .get(format!("{remote_url}/fetch_dependency_information"))
        .body(serde_json::to_string(&DependencyRequest {
            name: dep_name,
            version,
        })?)
        .header(USER_AGENT, format!("FDCN({})", env!("CARGO_PKG_VERSION")))
        .header(CONTENT_TYPE, "application/json")
        .send()
        .await?)
}
