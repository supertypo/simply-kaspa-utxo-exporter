# Simply Kaspa UTXO Exporter
UTXO exporter tool, currently exports top scripts (aka. rich list) and address distribution.

The exporter reads the UTXO set by directly accessing the Rusty Kaspad data directory in read-only mode.    
Kaspad doesn't need to be stopped and should be unaffected by this tool, however this tool can fail if Kaspad is currently performing block / header pruning.  
In case of failure to read from Kaspad (or writing to Postgres), it will retry using the configured retry intervals (see help).


## Binary releases
Docker images are available from https://hub.docker.com/r/supertypo/simply-kaspa-utxo-exporter


## Contribute to development
kaspa:qrjtsnnpjyvlmkffdqyayrny3qyen9yjkpuw7xvhsz36n69wmrfdyf3nwv67t


## Help
```
Usage: simply-kaspa-utxo-exporter [OPTIONS]

Options:
  -n, --network <NETWORK>
          The network type and suffix, e.g. 'testnet-10' [default: mainnet]
  -b, --base-dir <BASE_DIR>
          Kaspad data base directory [default: ~/.rusty-kaspa]
      --consensus-dir <CONSENSUS_DIR>
          Kaspad consensus subdir, leave empty to auto-detect
  -d, --database-url <DATABASE_URL>
          PostgreSQL url(s) [default: postgres://postgres:postgres@localhost:5432/postgres]
  -i, --interval-minutes <INTERVAL_MINUTES>
          Interval between utxo set rescanning [default: 60]
      --ignore-dust-amounts <IGNORE_DUST_AMOUNTS>
          Ignore utxos with amounts less than this (in sompi/litra) [default: 10000]
      --top-scripts-count <TOP_SCRIPTS_COUNT>
          Number of top scripts to index [default: 1000]
      --top-scripts-min-amount <TOP_SCRIPTS_MIN_AMOUNT>
          The minimum balance to be considered for top-n list [default: 100000]
      --data-dir-retry-interval <DATA_DIR_RETRY_INTERVAL>
          Interval between datadir read retries (in seconds) [default: 120]
      --db-retry-interval <DB_RETRY_INTERVAL>
          Interval between db retries (in seconds) [default: 30]
      --db-retry-count <DB_RETRY_COUNT>
          How many times to retry commit to db before continuing [default: 20]
  -c, --initialize-db
          Empties the tables. Use with care
  -l, --log-level <LOG_LEVEL>
          error, warn, info, debug, trace, off [default: info]
      --log-no-color
          Disable colored output
  -h, --help
          Print help
  -V, --version
          Print version
```
