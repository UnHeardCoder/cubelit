use cubelit_core::db::models::Cubelit;
use cubelit_core::error::{CoreError, CoreResult};

pub fn resolve_server_id(servers: &[Cubelit], input: &str) -> CoreResult<String> {
    if let Some(s) = servers.iter().find(|s| s.id == input) {
        return Ok(s.id.clone());
    }

    let name_matches: Vec<_> = servers.iter().filter(|s| s.name == input).collect();
    match name_matches.len() {
        0 => {}
        1 => return Ok(name_matches[0].id.clone()),
        n => {
            return Err(CoreError::Validation(format!(
                "ambiguous identifier '{input}'; matched {n} servers — use the full UUID"
            )));
        }
    }

    if input.len() >= 4 {
        let pref: Vec<_> = servers
            .iter()
            .filter(|s| s.id.starts_with(input))
            .collect();
        match pref.len() {
            0 => {}
            1 => return Ok(pref[0].id.clone()),
            n => {
                return Err(CoreError::Validation(format!(
                    "ambiguous identifier '{input}'; matched {n} servers — use the full UUID"
                )));
            }
        }
    }

    Err(CoreError::NotFound(format!(
        "no server matching '{input}'"
    )))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample(id: &str, name: &str) -> Cubelit {
        Cubelit {
            id: id.into(),
            name: name.into(),
            game: "g".into(),
            recipe_id: "r".into(),
            docker_image: "i".into(),
            container_id: None,
            status: "stopped".into(),
            port_mappings: "{}".into(),
            environment: "{}".into(),
            volume_path: "/tmp".into(),
            container_mount_path: "/data".into(),
            sidecar_container_id: None,
            sidecar_image: None,
            created_at: "c".into(),
            updated_at: "u".into(),
        }
    }

    #[test]
    fn exact_uuid() {
        let servers = vec![sample("550e8400-e29b-41d4-a716-446655440000", "srv")];
        let id = resolve_server_id(&servers, "550e8400-e29b-41d4-a716-446655440000").unwrap();
        assert_eq!(id, "550e8400-e29b-41d4-a716-446655440000");
    }

    #[test]
    fn exact_unique_name() {
        let servers = vec![sample("550e8400-e29b-41d4-a716-446655440000", "alpha")];
        let id = resolve_server_id(&servers, "alpha").unwrap();
        assert_eq!(id, "550e8400-e29b-41d4-a716-446655440000");
    }

    #[test]
    fn unique_prefix() {
        let servers = vec![sample("550e8400-e29b-41d4-a716-446655440000", "a")];
        let id = resolve_server_id(&servers, "550e8400").unwrap();
        assert_eq!(id, "550e8400-e29b-41d4-a716-446655440000");
    }

    #[test]
    fn ambiguous_name() {
        let servers = vec![
            sample("550e8400-e29b-41d4-a716-446655440000", "dup"),
            sample("660e8400-e29b-41d4-a716-446655440001", "dup"),
        ];
        assert!(matches!(
            resolve_server_id(&servers, "dup"),
            Err(CoreError::Validation(_))
        ));
    }

    #[test]
    fn ambiguous_prefix() {
        let servers = vec![
            sample("550e8400-e29b-41d4-a716-446655440000", "a"),
            sample("550e8400-e29b-41d4-a716-446655440001", "b"),
        ];
        assert!(matches!(
            resolve_server_id(&servers, "550e8400-e29b-41d4-a716-44665544000"),
            Err(CoreError::Validation(_))
        ));
    }

    #[test]
    fn no_match() {
        let servers = vec![sample("550e8400-e29b-41d4-a716-446655440000", "a")];
        assert!(matches!(
            resolve_server_id(&servers, "nope"),
            Err(CoreError::NotFound(_))
        ));
    }

    #[test]
    fn prefix_too_short_falls_through_to_not_found() {
        let servers = vec![sample("550e8400-e29b-41d4-a716-446655440000", "a")];
        assert!(matches!(
            resolve_server_id(&servers, "550"),
            Err(CoreError::NotFound(_))
        ));
    }
}
