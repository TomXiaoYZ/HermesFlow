#!/bin/bash
set -e

echo "🚀 Starting Simplified Stitch MCP Setup..."

# Add gcloud to PATH
export PATH="/opt/homebrew/share/google-cloud-sdk/bin:$PATH"

# 1. Get Project ID
PROJECT_ID=$(gcloud config get-value project 2>/dev/null || echo "")
if [ -z "$PROJECT_ID" ]; then
    echo "📝 Enter your Google Cloud Project ID:"
    read PROJECT_ID
    gcloud config set project "$PROJECT_ID"
fi

echo "✅ Using Project ID: $PROJECT_ID"

# 2. Generate Access Token
echo "🎫 Generating Access Token..."
TOKEN=$(gcloud auth application-default print-access-token)

if [ -z "$TOKEN" ]; then
    echo "❌ Failed to generate token. Please ensure you're authenticated."
    exit 1
fi

# 3. Configure Cursor MCP
echo "⚙️ Configuring .cursor/mcp.json..."
mkdir -p .cursor

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

echo ""
echo "✅ Setup Complete!"
echo "📂 Configuration saved to .cursor/mcp.json"
echo ""
echo "⚠️  IMPORTANT NOTES:"
echo "1. Access token expires in 1 hour. Regenerate by re-running this script."
echo "2. Enable Stitch API manually at:"
echo "   https://console.cloud.google.com/apis/library/stitch.googleapis.com?project=$PROJECT_ID"
echo "3. Restart Cursor/Antigravity to load the new MCP configuration."
echo ""
