export interface MarketData {
  timestamp: number;
  open: number;
  high: number;
  low: number;
  close: number;
  volume: number;
}

export interface Position {
  symbol: string;
  size: number;
  entryPrice: number;
  timestamp: number;
}

export interface Trade {
  symbol: string;
  side: 'buy' | 'sell';
  size: number;
  price: number;
  timestamp: number;
}