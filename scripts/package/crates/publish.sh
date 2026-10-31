#!/usr/bin/env bash
# crates.io release channel for the `context-kernel` CLI and its internal closure.
#
#   publish.sh --check
#       Validates the publishable set (names, metadata, lockstep pins, order, no
#       unrelated publishable crates), then packages every crate and dry-runs the
#       publication. Needs no credentials and publishes nothing.
#
#   publish.sh --publish
#       Publishes missing crates in dependency order. Idempotent: a crate whose
#       exact version is already on crates.io is skipped, and its published
#       checksum must equal this commit's package or the run fails. Writes one
#       receipt per crate to $CK_CRATE_RECEIPTS (default dist/crates-receipts).
#       Requires CARGO_REGISTRY_TOKEN in the environment; the token is never printed.
set -euo pipefail

root="$(cd "$(dirname "${BASH_SOURCE[0]}")/../../.." && pwd)"
cd "$root"
mode="${1:?usage: publish.sh --check|--publish}"
version="$(bash scripts/package/version.sh)"
order_file="scripts/package/crates/publish-order.txt"
receipt_dir="${CK_CRATE_RECEIPTS:-$root/dist/crates-receipts}"
index_url="${CK_CRATES_INDEX:-https://index.crates.io}"

log() { printf '%s\n' "$*" >&2; }
die() { log "error: $*"; exit 1; }

# ---- 1. Validate the publishable set --------------------------------------
metadata="$(cargo metadata --format-version 1 --locked)"

