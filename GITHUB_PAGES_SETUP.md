# GitHub Pages Deployment (5 minutes)

Get your constitutional research system live on GitHub Pages TODAY.

## 1. Generate Analytics (2 min)

```bash
# Install dependencies if not already done
pip install -r requirements.txt

# Generate all analytics data
python scripts/analytics_engine.py

# Verify files were created
ls -la analytics/data/
```

You should see 12 JSON files created.

## 2. Commit & Push (1 min)

```bash
# Stage all files
git add .

# Commit
git commit -m "Deploy to GitHub Pages: analytics and search interface ready"

# Push
git push origin claude/constitutional-research-system-LopyL
```

## 3. Enable GitHub Pages (1 min)

1. Go to your repository on GitHub
2. Click **Settings**
3. Scroll to **Pages** (left sidebar)
4. Under "Source", select:
   - **Deploy from a branch**
   - Branch: `claude/constitutional-research-system-LopyL`
   - Folder: `/ (root)`
5. Click **Save**

GitHub will show you your site URL in ~30 seconds.

## 4. Access Your Site

Your site is now live at:
```
https://timothyhartzog.github.io/Usa-constitution-orginal/
```

### Key URLs

- **Search interface**: `https://timothyhartzog.github.io/Usa-constitution-orginal/frontend/index.html`
- **Analytics dashboard**: `https://timothyhartzog.github.io/Usa-constitution-orginal/analytics/dashboard.html`

## What's Available

✅ Full-text search across 3000+ chunks
✅ Advanced filtering (collection, author, date)
✅ 12+ interactive visualizations
✅ Author comparison
✅ Temporal analysis charts
✅ Document similarity network
✅ Word clouds & heat maps
✅ Download reports (JSON, HTML, PDF)
✅ Mobile responsive
✅ Fast (GitHub CDN)

## Testing

1. **Search interface**: Try searching for "federalism" or "commerce"
2. **Analytics**: View all charts and visualizations
3. **Compare**: See author statistics
4. **Export**: Download JSON/HTML report

## Future: Add Cloudflare + Render

When you're ready (no rush), you can:
1. Enable Cloudflare Pages for even faster CDN
2. Connect Render for live dynamic filtering

Setup guides are already prepared in:
- `DEPLOY_QUICK_START.md` (5-step guide)
- `docs/DEPLOYMENT.md` (detailed guide)

For now, GitHub Pages gives you 95% of the functionality with zero setup.

---

**That's it! Your constitutional research system is live!** 🚀
