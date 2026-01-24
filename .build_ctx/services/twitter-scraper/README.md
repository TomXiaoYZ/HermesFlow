# Twitter Scraper Service

Playwright-based Twitter/X scraping microservice.  
Based on [x-scraper](https://github.com/Liaofushen/x-scraper) project.

## Features

- **Playwright-based**: Modern browser automation
- **GraphQL API Interception**: Directly capture Twitter's API responses
- **Guest Mode**: Scrape public tweets without login
- **Authenticated Mode**: Full access with Twitter credentials
- **Cookie Persistence**: Avoid frequent logins
- **FastAPI**: RESTful HTTP API for easy integration

## Quick Start

### Local Development

```bash
# Install dependencies
pip install -r requirements.txt

# Install Playwright browsers
playwright install chromium

# Set environment variables
export TWITTER_USERNAME=your_username
export TWITTER_EMAIL=your_email
export TWITTER_PASSWORD=your_password

# Run the service
python -m uvicorn api:app --reload --port 8000
```

### Docker

```bash
# Build
docker build -t twitter-scraper-service .

# Run
docker run -p 8000:8000 \
  -e TWITTER_USERNAME=your_username \
  -e TWITTER_EMAIL=your_email \
  -e TWITTER_PASSWORD=your_password \
  twitter-scraper-service
```

## API Endpoints

### Health Check
```http
GET /
GET /health
```

### Scrape User Timeline
```http
POST /scrape/user
Content-Type: application/json

{
  "username": "elonmusk",
  "max_tweets": 200,
  "guest_mode": false
}
```

Response:
```json
{
  "username": "elonmusk",
  "tweet_count": 150,
  "tweets": [...],
  "user_data": {...}
}
```

### Search Tweets
```http
POST /scrape/search
Content-Type: application/json

{
  "query": "spacex",
  "max_tweets": 200,
  "guest_mode": false
}
```

## Integration with Rust Data Engine

The Rust data-engine calls this service via HTTP:

```rust
// In Rust data-engine
let url = "http://twitter-scraper:8000/scrape/user";
let response = reqwest::post(url)
    .json(&json!({
        "username": "elonmusk",
        "max_tweets": 200,
        "guest_mode": false
    }))
    .send()
    .await?;
    
let result: TwitterApiResponse = response.json().await?;
```

## Environment Variables

| Variable | Required | Description |
|----------|----------|-------------|
| `TWITTER_USERNAME` | For authenticated mode | Twitter username |
| `TWITTER_EMAIL` | For authenticated mode | Twitter email |
| `TWITTER_PASSWORD` | For authenticated mode | Twitter password |

## Architecture

```
┌─────────────────┐    HTTP POST    ┌──────────────────────┐
│  Data Engine    │─────────────>│  Twitter Scraper     │
│  (Rust)         │<─────────────│  (Python+Playwright) │
│                 │   JSON tweets │                      │
└────────┬────────┘                └──────────┬───────────┘
         │                                    │
         │                                    │ GraphQL
         v                                    │ Intercept
    ┌────────┐                          ┌─────v───┐
    │Postgres│                          │Twitter/X│
    └────────┘                          └─────────┘
```

## Technical Details

### Why This Works

1. **GraphQL API Interception**: Instead of parsing DOM (which changes frequently), we intercept the GraphQL API responses that Twitter's own webpage uses
2. **Modern Browser Automation**: Playwright is actively maintained and has better anti-detection
3. **Cookie Persistence**: Login once, reuse cookies to avoid detection
4. **Proper Anti-Detection**: Disables automation flags, uses realistic user-agent and browser settings

### Key Differences from headless_chrome

| Feature | headless_chrome | Playwright |
|---------|----------------|------------|
| Maintenance | Outdated | Active |
| Anti-Detection | Basic | Advanced |
| API | Limited | Full async |
| Network Interception | Limited | Complete |
