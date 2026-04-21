# Docker Hub Setup Guide

This guide explains how to configure your Docker Hub repository with README and icon.

## 📝 Adding README to Docker Hub

### Method 1: Manual Upload (Recommended)

1. **Login to Docker Hub**: https://hub.docker.com
2. **Navigate to your repository**: `https://hub.docker.com/r/USERNAME/vectorizer`
3. **Click "Edit"** button (top right)
4. **Go to "Full Description"** tab
5. **Copy content from `dockerhub-readme.md`** (replace `USERNAME` with your Docker Hub username)
6. **Paste and save**

### Method 2: Using Docker Hub API

```bash
# Login first
docker login

# Update README via API (requires Docker Hub token)
curl -X PATCH \
  https://hub.docker.com/v2/repositories/USERNAME/vectorizer/ \
  -H "Authorization: JWT YOUR_DOCKERHUB_TOKEN" \
  -H "Content-Type: application/json" \
  -d @- << EOF
{
  "full_description": "$(cat dockerhub-readme.md | jq -Rs .)"
}
EOF
```

## 🎨 Adding Icon to Docker Hub

### Method 1: Via Web Interface (Recommended)

1. **Login to Docker Hub**: https://hub.docker.com
2. **Navigate to your repository**: `https://hub.docker.com/r/USERNAME/vectorizer`
3. **Click "Settings"** (gear icon)
4. **Scroll to "Repository Icon"** section
5. **Click "Upload Icon"**
6. **Select `assets/icon.png`** (or convert `assets/icon.ico` to PNG)
7. **Save changes**

**Icon Requirements:**
- Format: PNG or JPG
- Size: 200x200 pixels (recommended)
- Max file size: 1MB

### Method 2: Convert ICO to PNG

If you only have `icon.ico`, convert it to PNG:

```bash
# Using ImageMagick
magick convert assets/icon.ico -resize 200x200 assets/icon-dockerhub.png

# Using ffmpeg
ffmpeg -i assets/icon.ico -vf scale=200:200 assets/icon-dockerhub.png
```

## 🔧 Updating Dockerfile Labels

The Dockerfile already includes OpenContainer labels. To add Docker Hub specific labels:

```dockerfile
LABEL org.opencontainers.image.title="Vectorizer"
LABEL org.opencontainers.image.description="Official Vectorizer image - High-Performance Vector Database"
LABEL org.opencontainers.image.url="https://github.com/hivellm/vectorizer"
LABEL org.opencontainers.image.documentation="https://github.com/hivellm/vectorizer/docs"
LABEL org.opencontainers.image.source="https://github.com/hivellm/vectorizer"
LABEL org.opencontainers.image.vendor="HiveLLM"
```

## 📋 Checklist

- [ ] README added to Docker Hub (Full Description)
- [ ] Icon uploaded to Docker Hub (200x200 PNG)
- [ ] Replace `USERNAME` in `dockerhub-readme.md` with your Docker Hub username
- [ ] Test Docker pull: `docker pull USERNAME/vectorizer:latest`
- [ ] Verify README displays correctly on Docker Hub page

## 🔗 Useful Links

- **Docker Hub**: https://hub.docker.com
- **Docker Hub API**: https://docs.docker.com/docker-hub/api/
- **Repository Settings**: https://hub.docker.com/r/USERNAME/vectorizer/settings

