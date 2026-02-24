// Mock data for charts and tables in Chinese

export const EQUITY_DATA = [
  { time: '09:30', value: 1000000, benchmark: 1000000 },
  { time: '10:30', value: 1002300, benchmark: 1000500 },
  { time: '11:30', value: 1005600, benchmark: 1001200 },
  { time: '13:00', value: 1004800, benchmark: 1000800 },
  { time: '14:00', value: 1008900, benchmark: 1001500 },
  { time: '15:00', value: 1012400, benchmark: 1002100 },
];

export const ORDER_BOOK_ASKS = [
  { price: 64240.50, size: 0.045, total: 4.2 },
  { price: 64239.00, size: 1.200, total: 4.1 },
  { price: 64238.50, size: 0.500, total: 2.9 },
  { price: 64236.00, size: 0.100, total: 2.4 },
].reverse();

export const ORDER_BOOK_BIDS = [
  { price: 64234.00, size: 2.500, total: 2.5 },
  { price: 64233.50, size: 0.300, total: 2.8 },
  { price: 64231.00, size: 0.800, total: 3.6 },
  { price: 64230.00, size: 5.000, total: 8.6 },
];

export const ACTIVE_STRATEGIES = [
  { id: 1, name: 'Alpha-Trend-v1 (趋势追踪)', asset: 'BTC/USDT', status: '运行中', dailyPnL: 5230.50, totalPnL: 45200.00, return: 15.4, sharpe: 2.45, leverage: '2.5x' },
  { id: 2, name: 'Mean-Reversion (均值回归)', asset: 'ETH/USDT', status: '运行中', dailyPnL: -1200.00, totalPnL: 8400.00, return: 3.2, sharpe: 1.12, leverage: '1.0x' },
  { id: 3, name: 'Option-Iron-Condor (铁鹰)', asset: 'SPX Options', status: '已停止', dailyPnL: 0.00, totalPnL: 12150.00, return: 8.5, sharpe: 3.05, leverage: '5.0x' },
  { id: 4, name: 'Stat-Arb-Pairs (统计套利)', asset: 'GOOG/MSFT', status: '运行中', dailyPnL: 450.20, totalPnL: 3200.00, return: 5.1, sharpe: 1.88, leverage: '4.0x' },
];

export const SENTIMENT_FEED = [
  { id: 1, source: 'Bloomberg', type: 'News', time: '14:32', content: 'SEC 内部人士暗示比特币现货 ETF 审批进入最终阶段，预计下周一公布结果。', sentiment: '极度看涨', score: 92, entity: 'BTC' },
  { id: 2, source: 'Twitter (X)', type: 'Social', time: '14:30', content: '巨鲸警报: 10,000 ETH 从 Binance 转移至未知冷钱包。', sentiment: '看涨', score: 65, entity: 'ETH' },
  { id: 3, source: 'Fed Watch', type: 'Macro', time: '14:15', content: '美联储会议纪要显示，多数官员支持在年底前维持高利率，降息预期降温。', sentiment: '看跌', score: -78, entity: 'Macro' },
  { id: 4, source: 'Reuters', type: 'News', time: '14:05', content: '特斯拉上海超级工厂出货量同比增长 15%，超出分析师预期。', sentiment: '看涨', score: 55, entity: 'TSLA' },
  { id: 5, source: 'SeekingAlpha', type: 'Report', time: '13:58', content: '分析师下调苹果 Q4 营收预期，理由是中国市场需求疲软。', sentiment: '看跌', score: -45, entity: 'AAPL' },
  { id: 6, source: 'Reddit /r/wsb', type: 'Social', time: '13:45', content: 'GME 散户讨论热度飙升 300%，空头挤压风险增加。', sentiment: '高波', score: 30, entity: 'GME' },
];

export const SENTIMENT_MOVERS = [
  { rank: 1, symbol: 'SOL', name: 'Solana', score: 88, change: '+12%', vol: 'High', signal: 'Buy' },
  { rank: 2, symbol: 'NVDA', name: 'Nvidia', score: 76, change: '+5%', vol: 'Med', signal: 'Hold' },
  { rank: 3, symbol: 'TSLA', name: 'Tesla', score: -42, change: '-8%', vol: 'High', signal: 'Sell' },
  { rank: 4, symbol: 'AMD', name: 'AMD', score: 65, change: '+3%', vol: 'Med', signal: 'Buy' },
  { rank: 5, symbol: 'ETH', name: 'Ethereum', score: 15, change: '-1%', vol: 'Low', signal: 'Neutral' },
];

