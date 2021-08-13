# ETH current block time

A simple web service that exposes the time of the newest produced block in Ethereum public blockchain in a form
of `GET /currentBlockTime`. Proxies https://etherscan.io/apis.

----

## Usage

_docker build -t eth-proxy ._

and then

_docker run -p 8080:8080 -e ETHERSCAN_API_KEY=<etherscan-api-key> --rm eth-proxy_
