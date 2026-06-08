# Quick Start: Deploy to GitHub Pages (Cloudflare) + Render

**Total time: ~15 minutes**

## Step 1: Prepare Analytics (2 min)

```bash
# Install dependencies
pip install -r requirements.txt

# Generate analytics files
python scripts/prepare_deployment.py

# This will:
# - Generate all analytics data
# - Verify all files exist
# - Show deployment instructions
```

## Step 2: Commit & Push (1 min)

```bash
git add .
git commit -m "Prepare deployment"
git push origin claude/constitutional-research-system-LopyL
```

## Step 3: Deploy Frontend to Cloudflare Pages (5 min)

1. Go to https://dash.cloudflare.com
2. **Pages** → **Create Project**
3. **Connect to Git** → Select repository
4. **Select branch**: `claude/constitutional-research-system-LopyL`
5. **Build settings**:
   - Framework: `None`
   - Build command: (leave empty)
   - Output directory: `/`
6. **Save and deploy**

**Your site is live at**: `https://<project>.pages.dev`

Test it:
- Search interface: `https://<project>.pages.dev/frontend/index.html`
- Analytics dashboard: `https://<project>.pages.dev/analytics/dashboard.html`

## Step 4: Deploy API to Render (5 min)

1. Go to https://render.com
2. **New** → **Web Service**
3. **Connect GitHub repository**
4. **Settings**:
   - Name: `constitutional-analytics-api`
   - Environment: `Python 3`
   - Build: `pip install -r requirements.txt && python scripts/analytics_engine.py`
   - Start: `gunicorn analytics.api:app --bind 0.0.0.0:$PORT`
5. **Plan**: Free
6. **Deploy**

**Your API is live at**: `https://constitutional-analytics-api.onrender.com`

## Step 5: Connect Dashboard to API (2 min)

Edit `analytics/dashboard.html` and update the API URL:

```html
<script>
    window.ANALYTICS_API_URL = 'https://constitutional-analytics-api.onrender.com/api';
</script>
```

Push the change:
```bash
git add analytics/dashboard.html
git commit -m "Update API URL for Render"
git push
```

Cloudflare Pages auto-deploys within seconds.

## Done! 🚀

Your constitutional research system is now live:
- **Search interface**: `https://<cloudflare>.pages.dev/frontend/index.html`
- **Analytics dashboard**: `https://<cloudflare>.pages.dev/analytics/dashboard.html`
- **API**: `https://constitutional-analytics-api.onrender.com/api`

### Features Available:

✅ Full-text search of corpus
✅ Analytics dashboard with charts
✅ Dynamic filtering (by collection, author, date)
✅ Author comparison
✅ Temporal analysis
✅ Download reports (JSON, HTML, PDF)
✅ Mobile responsive
✅ HTTPS everywhere

## Costs

- **Cloudflare Pages**: Free (unlimited bandwidth)
- **Render API**: Free (cold starts after 15 min inactivity)
- **Total**: $0/month

**Optional upgrade** to Render Standard ($7/mo) removes cold starts if you want instant API response.

## What's Pre-Generated

Analytics data is pre-generated before deployment:
- Corpus overview statistics
- Top clauses and issues
- Author analysis
- Document relationships
- Temporal analysis
- Collection breakdown
- Word frequency
- Chunk size distribution

These static files load instantly on Cloudflare. When Render API is connected, you can also:
- Filter by any combination of criteria
- Compare any two authors
- Generate live reports

## Troubleshooting

**Search not working?**
- Ensure `data/chunks/constitution_full_corpus.json` exists
- Check browser console (F12) for errors
- Clear cache and reload

**Analytics not loading?**
- Ensure `analytics/data/*.json` files exist
- Check browser console for errors
- If using API, wait for first cold start (~30s)

**API not responding?**
- Check Render dashboard for deploy status
- Wait for Render cold start (first request takes 10-30s)
- Test health endpoint: `https://api-url/api/health`

## See Also

- **Full deployment guide**: [docs/DEPLOYMENT.md](docs/DEPLOYMENT.md)
- **Advanced analytics**: [docs/ADVANCED_ANALYTICS.md](docs/ADVANCED_ANALYTICS.md)
- **API reference**: [docs/API.md](docs/API.md)
- **Corpus info**: [docs/SOURCES.md](docs/SOURCES.md)

## Next Steps

### Add Custom Domain (Optional)
1. Cloudflare: Add domain to Cloudflare, point nameservers
2. Render: Add CNAME for API subdomain
3. Both auto-configure HTTPS

### Monitor Performance (Optional)
- Cloudflare Analytics: Built into dashboard
- Render Metrics: Monitor API usage in dashboard
- Set up error alerts (both platforms support this)

### Scale Up (Optional)
- Upgrade Render to Standard ($7/mo) for instant API
- Add more collections/documents to corpus
- Add custom indexes or search features

---

**Questions?** Check [DEPLOYMENT.md](docs/DEPLOYMENT.md) for detailed instructions.
