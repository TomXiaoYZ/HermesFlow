"""
FastAPI service for Twitter scraping
Provides HTTP endpoints for the Rust data-engine to call
"""
import logging
import os
from fastapi import FastAPI, HTTPException
from fastapi.responses import JSONResponse
from pydantic import BaseModel
from typing import Optional, List
from xscraper.playwright_scraper import PlaywrightScraper

# Configure logging
logging.basicConfig(
    level=logging.INFO,
    format='%(asctime)s - %(name)s - %(levelname)s - %(message)s'
)
logger = logging.getLogger(__name__)

app = FastAPI(
    title="Twitter Scraper Service",
    description="Playwright-based Twitter scraping API",
    version="0.1.0"
)

# Global scraper instance (reused for performance)
scraper_instance: Optional[PlaywrightScraper] = None


class ScrapeUserRequest(BaseModel):
    username: str
    max_tweets: int = 200
    guest_mode: bool = False


class SearchRequest(BaseModel):
    query: str
    max_tweets: int = 200
    guest_mode: bool = False


async def get_scraper(guest_mode: bool = False) -> PlaywrightScraper:
    """Get or create scraper instance"""
    global scraper_instance
    
    # Get credentials from environment
    twitter_username = os.getenv('TWITTER_USERNAME')
    twitter_password = os.getenv('TWITTER_PASSWORD')
    twitter_email = os.getenv('TWITTER_EMAIL')
    
    # Create new scraper if needed
    if scraper_instance is None or scraper_instance.guest_mode != guest_mode:
        if scraper_instance:
            await scraper_instance.close()
        
        scraper_instance = PlaywrightScraper(
            username=twitter_username if not guest_mode else None,
            password=twitter_password if not guest_mode else None,
            email=twitter_email if not guest_mode else None,
            guest_mode=guest_mode,
            headless=True
        )
        
        if not await scraper_instance.initialize():
            raise HTTPException(status_code=500, detail="Failed to initialize scraper")
    
    return scraper_instance


@app.get("/")
async def root():
    """Health check endpoint"""
    return {
        "service": "twitter-scraper",
        "status": "running",
        "version": "0.1.0"
    }


@app.get("/health")
async def health():
    """Detailed health check"""
    return {
        "status": "healthy",
        "scraper_initialized": scraper_instance is not None,
        "scraper_logged_in": scraper_instance.is_logged_in if scraper_instance else False
    }


@app.post("/scrape/user")
async def scrape_user(request: ScrapeUserRequest):
    """
    Scrape tweets from a user's timeline
    
    Example:
        POST /scrape/user
        {
            "username": "elonmusk",
            "max_tweets": 200,
            "guest_mode": false
        }
    """
    try:
        logger.info(f"Scraping user @{request.username} (max={request.max_tweets}, guest={request.guest_mode})")
        
        scraper = await get_scraper(guest_mode=request.guest_mode)
        result = await scraper.scrape_user_tweets(
            username=request.username,
            max_tweets=request.max_tweets
        )
        
        if 'error' in result:
            raise HTTPException(status_code=500, detail=result['error'])
        
        return JSONResponse(content=result)
        
    except HTTPException:
        raise
    except Exception as e:
        logger.error(f"Error scraping user: {e}")
        raise HTTPException(status_code=500, detail=str(e))


@app.post("/scrape/search")
async def scrape_search(request: SearchRequest):
    """
    Search for tweets by query
    
    Example:
        POST /scrape/search
        {
            "query": "spacex",
            "max_tweets": 200,
            "guest_mode": false
        }
    """
    try:
        logger.info(f"Searching for '{request.query}' (max={request.max_tweets}, guest={request.guest_mode})")
        
        scraper = await get_scraper(guest_mode=request.guest_mode)
        result = await scraper.search_tweets(
            query=request.query,
            max_tweets=request.max_tweets
        )
        
        if 'error' in result:
            raise HTTPException(status_code=500, detail=result['error'])
        
        return JSONResponse(content=result)
        
    except HTTPException:
        raise
    except Exception as e:
        logger.error(f"Error searching tweets: {e}")
        raise HTTPException(status_code=500, detail=str(e))


@app.on_event("shutdown")
async def shutdown_event():
    """Cleanup on shutdown"""
    global scraper_instance
    if scraper_instance:
        await scraper_instance.close()
        logger.info("Scraper closed on shutdown")


if __name__ == "__main__":
    import uvicorn
    uvicorn.run(app, host="0.0.0.0", port=8000)
