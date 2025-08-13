[//]: # (CONFIG_DOCS_START)

| Field Name | Type   | Required | Default | Details              | Group          |
|------------|--------|----------|---------|----------------------|----------------|
| port       | u16    | No       | 8080    | Server port          | MainConfig     |
| db-host    | String | Yes      | -       | Database host        | DatabaseConfig |
| db-port    | u16    | No       | 5432    | Database port        | DatabaseConfig |
| cache-host | String | Yes      | -       | Cache host           | CacheConfig    |
| cache-ttl  | u32    | No       | 3600    | Cache TTL in seconds | CacheConfig    |

[//]: # (CONFIG_DOCS_END)