# clap_autodoc

Auto-generate markdown documentation tables from clap configuration structs.

## Quick Start

```toml
[dependencies]
clap_autodoc = "0.2.0"
clap = { version = "4.0", features = ["derive"] }
```

### Basic Usage

```rust
use clap_autodoc::generate;
use clap::Parser;

#[derive(Parser)]
#[clap(rename_all = "kebab-case")]
#[generate(target = "README.md")]
pub struct Config {
    /// Database host
    #[clap(env = "DATABASE_HOST", long)]
    pub database_host: String,
    
    /// Database port
    #[clap(env = "DATABASE_PORT", long, default_value_t = 5432)]
    pub database_port: u16,
}
```

**Generated output:**

| Field Name | Type | Required | Default | Details | Group |
|------------|------|----------|---------|---------|-------|
| database-host | String | Yes | - | Database host | Config |
| database-port | u16 | No | 5432 | Database port | Config |

### Nested Structs (Grouped Format)

```rust
use clap_autodoc::{generate, register};
use clap::Parser;

#[derive(Parser)]
#[generate(target = "CONFIG.md", format = "grouped")]
pub struct AppConfig {
    #[clap(flatten)]
    pub database: DatabaseConfig,
    
    #[clap(long, default_value_t = 8080)]
    pub port: u16,
}

#[derive(Parser)]
#[register]  // Required for nested structs
pub struct DatabaseConfig {
    /// Database host
    #[clap(long)]
    pub host: String,
}
```

**Generated output:**

## DatabaseConfig Configuration
| Field Name | Type | Required | Default | Details |
|------------|------|----------|---------|---------|
| host | String | Yes | - | Database host |

## AppConfig Configuration
| Field Name | Type | Required | Default | Details |
|------------|------|----------|---------|---------|
| port | u16 | No | 8080 | - |

## How It Works

1. Add `[//]: # (CONFIG_DOCS_START)` and `[//]: # (CONFIG_DOCS_END)` markers to your markdown file
2. The macro generates tables between these markers during compilation
3. Supports flat (single table) and grouped (separate sections) formats

## Options

- `target`: Path to markdown file (required)
- `format`: `"flat"` (default) or `"grouped"`

## License

MIT OR Apache-2.0
