import axios from 'axios'
import { ApiResponse } from '@types/api'

const api = axios.create({
  baseURL: '/api'
})

// 验证API密钥
export const validateApiKeys = async (
  exchange: string,
  apiKey: string,
  secretKey: string
): Promise<boolean> => {
  try {
    const response = await api.post<ApiResponse<boolean>>('/exchange/validate-api', {
      exchange,
      apiKey,
      secretKey
    })
    return response.data.data
  } catch (error) {
    return false
  }
}

// 获取交易所支持的交易对
export const getExchangeSymbols = async (exchange: string): Promise<string[]> => {
  const response = await api.get<ApiResponse<string[]>>(`/exchange/${exchange}/symbols`)
  return response.data.data
}

// 获取交易所支持的时间周期
export const getExchangeIntervals = async (exchange: string): Promise<string[]> => {
  const response = await api.get<ApiResponse<string[]>>(`/exchange/${exchange}/intervals`)
  return response.data.data
}

// 获取交易所API限制
export const getExchangeLimits = async (exchange: string): Promise<{
  requests: { [key: string]: number }
  orders: { [key: string]: number }
}> => {
  const response = await api.get<
    ApiResponse<{
      requests: { [key: string]: number }
      orders: { [key: string]: number }
    }>
  >(`/exchange/${exchange}/limits`)
  return response.data.data
} 