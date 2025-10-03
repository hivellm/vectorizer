#!/bin/bash

# Log cleanup script for Vectorizer
# This script cleans up log files older than 1 day
# Can be run manually or via cron job

set -e

echo "🧹 Starting log cleanup for Vectorizer..."

# Change to the project directory
cd "$(dirname "$0")/.."

# Create .logs directory if it doesn't exist
mkdir -p .logs

# Count files before cleanup
LOG_COUNT_BEFORE=$(find .logs -name "*.log" -type f | wc -l)
echo "📊 Found $LOG_COUNT_BEFORE log files before cleanup"

# Remove log files older than 1 day
DELETED_COUNT=0
while IFS= read -r -d '' file; do
    echo "🗑️  Removing old log file: $file"
    rm -f "$file"
    ((DELETED_COUNT++))
done < <(find .logs -name "*.log" -type f -mtime +1 -print0)

# Count files after cleanup
LOG_COUNT_AFTER=$(find .logs -name "*.log" -type f | wc -l)

echo "✅ Log cleanup completed!"
echo "📈 Summary:"
echo "   - Files before cleanup: $LOG_COUNT_BEFORE"
echo "   - Files deleted: $DELETED_COUNT"
echo "   - Files remaining: $LOG_COUNT_AFTER"

# Show remaining log files
if [ $LOG_COUNT_AFTER -gt 0 ]; then
    echo "📋 Remaining log files:"
    find .logs -name "*.log" -type f -exec ls -lh {} \; | awk '{print "   - " $9 " (" $5 ", modified " $6 " " $7 " " $8 ")"}'
fi

echo "🎉 Log cleanup script finished!"
