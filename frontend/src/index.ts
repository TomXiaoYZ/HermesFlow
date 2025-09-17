import { MovingAverageCrossover } from './strategies/MovingAverageCrossover';
import { MarketData } from './types';

// Example market data
const sampleData: MarketData[] = [
  { timestamp: 1699000000000, open: 100, high: 105, low: 98, close: 103, volume: 1000 },
  { timestamp: 1699000060000, open: 103, high: 107, low: 102, close: 106, volume: 1200 },
  // Add more historical data points here
];

async function main() {
  const strategy = new MovingAverageCrossover();
  const signal = strategy.analyze(sampleData);
  
  console.log('Trading Signal:', signal);
}

main().catch(console.error);