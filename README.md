# clap_autodoc

A Rust proc macro for automatically generating markdown documentation from configuration structs with clap derive attributes.

## Features

- **Automatic Documentation Generation**: Generates markdown tables from struct fields with clap attributes
- **Nested Struct Support**: Handles `#[clap(flatten)]` fields and nested configurations with proper field expansion
- **Configurable Output Formats**:
  - `flat`: Single table with all fields and a Group column
  - `grouped`: Separate sections for each nested struct

## Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
clap_autodoc = "0.1.0"
clap = { version = "4.0", features = ["derive", "env"] }
```

## Usage

### Configuration Options

#### `target` (required)
The path to the target markdown file where the documentation will be inserted.

#### `format` (optional, default: "flat")
- `"flat"`: Single table with all fields and a Group column
- `"grouped"`: Separate sections for each nested struct


### Flat format

```rust
use clap_autodoc::generate;
use clap::Parser;

#[derive(Clone, Debug, Parser)]
#[clap(rename_all = "kebab-case")]
#[generate(target = "README.md")]
pub struct Config {
    /// Database host
    #[clap(env = "DATABASE_HOST", long)]
    pub database_host: String,
    
    /// Database port
    #[clap(env = "DATABASE_PORT", long, default_value_t = 5432)]
    pub database_port: u16,
    
    /// Connection timeout in seconds
    #[clap(env = "CONNECTION_TIMEOUT", long, default_value_t = 30)]
    pub connection_timeout: u32,
}
```

### Flat format output

| Field Name | Type | Required | Default | Details | Group |
|------------|------|----------|---------|---------|-------|
| database-host | String | Yes | - | Database host | Config |
| database-port | u16 | No | 5432 | Database port | Config |
| connection-timeout | u32 | No | 30 | Connection timeout in seconds | Config |

### Grouped format

For nested structs with `#[clap(flatten)]`, you need to register the nested structs using `#[register]`. The documentation will be generated automatically when all dependencies are available. Note though, that the macro will not start generating documantation unless all the nested structs has been registered by the user.

```rust
use clap_autodoc::{generate, register};
use clap::Parser;

// Main config struct
#[derive(Clone, Debug, Parser)]
#[clap(rename_all = "kebab-case")]
#[generate(target = "CONFIG.md", format = "grouped")]
pub struct AppConfig {
    /// Database configuration
    #[clap(flatten)]
    pub database: DatabaseConfig,
    
    /// Redis configuration
    #[clap(flatten)]
    pub redis: RedisConfig,
    
    /// Server port
    #[clap(env = "SERVER_PORT", long, default_value_t = 8080)]
    pub port: u16,
}

// Make sure to register the nested struct with `register`
#[derive(Clone, Debug, Parser)]
#[clap(rename_all = "kebab-case")]
#[register]
pub struct DatabaseConfig {
    /// Database host
    #[clap(env = "POSTGRES_HOST", long)]
    pub postgres_host: String,
    
    /// Database port
    #[clap(env = "POSTGRES_PORT", long, default_value_t = 5432)]
    pub postgres_port: u16,
}

#[derive(Clone, Debug, Parser)]
#[clap(rename_all = "kebab-case")]
#[register]
pub struct RedisConfig {
    /// Redis host
    #[clap(env = "REDIS_HOST", long)]
    pub redis_host: String,
    
    /// Redis port
    #[clap(env = "REDIS_PORT", long, default_value_t = 6379)]
    pub redis_port: u16,
}
```

### Grouped format output

## DatabaseConfig Configuration

| Field Name | Type | Required | Default | Details |
|------------|------|----------|---------|---------|
| postgres-host | String | Yes | - | Database host |
| postgres-port | u16 | No | 5432 | Database port |

## RedisConfig Configuration

| Field Name | Type | Required | Default | Details |
|------------|------|----------|---------|---------|
| redis-host | String | Yes | - | Redis host |
| redis-port | u16 | No | 6379 | Redis port |

## AppConfig Configuration

| Field Name | Type | Required | Default | Details |
|------------|------|----------|---------|---------|
| port | u16 | No | 8080 | Server port |


### File Integration

The macro looks for specific markdown comment markers in your target file:

```markdown
# My Application Configuration

[//]: # (CONFIG_DOCS_START)
[//]: # (CONFIG_DOCS_END)

## Usage
...
```

The generated table will be inserted between these markers, replacing any existing content.

### Supported Clap Attributes
- `#[clap(default_value = "value")]` - String default value
- `#[clap(default_value_t = value)]` - Typed default value
- `#[clap(flatten)]` - Nested struct flattening
- `#[clap(rename_all = "case")]` - Field name transformation

### Field Data Extraction

The macro extracts the following information for each field:

- **Field Name**: Transformed according to `rename_all` settings
- **Type**: Rust type of the field
- **Required**: Whether the field has a default value
- **Default**: Default value if specified
- **Details**: Documentation comments (`///`)
- **Group**: Struct name or nested struct name for flattened fields


### Limitations
- Only supports named struct fields

## License

This project is dual-licensed under either:

- [MIT License](LICENSE-MIT)
- [Apache License, Version 2.0](LICENSE-APACHE)

at your option.

### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in the work by you, as defined in the Apache-2.0 license, shall be dual licensed as above, without any additional terms or conditions.
