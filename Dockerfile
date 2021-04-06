FROM golang:latest AS builder

ENV DEBIAN_FRONTEND=noninteractive

RUN apt-get update \
    && apt-get -y install sudo

WORKDIR /build

COPY go.mod .

RUN go mod download

COPY . .

RUN go build main.go

FROM ubuntu:latest

WORKDIR /app

COPY --from=builder /build/main .

RUN mkdir root && cp -r /usr/bin ./root/

CMD [ "./main" ]
