#!/bin/bash
echo "[MOCK SCRIPT] Dropping and recreating database at: $1"
# In reality, you put your logic here:
# psql $1 -c "DROP SCHEMA public CASCADE; CREATE SCHEMA public;"
sleep 1
