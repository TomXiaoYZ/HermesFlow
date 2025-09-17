import { MarketData } from '../types';
import { SMA } from 'technicalindicators';

export class MovingAverageCrossover {
  private shortPeriod: number;
  private longPeriod: number;
  private shortMA: number[];
  private longMA: number[];

  constructor(shortPeriod: number = 10, longPeriod: number = 20) {
    this.shortPeriod = shortPeriod;
    this.longPeriod = longPeriod;
    this.shortMA = [];
    this.longMA = [];
  }

  analyze(data: MarketData[]): { signal: 'buy' | 'sell' | 'hold' } {
    const prices = data.map(d => d.close);
    
    this.shortMA = SMA.calculate({
      period: this.shortPeriod,
      values: prices
    });

    this.longMA = SMA.calculate({
      period: this.longPeriod,
      values: prices
    });

    const lastShortMA = this.shortMA[this.shortMA.length - 1];
    const prevShortMA = this.shortMA[this.shortMA.length - 2];
    const lastLongMA = this.longMA[this.longMA.length - 1];
    const prevLongMA = this.longMA[this.longMA.length - 2];

    if (prevShortMA <= prevLongMA && lastShortMA > lastLongMA) {
      return { signal: 'buy' };
    } else if (prevShortMA >= prevLongMA && lastShortMA < lastLongMA) {
      return { signal: 'sell' };
    }

    return { signal: 'hold' };
  }
}