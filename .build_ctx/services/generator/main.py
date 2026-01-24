import os
import time
import json
import redis
import random

# Configuration
REDIS_URL = os.getenv("REDIS_URL", "redis://localhost:6379")
UPDATE_INTERVAL = 60  # seconds

# AlphaGPT Language Specification
# Features (0-5 via vm.rs offset 6): 0:Close, 1:Open, 2:High, 3:Low, 4:Vol, 5:Liq
# Operators (Offset 6): 
# 0:ADD, 1:SUB, 2:MUL, 3:DIV, 4:NEG, 5:ABS, 6:SIGN, 7:GATE, 8:JUMP, 9:DECAY, 10:DELAY1, 11:MAX3
FEATURE_OFFSET = 6

def connect_redis():
    return redis.Redis.from_url(REDIS_URL)

def generate_random_genome():
    """
    Simulates the "Mutation" step of the genetic algorithm.
    Generates a valid Postfix RPN formula.
    """
    # Simple templates:
    # 1. Momentum: Feature - Delay(Feature)
    # 2. Mean Reversion: Mean - Feature
    # 3. Volatility: Abs(Return)
    
    strategies = [
        # Strategy A: Volatility (Original) -> ABS(NormalizedReturn)
        # Tokens: [0, 11] -> 0 (Feat 0), 11 (Op 5 ABS? No, Op 5 is 11)
        # Wait, in vm.rs: op_idx = token - feat_offset (6).
        # ABS is index 5. So Token = 11. Correct.
        {
            "name": "Volatility Breakout (Base)",
            "tokens": [0, 11],
            "desc": "ABS(Return)"
        },
        # Strategy B: Momentum -> Return (No ABS)
        # Tokens: [0] (Just raw return)
        {
            "name": "Pure Momentum",
            "tokens": [0],
            "desc": "Return"
        },
        # Strategy C: Reversion -> NEG(Return) -> -Return
        # NEG is Op 4 -> Token 10
        {
            "name": "Mean Reversion",
            "tokens": [0, 10],
            "desc": "NEG(Return)"
        },
        # Strategy D: Volume Spike -> MUL(Return, Volume)
        # Volume is Feat 4. MUL is Op 2 -> Token 8.
        # Tokens: [0, 4, 8]
        {
            "name": "Volume Weighted Momentum",
            "tokens": [0, 4, 8],
            "desc": "MUL(Return, Volume)"
        }
    ]
    
    return random.choice(strategies)

def main():
    print("AlphaGPT Generator Service 1.0 Starting...", flush=True)
    r = connect_redis()
    
    while True:
        try:
            # 1. Evolve / Select Top Genome
            genome = generate_random_genome()
            
            # 2. Publish to Strategy Engine
            payload = json.dumps({
                "strategy_id": "alphagpt_evo_v1",
                "timestamp": time.time(),
                "formula": genome["tokens"],
                "meta": {
                    "name": genome["name"],
                    "description": genome["desc"]
                }
            })
            
            r.publish("strategy_updates", payload)
            
            # Log for User Visibility (via system logs if possible, or just stdout)
            print(f"[Evolution] New Strategy Deployed: {genome['name']} - {genome['desc']}", flush=True)
            
            # 3. Publish to Strategy Log channel so it shows up in Dashboard too!
            # UI expects: timestamp, strategy_id, symbol, action, message
            log_payload = json.dumps({
                "timestamp": time.strftime("%Y-%m-%dT%H:%M:%SZ", time.gmtime()),
                "strategy_id": "EvolutionaryKernel",
                "symbol": "SYSTEM",
                "action": "Evolving",
                "message": f"Global Strategy Updated to: {genome['name']} ({genome['desc']})"
            })
            r.publish("strategy_logs", log_payload)
            
        except Exception as e:
            print(f"Error in Generator loop: {e}", flush=True)
            
        time.sleep(UPDATE_INTERVAL)

if __name__ == "__main__":
    main()
