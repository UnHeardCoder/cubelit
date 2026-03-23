CREATE TABLE IF NOT EXISTS cubelits (
    id TEXT PRIMARY KEY NOT NULL,
    name TEXT NOT NULL,
    game TEXT NOT NULL,
    recipe_id TEXT NOT NULL,
    docker_image TEXT NOT NULL,
    container_id TEXT,
    status TEXT NOT NULL DEFAULT 'created',
    port_mappings TEXT NOT NULL DEFAULT '{}',
    environment TEXT NOT NULL DEFAULT '{}',
    volume_path TEXT NOT NULL,
    container_mount_path TEXT NOT NULL DEFAULT '/data',
    sidecar_container_id TEXT,
    sidecar_image TEXT,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);
