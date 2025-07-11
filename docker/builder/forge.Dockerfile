### Forge Image ###

FROM debian-base as forge

RUN --mount=type=cache,target=/var/cache/apt,sharing=locked \
    --mount=type=cache,target=/var/lib/apt,sharing=locked \
    apt-get update && apt-get install --no-install-recommends -y \
        awscli \
        busybox \
        git \
        openssh-client \
        unzip \
        wget

WORKDIR /cedra

# copy helm charts from source
COPY --link --from=tools-builder /cedra/terraform/helm /cedra/terraform/helm
COPY --link --from=tools-builder /cedra/testsuite/forge/src/backend/k8s/helm-values/cedra-node-default-values.yaml /cedra/terraform/cedra-node-default-values.yaml

RUN cd /usr/local/bin && wget "https://storage.googleapis.com/kubernetes-release/release/v1.18.6/bin/linux/amd64/kubectl" -O kubectl && chmod +x kubectl
RUN cd /usr/local/bin && wget "https://get.helm.sh/helm-v3.8.0-linux-amd64.tar.gz" -O- | busybox tar -zxvf - && mv linux-amd64/helm . && chmod +x helm
ENV PATH "$PATH:/root/bin"

WORKDIR /cedra
COPY --link --from=node-builder /cedra/dist/forge /usr/local/bin/forge

### Get Cedra Framework Release for forge framework upgrade testing
COPY --link --from=tools-builder /cedra/cedra-move/framework/ /cedra/cedra-move/framework/
COPY --link --from=tools-builder /cedra/cedra-move/cedra-release-builder/ /cedra/cedra-move/cedra-release-builder/

ENV RUST_LOG_FORMAT=json

# add build info
ARG BUILD_DATE
ENV BUILD_DATE ${BUILD_DATE}
ARG GIT_TAG
ENV GIT_TAG ${GIT_TAG}
ARG GIT_BRANCH
ENV GIT_BRANCH ${GIT_BRANCH}
ARG GIT_SHA
ENV GIT_SHA ${GIT_SHA}

ENTRYPOINT ["/tini", "--", "forge"]
