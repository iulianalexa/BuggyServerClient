# Buggy Server

## Overview

This CLI application connects to the buggy server provided [here](https://gist.github.com/vladimirlagunov/dcdf90bb19e9de306344d46f20920dce). The servers sends some randomised data on every GET request, but often it only sends partial data.

The server supports the `Range` HTTP header, so in this implementation I make use of it by connecting to the server multiple times and asking for the bytes that I have not received yet, until the entire file has been read.

The test task does not specify what should be done with this file, so I read all contents into memory. Because Rust does not provide a sha256 implementation directly (and external crates should be avoided), I decided to save the contents to `downloaded.bin`.

Please check that the SHA-256 checksum matches the one reported by the server by running:

```bash
sha256sum downloaded.bin
```

This application was created as a solution to the task required for applying to the 2025 JetBrains Internship Project **Effective file transfer between machines for remote development**.
