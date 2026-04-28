use std::io::Write;

use bollard::container::LogsOptions;
use futures_util::StreamExt;

use cubelit_core::error::{CoreError, CoreResult};
use cubelit_core::server::ServerLifecycle;

use crate::context::Context;
use crate::resolve::resolve_server_id;

pub async fn follow(ctx: &Context, id: &str, tail: u64) -> CoreResult<()> {
    let servers = ctx.host.list_servers().await?;
    let id = resolve_server_id(&servers, id)?;
    let cubelit = ctx.host.get_server(&id).await?;
    let container_id = cubelit.container_id.ok_or_else(|| {
        CoreError::NotFound("No container associated with this server".into())
    })?;

    let opts = LogsOptions::<String> {
        follow: true,
        stdout: true,
        stderr: true,
        tail: tail.to_string(),
        ..Default::default()
    };

    let mut stream = ctx.host.docker.logs(&container_id, Some(opts));
    loop {
        tokio::select! {
            _ = tokio::signal::ctrl_c() => break,
            item = stream.next() => {
                match item {
                    None => break,
                    Some(Ok(output)) => {
                        print!("{}", output);
                        let _ = std::io::stdout().flush();
                    }
                    Some(Err(e)) => return Err(CoreError::Docker(e)),
                }
            }
        }
    }
    Ok(())
}
