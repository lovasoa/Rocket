#!/usr/bin/env bash
set -e

# Brings in _ROOT, _DIR, _DIRS globals.
SCRIPT_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"
source "${SCRIPT_DIR}/config.sh"

# Add Cargo to PATH.
export PATH=${HOME}/.cargo/bin:${PATH}

# Checks that the versions for Cargo projects $@ all match
function check_versions_match() {
  local last_version=""
  for dir in "${@}"; do
    local cargo_toml="${dir}/Cargo.toml"
    if ! [ -f "${cargo_toml}" ]; then
      echo "Cargo configuration file '${cargo_toml}' does not exist."
      exit 1
    fi

    local version=$(grep version "${cargo_toml}" | head -n 1 | cut -d' ' -f3)
    if [ -z "${last_version}" ]; then
      last_version="${version}"
    elif ! [ "${version}" = "${last_version}" ]; then
      echo "Versions differ in '${cargo_toml}'. ${version} != ${last_version}"
      exit 1
    fi
  done
}

# Ensures there are no tabs in any file.
function ensure_tab_free() {
  local tab=$(printf '\t')
  local matches=$(git grep -E -I "${tab}" "${PROJECT_ROOT}" | grep -v 'LICENSE')
  if ! [ -z "${matches}" ]; then
    echo "Tab characters were found in the following:"
    echo "${matches}"
    exit 1
  fi
}

# Ensures there are no files with trailing whitespace.
function ensure_trailing_whitespace_free() {
  local matches=$(git grep -E -I "\s+$" "${PROJECT_ROOT}")
  if ! [ -z "${matches}" ]; then
    echo "Trailing whitespace was found in the following:"
    echo "${matches}"
    exit 1
  fi
}

function bootstrap_examples() {
  while read -r file; do
    bootstrap_script="${file}/bootstrap.sh"
    if [ -x "${bootstrap_script}" ]; then
      echo "    Bootstrapping ${file}..."

      env_vars=$(bash "${bootstrap_script}")
      bootstrap_result=$?
      if [ $bootstrap_result -ne 0 ]; then
        echo "    Running bootstrap script (${bootstrap_script}) failed!"
        exit 1
      else
        eval $env_vars
      fi
    fi
  done < <(find "${EXAMPLES_DIR}" -maxdepth 1 -type d)
}

echo ":: Ensuring all crate versions match..."
check_versions_match "${ALL_PROJECT_DIRS[@]}"

echo ":: Checking for tabs..."
ensure_tab_free

echo ":: Checking for trailing whitespace..."
ensure_trailing_whitespace_free

echo ":: Updating dependencies..."
cargo update

if [ "$1" = "--contrib" ]; then
  FEATURES=(
    json
    msgpack
    tera_templates
    handlebars_templates
    serve
    diesel_postgres_pool
    diesel_sqlite_pool
    diesel_mysql_pool
    postgres_pool
    mysql_pool
    sqlite_pool
    cypher_pool
    redis_pool
  )

  pushd "${CONTRIB_LIB_ROOT}" > /dev/null 2>&1

  echo ":: Building and testing contrib [default]..."
  CARGO_INCREMENTAL=0 cargo test

  for feature in "${FEATURES[@]}"; do
    echo ":: Building and testing contrib [${feature}]..."
    CARGO_INCREMENTAL=0 cargo test --no-default-features --features "${feature}"
  done

  popd > /dev/null 2>&1
else
  echo ":: Bootstrapping examples..."
  bootstrap_examples

  echo ":: Building and testing libraries..."
  CARGO_INCREMENTAL=0 cargo test --all-features --all $@
fi
