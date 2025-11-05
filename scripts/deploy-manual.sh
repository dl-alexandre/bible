#!/bin/bash
# Manual deployment script for GitHub Pages
set -e

echo "Building site..."
./scripts/build.sh

echo "Checking if gh-pages branch exists..."
if git show-ref --verify --quiet refs/heads/gh-pages; then
    echo "gh-pages branch exists"
else
    echo "Creating gh-pages branch..."
    git checkout --orphan gh-pages
    git rm -rf .
    git checkout master -- out/
    git mv out/* .
    git add .
    git commit -m "Deploy site"
    git push origin gh-pages
    git checkout master
    exit 0
fi

echo "Deploying to gh-pages..."
git checkout gh-pages
git rm -rf . --ignore-unmatch
cp -r out/* .
git add .
git commit -m "Deploy site $(date)" || echo "No changes to deploy"
git push origin gh-pages
git checkout master

echo "✅ Deployed! Site should be live at: https://dl-alexandre.github.io/bible/"
