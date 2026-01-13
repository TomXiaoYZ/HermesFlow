import asyncio
import json
import os
import sys
import boto3
from playwright.async_api import async_playwright

# Configuration
S3_BUCKET = "hermesflow-scraper-data"
REGION_NAME = "us-west-2"

async def main():
    print("🚀 Starting Twitter Cookie Generator")
    print(f"Target S3 Bucket: {S3_BUCKET}")
    
    # Get credentials from user if not in env
    username = os.getenv("TWITTER_USERNAME")
    if not username:
        username = input("Enter Twitter Username: ")
    
    print(f"Generating cookies for: {username}")
    
    async with async_playwright() as p:
        # Launch headed browser
        print("Launching browser... (Please interact with the window)")
        browser = await p.chromium.launch(headless=False)
        context = await browser.new_context(
            viewport={'width': 1280, 'height': 720}
        )
        page = await context.new_page()
        
        # Go to login
        print("Navigating to https://twitter.com/login ...")
        await page.goto("https://twitter.com/login")
        
        print("\n" + "="*50)
        print("⚡️ PLEASE LOG IN MANUALLY IN THE BROWSER WINDOW")
        print("⚡️ Once you are on the home timeline (twitter.com/home), press ENTER in this terminal.")
        print("="*50 + "\n")
        
        input("Press ENTER after successful login...")
        
        # Get cookies
        cookies = await context.cookies()
        print(f"Captured {len(cookies)} cookies.")
        
        # Save locally
        local_filename = f"{username}_cookies.json"
        with open(local_filename, "w") as f:
            json.dump(cookies, f, indent=2)
        print(f"Saved locally to: {local_filename}")
        
        # Upload to S3
        s3_key = f"cookies/{username}_cookies.json"
        print(f"Uploading to s3://{S3_BUCKET}/{s3_key} ...")
        
        try:
            s3_client = boto3.client('s3', region_name=REGION_NAME)
            s3_client.put_object(
                Bucket=S3_BUCKET,
                Key=s3_key,
                Body=json.dumps(cookies),
                ContentType='application/json'
            )
            print("✅ Successfully uploaded cookies to S3!")
        except Exception as e:
            print(f"❌ Failed to upload to S3: {e}")
            print("Please ensure you have AWS credentials configured.")
            
        await browser.close()

if __name__ == "__main__":
    try:
        asyncio.run(main())
    except KeyboardInterrupt:
        print("\nCancelled.")
