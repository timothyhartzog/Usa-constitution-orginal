# Deployment Guide: GitHub Pages + Render

This guide covers deploying the Constitutional Research System with:
- **Frontend**: Cloudflare Pages or GitHub Pages (static hosting)
- **Analytics API**: Render (free tier Python deployment)

## Architecture

```
┌─────────────────────────────────┐
│  Cloudflare Pages (FREE)        │  Static files
│  - Search interface             │  - HTML, CSS, JS
│  - Analytics dashboard          │  - Pre-generated JSON
│  - Corpus search index          │
└─────────────────────────────────┘
           ↓
┌─────────────────────────────────┐
│  Render (FREE tier)             │  Dynamic API
│  - Flask analytics server       │  - /api/analytics/*
│  - Dynamic filtering            │  - Author comparison
│  - Live reports                 │  - Temporal analysis
└─────────────────────────────────┘
```

**Features by Deployment:**

| Feature | Static Only | With Render API |
|---------|-------------|-----------------|
| Search interface | ✅ Full | ✅ Full |
| Basic analytics | ✅ View only | ✅ View + dynamic |
| Filtering | ❌ Not available | ✅ Full |
| Author comparison | ❌ Not available | ✅ Full |
| Reports | ✅ Basic export | ✅ Advanced |

---

## Option 1: Cloudflare Pages + Render (RECOMMENDED)

### Prerequisites
- GitHub account
- Cloudflare account (free)
- Render account (free)

### Step 1: Generate Analytics Files

Before deployment, pre-generate all analytics data:

```bash
# Install dependencies
pip install -r requirements.txt

# Generate analytics
python scripts/analytics_engine.py

# Verify analytics files were created
ls -la analytics/data/
```

Files created:
- `corpus_overview.json`
- `clause_analysis.json`
- `issue_analysis.json`
- `collection_analysis.json`
- `author_analysis.json`
- `document_analysis.json`
- `clause_issue_matrix.json`
- `author_clause_matrix.json`
- `word_analysis.json`
- `chunk_size_distribution.json`
- `document_relationships.json`
- `temporal_analysis.json`

### Step 2: Commit Pre-Generated Files

```bash
# Commit analytics data to GitHub
git add analytics/data/*.json
git commit -m "Pre-generate analytics data for static deployment"
git push origin claude/constitutional-research-system-LopyL
```

### Step 3: Deploy Frontend to Cloudflare Pages

1. **Log in to Cloudflare** → https://dash.cloudflare.com
2. **Pages** → **Create a project**
3. **Connect to Git** → Select your GitHub repository
4. **Select repository** → `timothyhartzog/Usa-constitution-orginal`
5. **Select branch** → `claude/constitutional-research-system-LopyL`
6. **Build settings**:
   - Framework: None (static site)
   - Build command: (leave empty)
   - Build output directory: `/`
7. **Save and deploy** → Cloudflare auto-deploys on git push

**Your frontend is now live at**: `https://<project-name>.pages.dev`

### Step 4: Deploy API to Render

1. **Go to Render** → https://render.com
2. **Sign up/Login** → Connect GitHub
3. **New** → **Web Service**
4. **Connect repository** → Select your GitHub repo
5. **Settings**:
   - Name: `constitutional-analytics-api`
   - Environment: `Python 3`
   - Build command: `pip install -r requirements.txt && python scripts/analytics_engine.py`
   - Start command: `gunicorn analytics.api:app --bind 0.0.0.0:$PORT`
6. **Plan**: Free (acceptable for this use case)
7. **Deploy** → Render auto-deploys on git push

**Your API is now live at**: `https://constitutional-analytics-api.onrender.com`

### Step 5: Connect Frontend to API

Update dashboard to use Render API:

```javascript
// In analytics/dashboard.js, update the AnalyticsLoader initialization:
const loader = new AnalyticsLoader(
    'https://constitutional-analytics-api.onrender.com/api',
    '/analytics/data'
);
```

Or update `analytics/dashboard.html` to set the API URL:

```html
<script>
    // Set API URL before loading dashboard.js
    window.ANALYTICS_API_URL = 'https://constitutional-analytics-api.onrender.com/api';
</script>
<script src="dashboard.js"></script>
```

Then in `dashboard.js`:
```javascript
const apiUrl = window.ANALYTICS_API_URL || '/api';
const loader = new AnalyticsLoader(apiUrl, '/analytics/data');
```

### Step 6: Test Full Deployment

1. **Visit search interface**: `https://<cloudflare-project>.pages.dev/frontend/index.html`
2. **Search for terms**: Verify search works
3. **Visit analytics dashboard**: `https://<cloudflare-project>.pages.dev/analytics/dashboard.html`
4. **View charts**: Static analytics should load
5. **Apply filters** (if API connected): Should work
6. **Test author comparison**: Should work with Render API

---

## Option 2: GitHub Pages + Render

Similar to Cloudflare but uses GitHub Pages for static hosting:

### Setup GitHub Pages

1. **Go to repository** → **Settings** → **Pages**
2. **Source**: Deploy from a branch
3. **Branch**: `claude/constitutional-research-system-LopyL` → `/root`
4. **Save**

Your frontend is now at: `https://timothyhartzog.github.io/Usa-constitution-orginal/`

### Deploy Render API (same as above)

Follow Render deployment steps from Option 1, Step 4.

### Connect to API (same as above)

Update dashboard API URL as shown in Option 1, Step 5.

---

## Configuration Options

### Environment Variables for Render

Create `.env` file or set in Render dashboard:

