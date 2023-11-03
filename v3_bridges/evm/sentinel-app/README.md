In order to run the app you'll need a configuration file at the root of this repo called `sentinel-config.toml`. To see an example of such a file, see `../../../common/sentinel/src/config/test_utils/sample-config.toml`

You'll also need to provision a private key for broadcasting transactions via an `.env` file at the root of this repo, containing the private key under the key: `PRIVATE_KEY`. Any `.env` files in this repo will be ignored by git.
