[//]: # (CONFIG_DOCS_START)

| Field Name        | Type   | Required | Default        | Details       | Group            |
|-------------------|--------|----------|----------------|---------------|------------------|
| postgres-host     | String | Yes      | -              | Database host | DatabaseConfig   |
| postgres-port     | u16    | No       | 5432           | Database port | DatabaseConfig   |
| postgres-user     | String | Yes      | -              |               | DatabaseConfig   |
| postgres-password | String | Yes      | -              |               | DatabaseConfig   |
| postgres-database | String | No       | data-ingestion |               | DatabaseConfig   |
| redis-host        | String | Yes      | -              | Redis host    | RedisConfig      |
| redis-port        | u16    | No       | 6379           | Redis port    | RedisConfig      |
| port              | u16    | No       | 8080           | Server port   | NestedConfigFlat |

[//]: # (CONFIG_DOCS_END)