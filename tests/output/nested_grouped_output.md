[//]: # (CONFIG_DOCS_START)

## DatabaseConfig Configuration

| Field Name        | Type   | Required | Default        | Details       |
|-------------------|--------|----------|----------------|---------------|
| postgres-host     | String | Yes      | -              | Database host |
| postgres-port     | u16    | No       | 5432           | Database port |
| postgres-user     | String | Yes      | -              |               |
| postgres-password | String | Yes      | -              |               |
| postgres-database | String | No       | data-ingestion |               |

## RedisConfig Configuration

| Field Name | Type   | Required | Default | Details    |
|------------|--------|----------|---------|------------|
| redis-host | String | Yes      | -       | Redis host |
| redis-port | u16    | No       | 6379    | Redis port |

## NestedConfigGrouped Configuration

| Field Name | Type | Required | Default | Details     |
|------------|------|----------|---------|-------------|
| port       | u16  | No       | 8080    | Server port |



[//]: # (CONFIG_DOCS_END)