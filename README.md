# LSP Proxy

This is a tool to intercept and replay a LSP's communications.
It allows developers to understand the communications between a
client and a server; and also to replay them.

In theory, it could also be used to make debugging simpler, though
more work is left to do on it.

It's currently very rough, and only contains two binaries:

 - `lsp-proxy` -- intercepts communications between a client (VSCode) and
   the server (`rust-analyser`). Communications are written to a folder
   as specified in a config file.
 - `lsp-replay` -- accepts one path on each line of the input.
   opens that file, which should contain plain JSON, and encodes
   it as an LSP message. The LSP is then printed to stdout.


## What LSP Proxy Does

LSP Proxy outputs communications between the client and server to files
in a specificed folder. It means that you end up with a series of 
timestamped files showing exactly what the inputs and outputs of 
RustAnalyzer (and the client interacting with it) were.

For example, you might end up with the following:

``` sh
$ ls /path/to/lsp-captures/ | sort
1669457623539_client-to-server.json
1669457623542_server-to-client.json
1669457623545_client-to-server.json
1669457623547_server-to-client.json
1669457623562_client-to-server.json
1669457623563_client-to-server.json
1669457623643_client-to-server.json
1669457623644_client-to-server.json
1669457623644_server-to-client.json
1669457623645_server-to-client.json
1669457623769_server-to-client.json
1669457623820_client-to-server.json
1669457623821_server-to-client.json
...
```

Each json file is either a message from the client to the server,
 or the server to the client. The number is a unix timestamp in
 milliseconds, so you can order the messages by time sent.

## How To Use

### 1. Build the system

``` sh
$ cargo build
```

### 2. Place the configuration file in the same folder as the binaries.

Put a file named `lsp_proxy.toml` in the same directory as the binaries
(by default, `lsp-proxy/target/debug/`).

``` toml
output_folder = "/path/to/lsp-captures/"
binary = "/path/to/rust-analyzer/target/release/rust-analyzer"
```


Ensure that `output_folder` exists.
Ensure that `binary` is the path to your "real" rust-analyser installation.

### 3. Tell VSCode to Use The Proxy

In your VSCode settings, add the following to your `settings.json`:

``` json
    "rust-analyzer.server.path": "/path/to/lsp-proxy/target/debug/lsp-proxy"
```

### 4. Start VSCode!


