# http://marmelab.com/blog/2016/02/29/auto-documented-makefile.html
include .env

help: ## Show list of make targets and their description
	@grep -E '^[/%.a-zA-Z_-]+:.*?## .*$$' Makefile \
		| awk 'BEGIN {FS = ":.*?## "}; {printf "\033[36m%-15s\033[0m %s\n", $$1, $$2}'

.DEFAULT_GOAL:= help

.PHONY: database/up
database/up: ## bring up databases and nginx
	cd docker && docker compose -f ./docker-compose.yml up -d

.PHONY: database/down
database/down: ## tear down databases and nginx
	cd docker && docker compose -f ./docker-compose.yml down

.PHONY: server
server: ## build and start server
	cargo run --release

.PHONY: test
test: test/unit test/integration ## run all test case including unit test and integration test

.PHONY: test/unit
test/unit: ## unit test
	echo TODO

.PHONY: test/integration
test/integration: ## integration test, need to bring up server by `make server`
	echo TODO
