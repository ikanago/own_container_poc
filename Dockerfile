FROM golang:latest AS builder

WORKDIR /build

COPY go.mod .

RUN go mod download

COPY . .

RUN go build main.go

FROM ubuntu:latest

WORKDIR /app

COPY --from=builder /build/main .

CMD [ "./main" ]
