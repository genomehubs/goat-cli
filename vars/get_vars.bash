#!/usr/bin/env bash
# Refresh the GoaT field metadata used by build.rs to generate variable_data.rs.
#
# Usage (from anywhere in the repo):
#   ./vars/get_vars.bash
#
# After running, `cargo build` will regenerate the field map automatically.

set -euo pipefail

# Always write into the vars/ directory regardless of where the script is called from.
VARS_DIR="$(cd "$(dirname "$0")" && pwd)"

echo "Fetching taxon field metadata..."
curl -sf -X GET \
  'https://goat.genomehubs.org/api/v2/resultFields?result=taxon&taxonomy=ncbi' \
  -H 'accept: application/json' \
  | python3 -m json.tool \
  > "$VARS_DIR/taxon_vars.json"
echo "  -> $VARS_DIR/taxon_vars.json"

echo "Fetching assembly field metadata..."
curl -sf -X GET \
  'https://goat.genomehubs.org/api/v2/resultFields?result=assembly&taxonomy=ncbi' \
  -H 'accept: application/json' \
  | python3 -m json.tool \
  > "$VARS_DIR/assembly_vars.json"
echo "  -> $VARS_DIR/assembly_vars.json"

echo ""
echo "Done. Run 'cargo build' to regenerate the field map."
