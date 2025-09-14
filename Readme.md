Creating migrations\20250909140458_create_github_account.sql

sqlx migrate run

https://github.com/settings/applications/3158873

https://github.com/settings/developers



DOCKER 


docker-compose up -d
docker-compose up --build -d
docker-compose up --build -d --remove-orphans
docker-compose build --no-cache

docker-compose down -v
docker-compose down -v --rmi all
docker system prune -a --volumes
docker system prune -a 

docker-compose down && docker-compose up -d --build

sudo systemctl restart  apache2

docker-compose up -d db

docker-compose ps

docker-compose build --no-cache

docker-compose restart web

docker logs httpdocs-web-1

docker logs httpdocs-web-1 2>&1 | grep -i error

docker network prune
docker volume ls
docker exec -it nginx nginx -t



=====Redis

docker run -d --name redis -p 6379:6379 redis

docker exec -it 02f588ccde37f67e71a47e2643534bf2a4285ae18bf8cdb868d8e518ca639370 redis-cli FLUSHDB

docker run -it --network my-network redis redis-cli -h redis-container-name FLUSHALL

docker stop redis

docker rm redis

docker run -d -p 5080:5080 --name antmedia antmedia/enterprise

docker run --rm -it --network host kurento/kurento-media-serve

docker stop antmedia

docker  stop "/antmedia"

docker rm antmedia

docker stop antmedia/enterprise

redis-cli --scan --pattern "user:*" | xargs redis-cli DEL

docker run -d --name janus -p 8188:8188 -p 8088:8088 -p 10000-10200:10000-10200/udp lars-berger/janus-gatewa