#!/bin/bash

find ../src -type f -print0 | while IFS= read -r -d '' file; do
    if syn-tools < "$file" | rustfmt > "$file.tmp"; then
        mv "$file.tmp" "$file"
    else
        echo "Error processing $file"
        rm -f "$file.tmp"
    fi
done
