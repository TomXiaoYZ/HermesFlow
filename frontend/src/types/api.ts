// 市场数据类型
export interface KlineData {
  timestamp: number
  open: string
  high: string
  low: string
  close: string
  volume: string
}

export interface OrderbookData {
  bids: [string, string][] // [价格, 数量]
  asks: [string, string][] // [价格, 数量]
  timestamp: number
}

export interface TradeData {
  id: string
  price: string
  quantity: string
  timestamp: number
  isBuyerMaker: boolean
}

// WebSocket订阅回调
export interface WebSocketCallbacks {
  onKline: (data: KlineData) => void
  onOrderbook: (data: OrderbookData) => void
  onTrade: (data: TradeData) => void
}

// API响应类型
export interface ApiResponse<T> {
  code: number
  message: string
  data: T
}

// 错误响应
export interface ApiError {
  code: number
  message: string
} 