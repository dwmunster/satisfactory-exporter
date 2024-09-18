# Satisfactory Exporter

This project is a prometheus exporter for the game [Satisfactory](https://www.satisfactorygame.com/).
It periodically queries a dedicated server via the HTTPS API for metrics and exposes them via an HTTP endpoint.

## Usage

### CLI

```
Usage: satisfactory-exporter [OPTIONS] --endpoint <ENDPOINT>

Options:
      --update-interval <UPDATE_INTERVAL>
          Interval in seconds between each query to the server [env: SE_UPDATE_INTERVAL=] [default: 5]
      --endpoint <ENDPOINT>
          Hostname and port of the server to query [env: SE_ENDPOINT=]
      --token-file <TOKEN_FILE>
          File containing the bearer token to use for authentication. Mutually exclusive with --token [env: SE_TOKEN_FILE=]
      --token <TOKEN>
          Bearer token to use for authentication. Mutually exclusive with --token-file [env: SE_TOKEN=]
      --allow-insecure
          Allow insecure connections (e.g., to a server with a self-signed certificate) [env: SE_ALLOW_INSECURE=]
      --listen <LISTEN>
          Address:Port to which the server will listen [env: SE_LISTEN=] [default: 127.0.0.1:3030]
  -h, --help
          Print help
  -V, --version
          Print version

```

### Generating an API token

You can create an API token by executing the following command in the Satisfactory server console:

   ```
   server.GenerateAPIToken
   ```

It will create a long string that looks like the following:

   ```text
   ewoJInBsIjogIkFQSVRva2VuIgp9.<Long string of characters>
   ```

This token can then be used with the `--token` option, or saved to a file and used with the `--token-file` option.

### Connecting via Insecure HTTPS

If the server uses a self-signed certificate (the default), you can use the `--allow-insecure` option to allow the
exporter to connect to it.

### Example

```sh
./satisfactory-explorer --endpoint game.example.com:7777 --token-file /path/to/token.txt
```

Or, for a server with a self-signed certificate:

```sh
./satisfactory-explorer --endpoint game.example.com:7777 --token-file /path/to/token.txt --allow-insecure
```

## License

This project is licensed under the MIT License. See the `LICENSE` file for details.