#!/bin/bash
# FastLink Publish Script
# Run this after creating GitHub repository

echo "🚀 FastLink Publish Script"
echo "=========================="

# Check if in correct directory
if [ ! -f "Cargo.toml" ]; then
    echo "�?Error: Not in FastLink directory"
    exit 1
fi

echo "📍 Directory: $(pwd)"

# Remove old origin if exists
echo "🔄 Configuring remote..."
git remote remove origin 2>/dev/null
git remote add origin https://github.com/StarsailsClover/FastLink.git

# Verify remote
echo "🔗 Remote URL:"
git remote -v

# Push main branch
echo ""
echo "📤 Pushing main branch..."
git push -u origin main

# Push tags
echo ""
echo "🏷�? Pushing tags..."
git push origin v26.5-20260603
git push origin v26.5-20260531

echo ""
echo "�?Publish complete!"
echo ""
echo "Next steps:"
echo "1. Visit https://github.com/StarsailsClover/FastLink"
echo "2. Create a new Release with tag v26.5-20260603"
echo "3. See PUBLISH_GUIDE.md for detailed instructions"
