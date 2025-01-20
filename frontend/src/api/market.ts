import axios from 'axios'
import { KlineData, OrderbookData, TradeData, WebSocketCallbacks, ApiResponse } from '@types/api'

const api = axios.create({
  baseURL: '/api'
})

// 获取交易对列表
export const fetchSymbols = async (): Promise<string[]> => {
  const response = await api.get<ApiResponse<string[]>>('/market/symbols')
  return response.data.data
}

// 获取K线数据
export const fetchKlines = async (symbol: string): Promise<KlineData[]> => {
  const response = await api.get<ApiResponse<KlineData[]>>(`/market/klines/${symbol}`)
  return response.data.data
}

// 获取订单簿数据
export const fetchOrderbook = async (symbol: string): Promise<OrderbookData> => {
  const response = await api.get<ApiResponse<OrderbookData>>(`/market/orderbook/${symbol}`)
  return response.data.data
}

// 获取最新成交数据
export const fetchTrades = async (symbol: string): Promise<TradeData[]> => {
  const response = await api.get<ApiResponse<TradeData[]>>(`/market/trades/${symbol}`)
  return response.data.data
}

// WebSocket连接管理
const wsConnections: { [key: string]: WebSocket } = {}

// 订阅市场数据
export const subscribeToMarketData = (symbol: string, callbacks: WebSocketCallbacks): void => {
  const ws = new WebSocket(`ws://${window.location.host}/api/ws/market/${symbol}`)

  ws.onmessage = (event) => {
    const data = JSON.parse(event.data)
    
    switch (data.type) {
      case 'kline':
        callbacks.onKline(data.data)
        break
      case 'orderbook':
        callbacks.onOrderbook(data.data)
        break
      case 'trade':
        callbacks.onTrade(data.data)
        break
    }
  }

  wsConnections[symbol] = ws
}

// 取消订阅市场数据
export const unsubscribeFromMarketData = (symbol: string): void => {
  const ws = wsConnections[symbol]
  if (ws) {
    ws.close()
    delete wsConnections[symbol]
  }
} 