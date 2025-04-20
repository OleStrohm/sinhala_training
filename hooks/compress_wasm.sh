for FILE in $TRUNK_STAGING_DIR/*.wasm; do
    brotli -f "$FILE"
done
