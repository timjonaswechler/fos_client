release version:
    ./scripts/release.sh {{ version }}

build profile="debug":
    @if [ "{{ profile }}" = "release" ]; then \
      STEAM_APP_ID=480 cargo build --release; \
    else \
      STEAM_APP_ID=480 cargo build; \
    fi

run profile="debug":
    @if [ "{{ profile }}" = "release" ]; then \
      STEAM_APP_ID=480 cargo run --release; \
    else \
      STEAM_APP_ID=480 cargo run; \
    fi

[group('dev')]
test:
    cargo test

[group('dev')]
clean:
    cargo clean

[group('dev')]
api type="bin":
    @if [ "{{ type }}" = "bin" ]; then \
      just api-bin; \
    else \
      just api-other "{{ type }}"; \
    fi

_api-bin:
    cargo modules structure --bin client

_api-other type:
    cargo modules structure --{{ type }}
