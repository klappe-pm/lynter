# lynter

Lynter lints and fixes notes and code from one `lynter.yaml`. This package installs the prebuilt `lynter` command for your platform.

```bash
npm install -g lynter
lynter init
lynter check .
```

On install, the package downloads the `lynter` binary for your platform from the matching [GitHub release](https://github.com/klappe-pm/lynter/releases), checks it against the release's `SHA256SUMS`, and unpacks it. Linux and macOS on x86_64 and arm64, and Windows on x86_64, have binaries. Set `LYNTER_SKIP_DOWNLOAD=1` to skip the download.

Documentation, a playground and a config builder are at [lynter.dev](https://lynter.dev).

## License

MIT. Third-party notices ship inside each release archive.
