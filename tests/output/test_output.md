[//]: # (CONFIG_DOCS_START)

| Field Name               | Type   | Required | Default        | Details       | Group      |
|--------------------------|--------|----------|----------------|---------------|------------|
| postgres-host            | String | Yes      | -              | Database host | TestConfig |
| postgres-port            | u16    | No       | 5432           | Database port | TestConfig |
| postgres-user            | String | Yes      | -              |               | TestConfig |
| postgres-password        | String | Yes      | -              |               | TestConfig |
| postgres-database        | String | No       | data-ingestion |               | TestConfig |
| postgres-connection-pool | u32    | No       | 5              |               | TestConfig |

[//]: # (CONFIG_DOCS_END)