#!/bin/bash

# Script to commit and push changes to GitHub

# Check if git is installed
if ! command -v git &> /dev/null; then
    echo "Error: git is not installed. Please install git first."
    exit 1
fi

# Check if we're in a git repository
if ! git rev-parse --is-inside-work-tree &> /dev/null; then
    echo "Error: Not in a git repository. Please run this script from within a git repository."
    exit 1
fi

# Check for uncommitted changes
if [ -z "$(git status --porcelain)" ]; then
    echo "No changes to commit."
    exit 0
fi

# Display current status
echo "Current git status:"
git status

# Ask for confirmation
read -p "Do you want to commit and push these changes? (y/n): " confirm
if [[ $confirm != [yY] && $confirm != [yY][eE][sS] ]]; then
    echo "Operation cancelled."
    exit 0
fi

# Ask for commit message
read -p "Enter commit message: " message
if [ -z "$message" ]; then
    message="Update documentation and clean up references"
fi

# Add all changes
git add .

# Commit changes
git commit -m "$message"

# Check if main branch exists
if git show-ref --verify --quiet refs/heads/main; then
    branch="main"
elif git show-ref --verify --quiet refs/heads/master; then
    branch="master"
else
    # Ask for branch name
    read -p "Enter branch name to push to: " branch
    if [ -z "$branch" ]; then
        echo "No branch specified. Defaulting to 'main'."
        branch="main"
    fi
fi

# Push changes
echo "Pushing to $branch branch..."
git push origin $branch

echo "Done!" 