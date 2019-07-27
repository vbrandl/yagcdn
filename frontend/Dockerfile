FROM node:alpine

RUN yarn global add elm elm-test

COPY ./elm.json ./elm.json
COPY ./src ./src
COPY ./tests ./tests

RUN elm make src/Main.elm
