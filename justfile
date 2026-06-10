set shell := ["bash", "-euo", "pipefail", "-c"]
#!/bin/bash

docker_image := env_var_or_default('DOCKER_IMAGE', 'cw-infuser-optimizer:0.17.0')
arch := `if [ "$(uname -m)" = "arm64" ] || [ "$(uname -m)" = "aarch64" ]; then echo "linux/arm64"; else echo "linux/amd64"; fi`


optimizer-build:
        docker build -t {{docker_image}} optimizer/

workspace-optimize: optimizer-build
	docker run --rm \
		-v $(pwd)/..:/workspace \
		--env PROJECT_DIR=$$(basename $$(pwd)) \
		--mount type=volume,source=terp_optimizer_cache,target=/target \
		--mount type=volume,source=registry_cache,target=/usr/local/cargo/registry \
		--platform {{arch}} \
		{{docker_image}}

# Clear build caches (useful after toolchain changes or if builds fail)
optimizer-clean:
        docker volume rm terp_optimizer_cache 2>/dev/null || true%          

# Sync vendored external contract schemas from their source repos
sync-vendor:
    #!/bin/bash
    set -euo pipefail
    MANIFEST="vendor/external-schemas.json"
    if [ ! -f "$MANIFEST" ]; then
        echo "No vendor manifest found at $MANIFEST"
        exit 1
    fi
    COUNT=$(python3 -c "import json,sys; d=json.load(open('$MANIFEST')); print(len(d['schemas']))")
    echo "📦 Syncing $COUNT external schema(s)..."
    for i in $(seq 0 $((COUNT - 1))); do
        NAME=$(python3 -c "import json; d=json.load(open('$MANIFEST')); print(d['schemas'][$i]['name'])")
        SRC=$(python3 -c "import json; d=json.load(open('$MANIFEST')); print(d['schemas'][$i]['source'])")
        DEST=$(python3 -c "import json; d=json.load(open('$MANIFEST')); print(d['schemas'][$i]['dest'])")
        if [ ! -f "$SRC" ]; then
            echo "  ⚠ Source not found for $NAME: $SRC"
            continue
        fi
        mkdir -p "$(dirname "$DEST")"
        cp "$SRC" "$DEST"
        echo "  ✅ $NAME → $DEST"
    done

# Full pipeline: sync vendor schemas, generate Rust schemas, ts-codegen + bundle
schema-codegen: sync-vendor
    #!/bin/bash
    set -euo pipefail
    sh scripts/sh/schema-and-codegen.sh

# Only run ts-codegen + bundle (skip Rust schema generation)
codegen: sync-vendor
    #!/bin/bash
    set -euo pipefail
    echo "📝 Generating contracts config..."
    node scripts/ts/gen-contracts-config.js
    echo "⚙️  Running ts-codegen + esbuild..."
    cd scripts/ts && yarn codegen

deploy:
    #!/bin/bash
    cargo run --bin deploy

create-merkle:
    @cargo run --bin merkle -- -i $1 --proofs >> $2

coverage:
    @cargo carpulin -p test-suite >> carp.json
