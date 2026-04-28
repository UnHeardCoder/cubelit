use std::collections::HashMap;
use std::io::{BufRead, Write};
use std::sync::Arc;

use tabled::{Table, Tabled};

use cubelit_core::error::{CoreError, CoreResult};
use cubelit_core::recipes::get_recipe;
use cubelit_core::server::{CreateServerConfig, ServerLifecycle, ServerRunner};

use crate::context::Context;
use crate::resolve::resolve_server_id;
use crate::sink::CliEventSink;

#[derive(Tabled)]
struct ListRow {
    #[tabled(rename = "ID")]
    id: String,
    #[tabled(rename = "GAME")]
    game: String,
    #[tabled(rename = "NAME")]
    name: String,
    #[tabled(rename = "STATUS")]
    status: String,
    #[tabled(rename = "CREATED")]
    created: String,
}

pub async fn list(ctx: &Context) -> CoreResult<()> {
    ctx.host.sync_all().await?;
    let mut rows = Vec::new();
    for s in ctx.host.list_servers().await? {
        let status = if s.status == "starting" {
            if let Some(container_id) = s.container_id.as_deref() {
                if ctx.host.is_running(container_id).await {
                    "running".to_string()
                } else {
                    s.status.clone()
                }
            } else {
                s.status.clone()
            }
        } else {
            s.status.clone()
        };
        rows.push(ListRow {
            id: s.id.chars().take(8).collect(),
            game: s.game,
            name: s.name,
            status,
            created: s.created_at,
        });
    }
    println!("{}", Table::new(rows));
    Ok(())
}

pub async fn install(
    ctx: &Context,
    recipe_id: String,
    name: String,
    port: Option<u16>,
) -> CoreResult<()> {
    let recipe = get_recipe(&ctx.host.recipes_dir, &recipe_id)?;
    let port_overrides = if let Some(host_port) = port {
        let first = recipe.ports.first().ok_or_else(|| {
            CoreError::Validation("Recipe has no ports to apply --port to".into())
        })?;
        let key = format!("{}/{}", first.container_port, first.protocol);
        Some(HashMap::from([(key, host_port)]))
    } else {
        None
    };
    let config = CreateServerConfig {
        name,
        recipe_id,
        port_overrides,
        env_overrides: None,
        volume_path: None,
        tag_override: None,
    };
    let sink: Arc<dyn cubelit_core::events::EventSink> = Arc::new(CliEventSink::stdio());
    let cubelit = ctx.host.create_server(config, sink).await?;
    println!("{}", cubelit.id);
    Ok(())
}

pub async fn start(ctx: &Context, id: &str) -> CoreResult<()> {
    let servers = ctx.host.list_servers().await?;
    let id = resolve_server_id(&servers, id)?;
    let sink: Arc<dyn cubelit_core::events::EventSink> = Arc::new(CliEventSink::stdio());
    ctx.host.start_server(&id, sink).await
}

pub async fn stop(ctx: &Context, id: &str) -> CoreResult<()> {
    let servers = ctx.host.list_servers().await?;
    let id = resolve_server_id(&servers, id)?;
    ctx.host.stop_server(&id).await
}

pub async fn restart(ctx: &Context, id: &str) -> CoreResult<()> {
    let servers = ctx.host.list_servers().await?;
    let id = resolve_server_id(&servers, id)?;
    let sink: Arc<dyn cubelit_core::events::EventSink> = Arc::new(CliEventSink::stdio());
    ctx.host.restart_server(&id, sink).await
}

pub async fn status(ctx: &Context, id: &str) -> CoreResult<()> {
    let servers = ctx.host.list_servers().await?;
    let id = resolve_server_id(&servers, id)?;
    let s = ctx.host.sync_single(&id).await?;
    let ports_pretty = serde_json::from_str::<serde_json::Value>(&s.port_mappings)
        .map(|v| serde_json::to_string_pretty(&v).unwrap_or_else(|_| s.port_mappings.clone()))
        .unwrap_or_else(|_| s.port_mappings.clone());
    println!("id:             {}", s.id);
    println!("name:           {}", s.name);
    println!("game:           {}", s.game);
    println!("recipe_id:      {}", s.recipe_id);
    let display_status = if s.status == "starting" {
        if let Some(container_id) = s.container_id.as_deref() {
            if ctx.host.is_running(container_id).await {
                "running".to_string()
            } else {
                s.status.clone()
            }
        } else {
            s.status.clone()
        }
    } else {
        s.status.clone()
    };
    println!("status:         {}", display_status);
    println!("container_id:   {}", s.container_id.as_deref().unwrap_or("—"));
    println!("docker_image:   {}", s.docker_image);
    println!("port_mappings:\n{}", ports_pretty);
    println!("volume_path:    {}", s.volume_path);
    println!("created_at:     {}", s.created_at);
    println!("updated_at:     {}", s.updated_at);
    Ok(())
}

pub async fn remove(ctx: &Context, id: &str, keep_data: bool, yes: bool) -> CoreResult<()> {
    let servers = ctx.host.list_servers().await?;
    let id = resolve_server_id(&servers, id)?;
    if !yes {
        print!(
            "Remove server {} and {} on-disk data? [y/N] ",
            id,
            if keep_data { "keep" } else { "delete" }
        );
        std::io::stdout().flush().map_err(CoreError::Io)?;
        let mut line = String::new();
        std::io::stdin()
            .lock()
            .read_line(&mut line)
            .map_err(CoreError::Io)?;
        let line = line.trim();
        if !line.eq_ignore_ascii_case("y") && !line.eq_ignore_ascii_case("yes") {
            return Ok(());
        }
    }
    ctx.host.delete_server(&id, !keep_data).await
}
