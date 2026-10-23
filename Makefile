EXTENSION_NAME = duckdb_cloudeval
TARGET_DUCKDB_VERSION = v2.0.0
DUCKDB_TEST_VERSION := $(patsubst v%,%,$(TARGET_DUCKDB_VERSION))
# duckdb-neo's v2 extension API is pinned to the exact DuckDB build.
USE_UNSTABLE_C_API = 1

include extension-ci-tools/makefiles/c_api_extensions/base.Makefile
include extension-ci-tools/makefiles/c_api_extensions/rust.Makefile

.PHONY: configure debug release
configure: venv platform extension_version
debug: build_extension_library_debug build_extension_with_metadata_debug
release: build_extension_library_release build_extension_with_metadata_release
