FROM node:alpine

RUN npm install -g elm elm-test

COPY ./elm.json ./elm.json
COPY ./src ./src
COPY ./tests ./tests

RUN elm make src/Main.elm
