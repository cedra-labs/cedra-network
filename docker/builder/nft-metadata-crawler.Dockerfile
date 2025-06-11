### NFT Metadata Crawler Image ###

FROM indexer-builder

FROM debian-base AS nft-metadata-crawler

COPY --link --from=indexer-builder /cedra/dist/cedra-nft-metadata-crawler /usr/local/bin/cedra-nft-metadata-crawler

# The health check port
EXPOSE 8080