export const HOT_TOPICS = [
  { text: 'ETF批准', weight: 95, sentiment: 'up' },
  { text: '降息预期', weight: 80, sentiment: 'down' },
  { text: 'AI监管', weight: 65, sentiment: 'neutral' },
  { text: '非农数据', weight: 60, sentiment: 'warn' },
  { text: '减半周期', weight: 55, sentiment: 'up' },
  { text: '地缘政治', weight: 40, sentiment: 'down' },
  { text: 'Q3财报', weight: 35, sentiment: 'up' },
];

export const FACTOR_DATA = [
  { rank: 1, ticker: 'NVDA', name: '英伟达 (NVIDIA)', factorVal: 88.42, zScore: 2.84, sector: '科技' },
  { rank: 2, ticker: 'AMD', name: '超威半导体 (AMD)', factorVal: 82.15, zScore: 2.15, sector: '科技' },
  { rank: 3, ticker: 'TSLA', name: '特斯拉 (Tesla)', factorVal: 79.02, zScore: 1.92, sector: '消费' },
  { rank: 4, ticker: 'META', name: 'Meta Platforms', factorVal: 76.33, zScore: 1.75, sector: '通信' },
  { rank: 5, ticker: 'AVGO', name: '博通 (Broadcom)', factorVal: 71.10, zScore: 1.42, sector: '科技' },
];

export const LIVE_EXECUTIONS = [
  { id: 101, time: '14:32:05', ticker: 'BTCUSDT', side: '买入', price: 64235.5, size: 0.050, type: '限价', venue: 'Binance', strategy: 'Alpha-Trend' },
  { id: 102, time: '14:31:58', ticker: 'NVDA', side: '卖出', price: 895.20, size: 10, type: '市价', venue: 'IBKR', strategy: 'Mean-Rev' },
  { id: 103, time: '14:31:42', ticker: 'ETHUSDT', side: '买入', price: 3450.1, size: 1.200, type: '限价', venue: 'OKX', strategy: 'Alpha-Trend' },
  { id: 104, time: '14:30:15', ticker: 'AAPL', side: '买入', price: 175.40, size: 50, type: 'TWAP', venue: 'IBKR', strategy: 'L/S Equity' },
  { id: 105, time: '14:29:55', ticker: 'SOLUSDT', side: '卖出', price: 145.20, size: 25.000, type: '限价', venue: 'Bybit', strategy: 'HFT-Mix' },
  { id: 106, time: '14:28:10', ticker: 'BTCUSDT', side: '买入', price: 64210.0, size: 0.100, type: 'ICE', venue: 'Binance', strategy: 'Alpha-Trend' },
  { id: 107, time: '14:27:45', ticker: 'MSFT', side: '卖出', price: 412.50, size: 15, type: '市价', venue: 'IBKR', strategy: 'L/S Equity' },
  { id: 108, time: '14:26:30', ticker: 'ETHUSDT', side: '卖出', price: 3455.0, size: 0.500, type: '限价', venue: 'OKX', strategy: 'Mean-Rev' },
];

export const FACTOR_LIBRARY = {
  technical: [
    { name: 'RSI', desc: '相对强弱指标' },
    { name: 'MACD', desc: '指数平滑异同' },
    { name: 'KDJ', desc: '随机指标' },
    { name: 'Bollinger', desc: '布林带' },
    { name: 'ATR', desc: '平均真实波幅' }
  ],
  fundamental: [
    { name: 'P/E Ratio', desc: '市盈率' },
    { name: 'P/B Ratio', desc: '市净率' },
    { name: 'EPS Growth', desc: '每股收益增长' },
    { name: 'ROE', desc: '净资产收益率' }
  ],
  sentiment: [
    { name: 'AI Sentiment', desc: 'AI 舆情得分' },
    { name: 'Social Vol', desc: '社交声量' },
    { name: 'News Impact', desc: '新闻冲击指数' },
    { name: 'Fear & Greed', desc: '贪婪恐慌指数' }
  ],
  logic: [
    { name: 'AND', desc: '与逻辑' },
    { name: 'OR', desc: '或逻辑' },
    { name: 'IF/ELSE', desc: '条件判断' },
    { name: 'Compare', desc: '比较运算' }
  ]
};
