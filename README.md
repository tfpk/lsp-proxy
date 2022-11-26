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


