FROM debian-base AS keyless-pepper-service

COPY --link --from=tools-builder /cedra/dist/cedra-keyless-pepper-service /usr/local/bin/cedra-keyless-pepper-service

EXPOSE 8000
ENV RUST_LOG_FORMAT=json

# add build info
ARG GIT_TAG
ENV GIT_TAG ${GIT_TAG}
ARG GIT_BRANCH
ENV GIT_BRANCH ${GIT_BRANCH}
ARG GIT_SHA
ENV GIT_SHA ${GIT_SHA}

ENTRYPOINT [ "cedra-keyless-pepper-service" ]
