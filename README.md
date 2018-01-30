# pathfinder
An asynchronous WebSocket-over-RabbitMQ reverse proxy, based on the tokio and futures-rs crates.

# Usage
```
USAGE:
    pathfinder [FLAGS] [OPTIONS]

FLAGS:
    -s, --secured     Enable the SSL/TLS mode for connections with RabbitMQ
    -v, --validate    Validate a token that was specified with data
    -h, --help        Prints help information
    -V, --version     Prints version information

OPTIONS:
    -c, --config <config>                                  Path to a custom settings file [default: ]
    -i, --ip <ip>                                          The used IP for a server [default: 127.0.0.1]
    -p, --port <port>                                      The listened port [default: 8080]
    -l, --log-level <log_level>                            Verbosity level filter of the logger [default: info]
        --rabbitmq-host <rabbitmq_host>                    The used host by RabbitMQ broker [default: 127.0.0.1]
        --rabbitmq-port <rabbitmq_port>                    The listened port by RabbitMQ broker [default: 5672]
        --rabbitmq-virtual-host <rabbitmq_virtual_host>    The virtual host of a RabbitMQ node [default: vhost]
        --rabbitmq-user <rabbitmq_username>                A RabbitMQ application username [default: user]
        --rabbitmq-password <rabbitmq_password>            A RabbitMQ application password [default: password]
        --redis-host <redis_host>                          The used host by Redis [default: 127.0.0.1]
        --redis-port <redis_port>                          The listened port by Redis [default: 6379]
        --redis-password <redis_password>                  Password for connecting to redis [default: ]
        --jwt-secret <jwt_secret_key>                      Secret key for a JWT validation [default: secret]
        --ssl-cert <ssl_certificate>                       Path to a SSL certificate [default: ]
        --ssl-key <ssl_public_key>                         Path to a SSL public key [default: ]
```

# Configuration file
For using a custom configuration for reverse proxy, you will need to specify `-c` (or `--config`) option with a path to
a file. For example:
```bash
pathfinder --config=myconfig.yaml -p 8001
```
At the current stage of this project, reverse proxy is support only endpoints list, which is using for mapping URLs into certain Kafka topics.
Each of those endpoints contains four fields:
- `url` - URL that specified by a client in each request. Required.
- `microservice` - Means the name of topic (or queue) where will be storing the message. This topic (or queue) is listening by certain microservice. Required.
- `request_exchange` - Defines the name of exchange point for RabbitMQ, through which the reverse proxy should publish a message. Optional.
- `response_exchange` - Defines the name of exchange point for RabbitMQ, through which the reverse proxy should consume a message. Optional.

### Example
```yaml
endpoints:
  - search:
      url: "/api/matchmaking/search"
      microservice: "microservice.search"
  - leaderboard:
      url: "/api/matchmaking/leaderboard"
      microservice: "microservice.leaderboard"
      request_exchange: "amqp.direct"
      response_exchange:  "open-matchmaking.default.direct"
```

# Documentation
Information about why this reverse proxy was implemented you can find [here](https://github.com/OpenMatchmaking/documentation/blob/master/docs/components.md#reverse-proxy).

# License
The pathfinder is published under BSD license. For more details read the [LICENSE](https://github.com/OpenMatchmaking/pathfinder/blob/master/LICENSE) file.