```env
# Analytics configuration
ANALYTICS_DATA_PATH=analytics/data
FLASK_ENV=production
DEBUG=False
```

### Custom API URL in Dashboard

For flexibility, add to `analytics/dashboard.html`:

```html
<script>
    // Automatically detect API URL
    const isLocalhost = window.location.hostname === 'localhost';
    const isCloudflare = window.location.hostname.includes('.pages.dev');
    
    let apiUrl = '/api'; // Default for local development
    
    if (isCloudflare) {
        apiUrl = 'https://constitutional-analytics-api.onrender.com/api';
    }
    
    window.ANALYTICS_API_URL = apiUrl;
</script>
```

---

## Local Development

### Run Search Interface Locally

```bash
# No build needed, just serve static files
python -m http.server 8000

# Visit http://localhost:8000/frontend/index.html
```

### Run Full Stack Locally

```bash
# Terminal 1: Flask API
cd analytics
python api.py

# Terminal 2: Static file server
python -m http.server 8001

# Visit http://localhost:8001/analytics/dashboard.html
```

### Pre-generate Analytics Locally

```bash
# Ensure corpus is available
python scripts/analytics_engine.py

# Check output
ls -la analytics/data/*.json
```

---

## Performance Considerations

### Cloudflare Pages
- **Static files**: Served from global CDN, instant
- **No cold start**: Files always available
- **Bandwidth**: 100GB/month free tier (more than enough)

### Render (Free Tier)
- **Cold start**: ~10-30 seconds after 15 min inactivity
- **CPU**: 0.5 vCPU (adequate for analytics API)
- **Memory**: 512MB (sufficient)
- **Good for**: Hobby projects, research, low traffic

**When to upgrade Render**:
- Traffic >100 concurrent users
- Need instant API response (upgrade to Standard plan: $7/month)
- Want persistent instance (no cold starts)

---

## Troubleshooting

### Dashboard Won't Load

**Symptom**: Blank page or "Cannot load analytics"

**Solutions**:
1. Check browser console for errors (F12 → Console)
2. Verify static JSON files exist: `/analytics/data/*.json`
3. If using API, check Render health: `https://constitutional-analytics-api.onrender.com/api/health`
4. Clear browser cache and reload

### Search Not Working

**Symptom**: Search returns no results

**Solutions**:
1. Verify `data/index/search_index.json` exists
2. Verify `data/chunks/constitution_full_corpus.json` exists
3. Check browser console for fetch errors
4. Ensure files were committed to git

### API Returns 404

**Symptom**: `/api/analytics/*` endpoints not found

**Solutions**:
1. Check Render dashboard for deploy status
2. Verify build command succeeded
3. Check `analytics/api.py` is in root
4. Review Render build logs

### Cold Start Too Slow

**Symptom**: First API request after inactivity takes 30+ seconds

**This is normal for Render free tier.** Options:
1. Wait for first request (subsequent requests are fast)
2. Upgrade to Render Standard ($7/month) for instant response
3. Add keep-alive script that pings API every 10 minutes

---

## Production Checklist

- [ ] Analytics files pre-generated and committed
- [ ] Cloudflare Pages deployed and live
- [ ] Render API deployed and live
- [ ] API URL configured in dashboard
- [ ] All links point to correct domains
- [ ] Search interface works
- [ ] Analytics dashboard loads
- [ ] Filters work (if using Render API)
- [ ] Reports download correctly
- [ ] Mobile responsive (test on phone)
- [ ] HTTPS working (automatic with Cloudflare/Render)
- [ ] Sitemap or search engine submission (optional)

---

## Custom Domain Setup

### Cloudflare Custom Domain

1. Point domain nameservers to Cloudflare
2. Cloudflare Pages → **Custom domain** → Enter domain
3. Follow setup instructions
4. HTTPS auto-configured

### Render Custom Domain

1. Render → Web Service → **Settings** → **Custom domain**
2. Enter API subdomain (e.g., `api.yourdomain.com`)
3. Add CNAME record pointing to Render
4. HTTPS auto-configured

---

## Costs

### Free Tier
- Cloudflare Pages: Free (unlimited bandwidth)
- Render: Free (includes $5 monthly credit, adequate for hobby use)
- **Total: $0/month**

### With Paid Upgrades
- Cloudflare Pages: Included in Cloudflare plan (free tier available)
- Render Standard: $7/month (removes cold starts)
- **Total: $7/month** (optional for faster API)

---

## Auto-Deployment

Both Cloudflare Pages and Render auto-deploy on git push:

1. Make changes locally
2. Commit and push to GitHub
3. Cloudflare Pages redeploys within seconds
4. Render redeploys within 1-2 minutes
5. Changes live automatically

---

## Next Steps

1. Generate analytics: `python scripts/analytics_engine.py`
2. Commit files: `git add analytics/data/` && `git commit -m "..."`
3. Push: `git push origin claude/constitutional-research-system-LopyL`
4. Deploy to Cloudflare Pages (follow Step 3 above)
5. Deploy to Render (follow Step 4 above)
6. Update API URL in dashboard
7. Test all features

**Deployment time: ~10-15 minutes**

---

## Support

For issues:
1. Check Cloudflare Pages dashboard for deploy errors
2. Check Render dashboard for build/runtime logs
3. Review browser console (F12) for client-side errors
4. Check network tab for failed API calls
5. Verify analytics files exist in repository

## See Also

- [Advanced Analytics Guide](ADVANCED_ANALYTICS.md)
- [API Reference](API.md)
- [README](../README.md)
