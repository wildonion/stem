docker run -d \
        -h redis \
        -e REDIS_PASSWORD=geDteDd0Ltg2135FJYQ6rjNYHYkGQa70 \
        -v $(pwd)/infra/data/redis/:/data \
        -p 6379:6379 \
        --name redis \
        --network onion \
        --restart always \
        redis:latest /bin/sh -c 'redis-server --appendonly yes --requirepass ${REDIS_PASSWORD}'
docker run -d --network onion --name postgres --restart unless-stopped -p 5432:5432 -v $(pwd)/infra/data/postgres/:/var/lib/postgresql/data -e POSTGRES_PASSWORD=geDteDd0Ltg2135FJYQ6rjNYHYkGQa70 -e POSTGRES_USER=postgres -e PGDATA=/var/lib/postgresql/data/pgdata postgres
docker run -d --network onion --link postgres -p 8080:8080 adminer
docker run -d --network onion --name rabbitmq -e RABBITMQ_DEFAULT_USER=rabbitmq -e RABBITMQ_DEFAULT_PASS=geDteDd0Ltg2135FJYQ6rjNYHYkGQa70 -p 5672:5672 -p 15672:15672 rabbitmq:4.0-management