order="$(grep -vE '^\s*(#|$)' "$order_file")"
closure="$(jq -r '
  (.packages | map({key: .name, value: [.dependencies[] | select(.path != null and .kind != "dev") | .name]}) | from_entries) as $g
  | def clo($n): [$n] + ([$g[$n][]?] | map(clo(.)) | add // []);
  clo("context-kernel") | unique[]' <<<"$metadata")"

if [[ "$(sort <<<"$order")" != "$(sort <<<"$closure")" ]]; then
  log "order file:"; sort <<<"$order" >&2
  log "closure:";    sort <<<"$closure" >&2
  die "publish-order.txt does not equal the closure of context-kernel"
fi

order_json="$(jq -R . <<<"$order" | jq -s .)"
violations="$(jq -r --arg v "$version" --argjson o "$order_json" '
  . as $m
  | ($m.packages | map({key: .name, value: .}) | from_entries) as $p
  | def pos($x): ([range(0; $o | length) | select($o[.] == $x)] | first // null);
  $o[] as $n
  | ($p[$n]) as $pk
  | if $pk == null then "missing package \($n)"
    else
      (if $pk.publish != null then "\($n) is publish=false" else empty end),
      (if $pk.version != $v then "\($n) version \($pk.version) != \($v)" else empty end),
      (if ($pk.description // "") == "" then "\($n) has no description" else empty end),
      (if $pk.license != "Apache-2.0" then "\($n) license is not Apache-2.0" else empty end),
      (if ($pk.repository // "") == "" then "\($n) has no repository" else empty end),
      ( $pk.dependencies[] | select(.path != null and .kind != "dev") as $d
        | if $d.req != "=\($v)" then "\($n) dep \($d.name) is not pinned to =\($v) (req \($d.req))"
          elif pos($d.name) == null then "\($n) depends on \($d.name) outside the publish set"
          elif pos($d.name) >= pos($n) then "\($n) depends on \($d.name), which is published later"
          else empty end)
    end' <<<"$metadata")"
if [[ -n "$violations" ]]; then
  printf '%s\n' "$violations" >&2
  die "publication metadata validation failed"
fi

# Every publishable workspace member must belong to the closure.
unrelated="$(jq -r --argjson o "$(jq -R . <<<"$order" | jq -s .)" '
  .workspace_members as $w | .packages[] | select(.id as $i | $w | index($i))
  | select(.publish == null) | select(.name as $n | $o | index($n) | not) | .name' <<<"$metadata")"
[[ -z "$unrelated" ]] || die "publishable crates outside the closure (set publish = false): $unrelated"

log "validated: $(wc -l <<<"$order" | tr -d ' ') crates at $version, order, pins, metadata"

# ---- helpers ----------------------------------------------------------------
index_path() {
  local n="$1"
  case ${#n} in
    1) echo "1/$n" ;;
    2) echo "2/$n" ;;
    3) echo "3/${n:0:1}/$n" ;;
    *) echo "${n:0:2}/${n:2:2}/$n" ;;
  esac
}

# Prints the index cksum for name@version, or nothing when the version is absent.
index_cksum() {
  local name="$1" ver="$2"
  curl -fsS --retry 3 "$index_url/$(index_path "$name")" 2>/dev/null \
    | jq -r --arg v "$ver" 'select(.vers == $v) | .cksum' | head -1
}

# Hash of a .crate's contents, excluding .cargo_vcs_info.json (which records the commit).
content_hash() {
  local crate="$1" tmp
  tmp="$(mktemp -d)"
  tar -xzf "$crate" -C "$tmp"
  (cd "$tmp" && find . -type f ! -name .cargo_vcs_info.json -print0 | sort -z | xargs -0 sha256sum | sha256sum | cut -d' ' -f1)
  rm -rf "$tmp"
}

write_receipt() { # name status sha256 detail
  mkdir -p "$receipt_dir"
  jq -n --arg channel "crates.io" --arg package "$1" --arg version "$version" \
    --arg status "$2" --arg sha256 "$3" --arg detail "$4" \
    --arg commit "${GITHUB_SHA:-$(git rev-parse HEAD)}" \
    --arg run "${GITHUB_RUN_ID:-local}" --arg at "$(date -u +%Y-%m-%dT%H:%M:%SZ)" \
    '{channel:$channel, package:$package, version:$version, status:$status,
      sha256:$sha256, detail:$detail, commit:$commit, run:$run, at:$at}' \
    > "$receipt_dir/$1-$version.json"
}

# ---- 2. Check mode -----------------------------------------------------------
if [[ "$mode" == "--check" ]]; then
  # Workspace dry run: cargo packages and verifies each publishable crate in
  # dependency order, resolving dependents against the packaged dependencies. Crates
  # with publish = false are excluded automatically. Per-crate `cargo package` cannot
  # run for dependents before their dependencies exist on crates.io.
  cargo publish --workspace --dry-run --locked --allow-dirty -j 2 >&2 \
    || die "cargo publish --workspace --dry-run failed"
  log "check passed: all crates packaged and dry-run"
  exit 0
fi

# ---- 3. Publish mode ---------------------------------------------------------
[[ "$mode" == "--publish" ]] || die "unknown mode: $mode"
publish_log="$(mktemp)"
trap 'rm -f "$publish_log"' EXIT
: "${CARGO_REGISTRY_TOKEN:?CARGO_REGISTRY_TOKEN must be set (from the org secret)}"

wait_for_index() {
  local name="$1"
  for _ in $(seq 1 60); do
    if [[ -n "$(index_cksum "$name" "$version")" ]]; then return 0; fi
    sleep 10
  done
  return 1
}

while IFS= read -r name; do
  published_sum="$(index_cksum "$name" "$version" || true)"
  cargo package --locked --no-verify --allow-dirty -p "$name" >/dev/null
  local_sum="$(sha256sum "target/package/$name-$version.crate" | cut -d' ' -f1)"

  if [[ -n "$published_sum" ]]; then
    # cargo embeds the commit in .cargo_vcs_info.json, so the .crate checksum changes
    # with every commit. Compare the crate contents without that file instead.
    remote_crate="$(mktemp)"
    curl -fsSL --retry 3 "https://static.crates.io/crates/$name/$name-$version.crate" -o "$remote_crate" \
      || die "could not download published $name@$version to compare"
    if [[ "$(content_hash "$remote_crate")" != "$(content_hash "target/package/$name-$version.crate")" ]]; then
      rm -f "$remote_crate"
      write_receipt "$name" "conflict" "$local_sum" "published $name@$version has different source contents"
      die "$name@$version is already published with different contents; refusing to continue"
    fi
    rm -f "$remote_crate"
    write_receipt "$name" "already_published" "$local_sum" "published contents match this commit (vcs info excluded)"
    log "skip $name: already published with identical contents"
    continue
  fi

  # crates.io limits how many new crates an account may create in a short window
  # and answers 429 with "try again after <GMT date>". Wait for that time and retry
  # the same crate; any other failure stops the run.
  attempt=0
  while true; do
    attempt=$((attempt + 1))
    if cargo publish --locked --allow-dirty -p "$name" > "$publish_log" 2>&1; then
      cat "$publish_log" >&2
      break
    fi
    cat "$publish_log" >&2
    retry_at="$(grep -o 'try again after [A-Za-z]*, [0-9]* [A-Za-z]* [0-9]* [0-9:]* GMT' "$publish_log" | tail -1 | sed 's/^try again after //' || true)"
    if [[ -n "$retry_at" && "$attempt" -lt 30 ]]; then
      wait_s=$(( $(date -u -d "$retry_at" +%s) - $(date -u +%s) + 20 ))
      (( wait_s < 20 )) && wait_s=20
      log "crates.io rate limit on $name; waiting ${wait_s}s (until $retry_at), attempt $attempt"
      sleep "$wait_s"
      continue
    fi
    write_receipt "$name" "failed" "$local_sum" "cargo publish exited non-zero after $attempt attempt(s)"
    die "publishing $name@$version failed; earlier crates in the order may already be live (see receipts)"
  done
  if wait_for_index "$name"; then
    write_receipt "$name" "published" "$local_sum" "visible in index"
    log "published $name@$version"
  else
    write_receipt "$name" "unverified" "$local_sum" "uploaded but not visible in index after 10 minutes"
    die "$name@$version uploaded but not visible in the index"
  fi
done <<<"$order"

log "crates.io channel complete for $version"
