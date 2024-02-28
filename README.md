# chost

a small tool to host local static files.

## install

Shell(Mac, Linux)

```sh
curl -fsSL https://raw.githubusercontent.com/TrickyPi/chost/main/install/install.sh | sh
```

## usage

```sh
chost [OPTIONS] [PATH]
```

### PATH

The is the `PATH` to be hosted, and it is required. The usage like `chost ./dist`.

### OPTIONS

#### cors

This is a `boolean` flag to enable or disable `CORS`, the default is `false`. It can be enabled with the `-c` or `--cors` flag. For example: `chost ./dist --cors`

#### port

This is the port number on which the application will run. It can be specified with the `-p` or `--port` flag followed by the port number. If not specified, the default port is `7878`. For example: `chost ./dist --port 7879`.

Notice: If the specified port or the default port is not available, chost will attempt to find a free port within the range from `7878` to `8989`. However, if no free port is found within this range, chost will throw an error.

#### proxy

This is used for forwarding requests to other services. It can be specified with the `--proxy` flag followed by the strings. The format for each proxy string is `${api}|${origin}`. Multiple proxy strings can be separated by a space. For example: `chost ./dist --proxy "api1|origin1 api2|origin2"`.
