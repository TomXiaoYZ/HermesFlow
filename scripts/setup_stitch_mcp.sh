#!/bin/bash
set -e

echo "🚀 Starting Stitch MCP Setup..."

# 1. Check gcloud installation
# Try to find gcloud in common locations if not in PATH
if ! command -v gcloud &> /dev/null; then
    if [ -f "/opt/homebrew/share/google-cloud-sdk/bin/gcloud" ]; then
        export PATH="/opt/homebrew/share/google-cloud-sdk/bin:$PATH"
    elif [ -f "/usr/local/share/google-cloud-sdk/bin/gcloud" ]; then
        export PATH="/usr/local/share/google-cloud-sdk/bin:$PATH"
    fi
fi

if ! command -v gcloud &> /dev/null; then
    echo "❌ gcloud not found. Please restart your terminal or add google-cloud-sdk/bin to your PATH."
    exit 1
fi


echo "✅ gcloud found."

# 2. Authentication
echo "🔑 Please authenticate with Google Cloud..."
echo "Running: gcloud auth login"
gcloud auth login

echo "🔑 Setting up Application Default Credentials..."
echo "Running: gcloud auth application-default login"
gcloud auth application-default login

# 3. Project Configuration
echo "📝 Enter your Google Cloud Project ID (e.g. my-project-123):"
read PROJECT_ID

if [ -z "$PROJECT_ID" ]; then
    echo "❌ Project ID cannot be empty."
    exit 1
fi

echo "✅ Using Project ID: $PROJECT_ID"
gcloud config set project "$PROJECT_ID"

echo "🔌 Enabling Stitch API (stitch.googleapis.com)..."
gcloud beta services mcp enable stitch.googleapis.com --project="$PROJECT_ID"

USER_EMAIL=$(gcloud config get-value account)
echo "👤 Adding IAM binding for $USER_EMAIL..."
gcloud projects add-iam-policy-binding "$PROJECT_ID" \
  --member="user:$USER_EMAIL" \
  --role="roles/serviceusage.serviceUsageConsumer"

# 4. Generate Access Token
echo "🎫 Generating Access Token..."
TOKEN=$(gcloud auth application-default print-access-token)

# 5. Configure Cursor MCP
echo "⚙️ Configuring .cursor/mcp.json..."
mkdir -p .cursor

cat > .cursor/mcp.json <<EOF
{
  "mcpServers": {
    "stitch": {
      "command": "npx",
      "args": ["-y", "@google/mcp-stitch"],
      "env": {
        "STITCH_ACCESS_TOKEN": "$TOKEN",
        "GOOGLE_CLOUD_PROJECT": "$PROJECT_ID"
      },
      "url": "https://stitch.googleapis.com/mcp", 
      "headers": {
        "Authorization": "Bearer $TOKEN",
        "x-goog-user-project": "$PROJECT_ID"
      }
    }
  }
}
EOF

# Note: The guide specified simple JSON config for Cursor.
# But generally MCP servers run as local processes or connect remotely.
# The guide snippet for Cursor was:
# {
#   "mcpServers": {
#     "stitch": {
#       "url": "https://stitch.googleapis.com/mcp",
#       "headers": { ... }
#     }
#   }
# }
# Cursor usually supports `command` (stdio) or SSE (url). If URL is provided, it might be SSE.
# Let's stick strictly to the guide's JSON format.

cat > .cursor/mcp.json <<EOF
{
  "mcpServers": {
    "stitch": {
      "url": "https://stitch.googleapis.com/mcp",
      "headers": {
        "Authorization": "Bearer $TOKEN",
        "x-goog-user-project": "$PROJECT_ID"
      }
    }
  }
}
EOF

echo "✅ Setup Complete!"
echo "📂 Configuration saved to .cursor/mcp.json"
echo "⚠️  Note: The access token expires in 1 hour. You may need to regenerate it."
