#!/usr/bin/env sh

set -e

update() {
  git stash
  git pull
  git stash pop
}

build() {
  docker-compose build
  docker-compose down
  docker-compose up -d
}

update
build
