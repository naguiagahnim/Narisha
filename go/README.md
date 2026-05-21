# tinyrwm.go

Tiny river window manager implemented in Go.

## Building

```sh
go build -o tinyrwm .
```

## Running

```sh
river -c ./tinyrwm
```

## Code generation

The code in [./internal/proto/gen.go](./internal/proto/gen.go) is automatically
generated based on the xml files in the [./protocol](./protocol) directory.
It can be regenerated with:

```sh
go generate ./internal/proto
```

