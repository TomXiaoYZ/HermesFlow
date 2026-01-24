"""
Playwright-based Twitter scraper
Core functionality adapted from x-scraper project
"""
import asyncio
import logging
import json
import time
import random
import os
from pathlib import Path
from typing import Dict, List, Optional, Any, Set
from playwright.async_api import async_playwright, Page, Response, Browser, BrowserContext
import boto3
from botocore.exceptions import ClientError

logger = logging.getLogger(__name__)


class PlaywrightScraper:
    """Twitter scraper using Playwright for browser automation"""
    
    def __init__(
        self,
        username: Optional[str] = None,
        password: Optional[str] = None,
        email: Optional[str] = None,
        guest_mode: bool = False,
        headless: bool = True,
        proxy: Optional[str] = None
    ):
        self.username = username
        self.password = password
        self.email = email
        self.guest_mode = guest_mode
        self.headless = headless
        self.proxy = proxy
        
        self.playwright = None
        self.browser: Optional[Browser] = None
        self.context: Optional[BrowserContext] = None
        self.page: Optional[Page] = None
        
        self.scraped_tweet_ids: Set[str] = set()
        self.all_tweets: List[Dict] = []
        self.user_data: Optional[Dict] = None
        
        self.cookies_file = Path("/app/data/playwright_cookies.json")
        self.is_logged_in = False
        
        # S3 Configuration
        self.s3_bucket = "hermesflow-scraper-data"
        self.s3_client = boto3.client('s3', region_name='us-west-2')
        self.cookies_s3_key = f"cookies/{self.username}_cookies.json" if self.username else "cookies/guest_cookies.json"
        
        # Scraping settings
        self.scroll_delay_min = 2.0
        self.scroll_delay_max = 5.0
        self.max_scroll_attempts = 20
        self.max_attempts_without_new = 3
        
        logger.info(f"Initialized PlaywrightScraper (guest_mode={guest_mode})")
    
    async def _load_cookies_from_s3(self) -> Optional[List[Dict]]:
        """Load cookies from S3 bucket"""
        try:
            logger.info(f"Attempting to load cookies from S3: {self.cookies_s3_key}")
            response = self.s3_client.get_object(Bucket=self.s3_bucket, Key=self.cookies_s3_key)
            cookies_data = json.loads(response['Body'].read().decode('utf-8'))
            logger.info("Successfully loaded cookies from S3")
            return cookies_data
        except ClientError as e:
            if e.response['Error']['Code'] == 'NoSuchKey':
                logger.info("No cookies found in S3")
            else:
                logger.warning(f"Error loading cookies from S3: {e}")
        except Exception as e:
            logger.warning(f"Unexpected error loading cookies from S3: {e}")
        return None

    async def _save_cookies_to_s3(self):
        """Save current cookies to S3 bucket"""
        try:
            if not self.context:
                return
            
            cookies = await self.context.cookies()
            logger.info(f"Saving {len(cookies)} cookies to S3: {self.cookies_s3_key}")
            
            self.s3_client.put_object(
                Bucket=self.s3_bucket,
                Key=self.cookies_s3_key,
                Body=json.dumps(cookies),
                ContentType='application/json'
            )
            
            # Also save locally for cache
            self.cookies_file.write_text(json.dumps(cookies))
            logger.info("Cookies saved to S3 and local disk")
            
        except Exception as e:
            logger.error(f"Failed to save cookies to S3: {e}")

    async def _upload_screenshot_to_s3(self, name: str):
        """Take screenshot and upload to S3 for debugging"""
        try:
            if not self.page:
                return
                
            timestamp = int(time.time())
            filename = f"screenshots/{name}_{timestamp}.png"
            local_path = f"/app/data/{name}_{timestamp}.png"
            
            await self.page.screenshot(path=local_path, full_page=True)
            
            with open(local_path, 'rb') as f:
                self.s3_client.put_object(
                    Bucket=self.s3_bucket,
                    Key=filename,
                    Body=f,
                    ContentType='image/png'
                )
            logger.info(f"Screenshot uploaded to s3://{self.s3_bucket}/{filename}")
            
            # Clean up local file
            os.remove(local_path)
            
        except Exception as e:
            logger.error(f"Failed to capture/upload screenshot: {e}")

    async def initialize(self) -> bool:
        """Initialize Playwright browser and context"""
        try:
            logger.info("Starting Playwright initialization...")
            self.playwright = await async_playwright().start()
            logger.info("Playwright started")
            
            browser_args = {
                'headless': self.headless,
                'args': [
                    '--disable-blink-features=AutomationControlled',
                    '--disable-dev-shm-usage',
                    '--no-sandbox',
                    '--disable-setuid-sandbox',
                    '--disable-background-timer-throttling',
                    '--disable-backgrounding-occluded-windows',
                    '--disable-renderer-backgrounding',
                    '--disable-web-security',
                    '--disable-features=IsolateOrigins,site-per-process'
                ]
            }
            
            if self.proxy:
                parts = self.proxy.split(':')
                if len(parts) >= 2:
                    browser_args['proxy'] = {'server': f'http://{parts[0]}:{parts[1]}'}
            
            logger.info("Launching Chromium browser...")
            self.browser = await self.playwright.chromium.launch(**browser_args)
            logger.info(f"Browser launched successfully: {self.browser}")
            
            logger.info("Creating browser context...")
            self.context = await self.browser.new_context(
                viewport={'width': 1920, 'height': 1080},
                user_agent='Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36',
                locale='en-US',
                timezone_id='America/New_York',
                device_scale_factor=1,
                has_touch=False
            )
            
            # Add stealth scripts
            await self.context.add_init_script("""
                Object.defineProperty(navigator, 'webdriver', {
                    get: () => undefined
                });
            """)
            
            logger.info("Browser context created")
            
            # Try to load saved cookies (S3 priority, then local)
            cookies_loaded = False
            if not self.guest_mode:
                # Try S3 first
                cookies_data = await self._load_cookies_from_s3()
                
                # Try local if S3 failed
                if not cookies_data and self.cookies_file.exists():
                    try:
                        cookies_data = json.loads(self.cookies_file.read_text())
                        logger.info("Loaded cookies from local file")
                    except Exception as e:
                        logger.warning(f"Failed to load local cookies: {e}")
                
                if cookies_data:
                    await self.context.add_cookies(cookies_data)
                    self.is_logged_in = True
                    cookies_loaded = True
                    logger.info("Cookies loaded into context")
            
            logger.info("Creating new page...")
            self.page = await self.context.new_page()
            
            # Set default timeout
            self.page.set_default_timeout(30000)
            self.page.set_default_navigation_timeout(30000)
            
            logger.info(f"Page created successfully: {self.page}")
            
            # Set up API response interception
            # Enable for guest mode too, as we need it to capture data
            self.page.on("response", self._intercept_response)
            logger.debug("API response interception enabled")
            
            logger.info("Playwright browser initialized successfully")
            return True
            
        except Exception as e:
            logger.error(f"Failed to initialize Playwright: {e}", exc_info=True)
            return False
            
    async def _human_delay(self, min_seconds=1.0, max_seconds=3.0):
        """Sleep for a random amount of time to simulate human behavior"""
        delay = random.uniform(min_seconds, max_seconds)
        await asyncio.sleep(delay)

    async def login(self) -> bool:
        """Login to Twitter with enhanced robustness"""
        if self.guest_mode or self.is_logged_in:
            return True
        
        # Check credentials exist
        if not self.username or not self.password:
            logger.error(f"Login failed: username={self.username}, password={'set' if self.password else 'missing'}")
            await self._upload_screenshot_to_s3("login_no_credentials")
            return False
        
        try:
            logger.info(f"Starting Twitter login flow for @{self.username}...")
            logger.info(f"Navigating to login page...")
            
            try:
                await self.page.goto('https://twitter.com/login', wait_until='networkidle', timeout=45000)
            except Exception as e:
                logger.warning(f"Navigation timeout, but continuing... error: {e}")
            
            logger.info("Login page loaded (or timeout reached)")
            await self._human_delay(2, 4)
            
            # 1. Username Step
            logger.info("Attempting to find username input...")
            username_selectors = [
                'input[autocomplete="username"]',
                'input[name="text"]',
                'input[type="text"]'
            ]
            
            username_input = None
            for selector in username_selectors:
                try:
                    username_input = await self.page.wait_for_selector(selector, state='visible', timeout=5000)
                    if username_input:
                        logger.info(f"Found username input with selector: {selector}")
                        break
                except:
                    continue
            
            if not username_input:
                logger.error("Could not find username input")
                await self._upload_screenshot_to_s3("login_failed_username")
                return False
                
            await username_input.fill(self.username)
            await self._human_delay(1, 2)
            await self.page.keyboard.press('Enter')
            await self._human_delay(2, 4)
            
            # 2. Email/Phone Verification Step (Optional)
            try:
                logger.info("Checking for verification challenge...")
                # Check for various verification inputs - increased timeout to 10s
                verification_input = await self.page.wait_for_selector(
                    'input[data-testid="ocfEnterTextTextInput"], input[name="text"], input[name="email"], input[name="phoneNumber"]', 
                    state='visible', 
                    timeout=10000
                )
                
                if verification_input:
                    logger.info("Verification challenge detected")
                    # Determine what it's asking for (email or phone)
                    text_content = await self.page.inner_text('body')
                    logger.info(f"Page text (verification step): {text_content[:200]}")
                    
                    if "unusual activity" in text_content or "confirmation code" in text_content:
                        logger.warning("Suspicious login detected by Twitter!")
                        await self._upload_screenshot_to_s3("login_suspicious_challenge")

                    value_to_fill = self.email
                    if "phone" in text_content.lower() and "email" not in text_content.lower():
                        logger.info("Asking strictly for phone. Trying email anyway as fallback...")
                        # If we had a phone number in config, we would use it here
                    
                    logger.info(f"Filling verification with: {value_to_fill}")
                    await verification_input.fill(value_to_fill)
                    await self._human_delay(1, 2)
                    await self.page.keyboard.press('Enter')
                    await self._human_delay(3, 5)
            except Exception:
                logger.info("No verification step detected (or timeout)")
            
            # 3. Password Step
            logger.info("Attempting to find password input...")
            try:
                password_input = await self.page.wait_for_selector(
                    'input[name="password"]', 
                    state='visible', 
                    timeout=30000
                )
                
                if not password_input:
                    logger.error("Could not find password input (returned None)")
                    # Dump page content for debugging
                    content = await self.page.content()
                    logger.info(f"Page content dump: {content[:1000]}...") 
                    await self._upload_screenshot_to_s3("login_failed_password_not_found")
                    return False
                    
                logger.info("Password input found, filling...")
                await password_input.fill(self.password)
                await self._human_delay(1, 2)
                await self.page.keyboard.press('Enter')
                logger.info("Password submitted")
                await self._human_delay(5, 8)
            except Exception as e:
                logger.error(f"Error waiting for password input: {e}")
                # Check if we are still on the previous page or stuck
                content = await self.page.inner_text('body')
                logger.info(f"Visible text content: {content[:500]}...")
                await self._upload_screenshot_to_s3("login_failed_password_exception")
                return False
            
            # 4. Verify Success
            logger.info("Verifying login success...")
            try:
                # Look for home nav or tweet button
                await self.page.wait_for_selector(
                    'nav[role="navigation"], a[data-testid="SideNav_NewTweet_Button"]', 
                    state='visible', 
                    timeout=20000
                )
                logger.info("Login successful!")
                self.is_logged_in = True
                
                # Save cookies
                await self._save_cookies_to_s3()
                return True
                
            except Exception as e:
                logger.error(f"Login verification failed: {e}")
                await self._upload_screenshot_to_s3("login_failed_verification")
                return False
                
        except Exception as e:
            logger.error(f"Unexpected login error: {e}", exc_info=True)
            await self._upload_screenshot_to_s3("login_error_exception")
            return False

    async def _intercept_response(self, response: Response):
        """Intercept Twitter GraphQL API responses"""
        try:
            if response.request.resource_type not in ["xhr", "fetch"]:
                return
            
            url = response.url
            
            # Check if it's a Twitter API endpoint we care about
            if any(endpoint in url for endpoint in [
                'UserByScreenName',
                'UserTweets',
                'TweetDetail',
                'TweetResultByRestId',
                'SearchTimeline',
                'SearchAdaptive'
            ]):
                try:
                    data = await response.json()
                    
                    # Parse based on endpoint type
                    if 'UserByScreenName' in url:
                        self._parse_user_data(data)
                    elif 'UserTweets' in url or 'SearchTimeline' in url or 'SearchAdaptive' in url:
                        self._parse_tweets_from_timeline(data)
                    elif 'TweetResultByRestId' in url or 'TweetDetail' in url:
                        self._parse_single_tweet(data)
                        
                except Exception as e:
                    logger.warning(f"Failed to parse response: {e}")
                    
        except Exception as e:
            logger.debug(f"Error in response interceptor: {e}")
    
    def _parse_user_data(self, data: Dict):
        """Parse user data from GraphQL response"""
        try:
            user_result = data.get('data', {}).get('user', {}).get('result', {})
            if user_result:
                legacy = user_result.get('legacy', {})
                self.user_data = {
                    'id': user_result.get('rest_id', ''),
                    'username': legacy.get('screen_name', ''),
                    'display_name': legacy.get('name', ''),
                    'bio': legacy.get('description', ''),
                    'followers_count': legacy.get('followers_count', 0),
                    'following_count': legacy.get('friends_count', 0),
                    'tweet_count': legacy.get('statuses_count', 0),
                    'verified': user_result.get('is_blue_verified', False) or legacy.get('verified', False),
                    'profile_image_url': legacy.get('profile_image_url_https', ''),
                    'created_at': legacy.get('created_at', ''),
                }
                logger.info(f"Captured user data: @{self.user_data['username']}")
        except Exception as e:
            logger.error(f"Error parsing user data: {e}")
    
    def _parse_tweets_from_timeline(self, data: Dict):
        """Parse tweets from timeline GraphQL response"""
        try:
            # Try different data paths
            instructions = (
                data.get('data', {}).get('user', {}).get('result', {}).get('timeline_v2', {}).get('timeline', {}).get('instructions', []) or
                data.get('data', {}).get('user', {}).get('result', {}).get('timeline', {}).get('timeline', {}).get('instructions', []) or
                data.get('data', {}).get('search_by_raw_query', {}).get('search_timeline', {}).get('timeline', {}).get('instructions', [])
            )
            
            if not instructions:
                return
            
            for instruction in instructions:
                if instruction.get('type') == 'TimelineAddEntries':
                    entries = instruction.get('entries', [])
                    
                    for entry in entries:
                        entry_id = entry.get('entryId', '')
                        
                        # Skip non-tweet entries
                        if any(skip in entry_id for skip in ['cursor-', 'who-to-follow', 'profile-conversation']):
                            continue
                        
                        # Extract tweet from entry
                        tweet_result = entry.get('content', {}).get('itemContent', {}).get('tweet_results', {}).get('result', {})
                        if tweet_result:
                            parsed_tweet = self._extract_tweet_data(tweet_result)
                            if parsed_tweet and parsed_tweet.get('id'):
                                tweet_id = parsed_tweet['id']
                                if tweet_id not in self.scraped_tweet_ids:
                                    self.scraped_tweet_ids.add(tweet_id)
                                    self.all_tweets.append(parsed_tweet)
                                    
        except Exception as e:
            logger.error(f"Error parsing timeline: {e}")

    def _parse_single_tweet(self, data: Dict):
        """Parse single tweet response"""
        try:
            tweet_result = data.get('data', {}).get('tweetResult', {}).get('result', {})
            if tweet_result:
                parsed_tweet = self._extract_tweet_data(tweet_result)
                if parsed_tweet and parsed_tweet.get('id'):
                    tweet_id = parsed_tweet['id']
                    if tweet_id not in self.scraped_tweet_ids:
                        self.scraped_tweet_ids.add(tweet_id)
                        self.all_tweets.append(parsed_tweet)
        except Exception as e:
            logger.error(f"Error parsing single tweet: {e}")

    def _extract_tweet_data(self, tweet_data: Dict) -> Optional[Dict]:
        """Extract relevant data from tweet object"""
        try:
            # Handle potential nested tweet object (retweets etc)
            if 'tweet' in tweet_data:
                tweet_data = tweet_data['tweet']
                
            legacy = tweet_data.get('legacy', {})
            user_data = tweet_data.get('core', {}).get('user_results', {}).get('result', {})
            user_legacy = user_data.get('legacy', {})
            
            if not legacy:
                return None
                
            return {
                'id': legacy.get('id_str') or '',
                'text': legacy.get('full_text', '') or '',
                'created_at': legacy.get('created_at') or '',
                'lang': legacy.get('lang') or 'und',
                'is_retweet': 'retweeted_status_result' in legacy,
                'is_reply': legacy.get('in_reply_to_status_id_str') is not None,
                'metrics': {
                    'retweet_count': legacy.get('retweet_count', 0),
                    'favorite_count': legacy.get('favorite_count', 0),
                    'reply_count': legacy.get('reply_count', 0),
                    'quote_count': legacy.get('quote_count', 0),
                },
                'user': {
                    'id': user_data.get('rest_id') or '',
                    'username': user_legacy.get('screen_name') or '',
                    'display_name': user_legacy.get('name') or '',
                    'followers_count': user_legacy.get('followers_count', 0),
                    'verified': user_data.get('is_blue_verified', False) or user_legacy.get('verified', False),
                },
                'hashtags': [h.get('text') for h in legacy.get('entities', {}).get('hashtags', [])],
                'urls': [u.get('expanded_url') for u in legacy.get('entities', {}).get('urls', [])],
                'media': [
                    {
                        'type': m.get('type'),
                        'url': m.get('media_url_https')
                    }
                    for m in legacy.get('entities', {}).get('media', [])
                ]
            }
        except Exception as e:
            logger.debug(f"Error extracting tweet data: {e}")
            return None

    async def scrape_user_tweets(
        self,
        username: str,
        max_tweets: int = 200
    ) -> Dict[str, Any]:
        """Scrape tweets from a user's timeline"""
        self.all_tweets = []
        self.scraped_tweet_ids = set()
        
        try:
            # Login if needed
            if not self.guest_mode and not self.is_logged_in:
                if not await self.login():
                    return {'error': 'Login failed'}
            
            # Navigate to user profile
            url = f'https://twitter.com/{username}'
            logger.info(f"Navigating to {url}")
            await self.page.goto(url, wait_until='networkidle')
            await asyncio.sleep(3)
            
            # Scroll to load tweets
            scroll_attempts = 0
            attempts_without_new = 0
            last_count = 0
            
            while len(self.all_tweets) < max_tweets and scroll_attempts < self.max_scroll_attempts:
                # Scroll down
                await self.page.evaluate('window.scrollTo(0, document.body.scrollHeight)')
                await self._human_delay(self.scroll_delay_min, self.scroll_delay_max)
                
                current_count = len(self.all_tweets)
                if current_count == last_count:
                    attempts_without_new += 1
                else:
                    attempts_without_new = 0
                    
                if attempts_without_new >= self.max_attempts_without_new:
                    logger.info("No new tweets found after multiple scrolls, stopping")
                    break
                    
                last_count = current_count
                scroll_attempts += 1
                logger.info(f"Scrolled {scroll_attempts}/{self.max_scroll_attempts}, tweets collected: {current_count}")
            
            return {
                'tweet_count': len(self.all_tweets),
                'tweets': self.all_tweets,
                'user_data': self.user_data
            }
            
        except Exception as e:
            logger.error(f"Error scraping user {username}: {e}", exc_info=True)
            await self._upload_screenshot_to_s3(f"error_scrape_user_{username}")
            return {'error': str(e)}

    async def search_tweets(
        self,
        query: str,
        max_tweets: int = 200
    ) -> Dict[str, Any]:
        """Search for tweets"""
        self.all_tweets = []
        self.scraped_tweet_ids = set()
        
        try:
            # Login if needed
            if not self.guest_mode and not self.is_logged_in:
                if not await self.login():
                    return {'error': 'Login failed'}
            
            # Navigate to search page (using f=live for latest)
            url = f'https://twitter.com/search?q={query}&f=live'
            logger.info(f"Navigating to {url}")
            await self.page.goto(url, wait_until='networkidle')
            await asyncio.sleep(3)
            
            # Scroll to load tweets
            scroll_attempts = 0
            attempts_without_new = 0
            last_count = 0
            
            while len(self.all_tweets) < max_tweets and scroll_attempts < self.max_scroll_attempts:
                # Scroll down
                await self.page.evaluate('window.scrollTo(0, document.body.scrollHeight)')
                await self._human_delay(self.scroll_delay_min, self.scroll_delay_max)
                
                current_count = len(self.all_tweets)
                if current_count == last_count:
                    attempts_without_new += 1
                else:
                    attempts_without_new = 0
                    
                if attempts_without_new >= self.max_attempts_without_new:
                    break
                    
                last_count = current_count
                scroll_attempts += 1
                logger.info(f"Scrolled {scroll_attempts}/{self.max_scroll_attempts}, tweets collected: {current_count}")
            
            return {
                'tweet_count': len(self.all_tweets),
                'tweets': self.all_tweets
            }
            
        except Exception as e:
            logger.error(f"Error searching {query}: {e}", exc_info=True)
            await self._upload_screenshot_to_s3(f"error_search_{query}")
            return {'error': str(e)}

    async def close(self):
        """Close browser resources"""
        if self.context:
            await self.context.close()
        if self.browser:
            await self.browser.close()
        if self.playwright:
            await self.playwright.stop()
