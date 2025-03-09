#!/bin/bash

# Script to commit changes and push to both develop and main branches

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
read -p "Do you want to commit and push these changes to both develop and main branches? (y/n): " confirm
if [[ $confirm != [yY] && $confirm != [yY][eE][sS] ]]; then
    echo "Operation cancelled."
    exit 0
fi

# Ask for commit message
read -p "Enter commit message: " message
if [ -z "$message" ]; then
    message="Update documentation and clean up references"
fi

# Store the current branch
current_branch=$(git symbolic-ref --short HEAD)
echo "Current branch: $current_branch"

# Add all changes
git add .

# Commit changes
git commit -m "$message"

# Push to develop branch first
if git show-ref --verify --quiet refs/heads/develop; then
    echo "Pushing to develop branch..."
    
    # If we're not already on develop, switch to it
    if [ "$current_branch" != "develop" ]; then
        git checkout develop
    fi
    
    # Pull latest changes from develop to avoid conflicts
    git pull origin develop
    
    # Push to develop
    git push origin develop
    
    echo "Successfully pushed to develop branch."
else
    echo "Warning: develop branch not found. Skipping push to develop."
fi

# Push to main branch
if git show-ref --verify --quiet refs/heads/main; then
    echo "Pushing to main branch..."
    
    # Switch to main branch
    git checkout main
    
    # Pull latest changes from main to avoid conflicts
    git pull origin main
    
    # Merge changes from develop
    git merge develop
    
    # Push to main
    git push origin main
    
    echo "Successfully pushed to main branch."
else
    echo "Warning: main branch not found. Skipping push to main."
fi

# Return to the original branch
if [ "$current_branch" != "$(git symbolic-ref --short HEAD)" ]; then
    git checkout "$current_branch"
    echo "Returned to $current_branch branch."
fi

echo "Done!" 