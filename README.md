# URL Shortener

- [1.How to run](#how-to-run)
  - [Ubuntu](#ubuntu)
- [2.Functional requirement](#functional-requirement)
- [3.TODOs](#todos)

## How to run
------------
### Ubuntu
> **_NOTE:_** only tested on ubuntu 20.04 LTS, should only work on later version (inclusive)

Then run/follow commands and comments below
```
# install dependency
sudo apt update
sudo apt install -y make

# install docker
# https://docs.docker.com/engine/install/ubuntu/

# install docker compose if not yet
# MAKE SURE YOU USE DOCKER COMPOSE V2, OTHERWISE THINGS MIGHT BE BROKEN
# i.e. `docker compose` should work, instead of `docker-compose`
# https://docs.docker.com/compose/install/linux/

# to make sure you can run docker in non-root mode
# following command might be needed
sudo groupadd docker
sudo usermod -aG docker $USER
newgrp docker

# showing help message
make

# start database
# it is normal to block for a while (~30s),
# since it will wait until mysql to be ready
make database/up

# start the app
# you might get a panic
make server

# now you can run test
# run in another terminal
# make test

# run in another termianl
# example command
# port 11111 for accessing server directly
curl -X POST 'http://localhost:8000/url' -d '{"url":"https://google.com"}'  
curl -X GET -L 'http://localhost:8000/1234567'
curl -X GET 'http://localhost:8000/url'
``` 


## Functional requirement
-------------------------
 - A URL shortener service exposing two endpoint
   - POST /url
     - Create short URL
   - GET /{url_id}
     - Access short URL
   - GET /url
     - List all URL

## TODOs
--------
- [ ] implement bloom filter
- [ ] add CI
