import { createSlice, PayloadAction } from '@reduxjs/toolkit'

interface MarketState {
  loading: boolean
  error: string | null
  symbols: string[]
  selectedSymbol: string | null
  klines: {
    [key: string]: {
      timestamp: number
      open: string
      high: string
      low: string
      close: string
      volume: string
    }[]
  }
  orderbook: {
    [key: string]: {
      bids: [string, string][]
      asks: [string, string][]
      timestamp: number
    }
  }
  trades: {
    [key: string]: {
      id: string
      price: string
      quantity: string
      timestamp: number
      isBuyerMaker: boolean
    }[]
  }
}

const initialState: MarketState = {
  loading: false,
  error: null,
  symbols: [],
  selectedSymbol: null,
  klines: {},
  orderbook: {},
  trades: {}
}

const marketSlice = createSlice({
  name: 'market',
  initialState,
  reducers: {
    setLoading: (state, action: PayloadAction<boolean>) => {
      state.loading = action.payload
    },
    setError: (state, action: PayloadAction<string | null>) => {
      state.error = action.payload
    },
    setSymbols: (state, action: PayloadAction<string[]>) => {
      state.symbols = action.payload
    },
    setSelectedSymbol: (state, action: PayloadAction<string | null>) => {
      state.selectedSymbol = action.payload
    },
    updateKlines: (
      state,
      action: PayloadAction<{
        symbol: string
        data: {
          timestamp: number
          open: string
          high: string
          low: string
          close: string
          volume: string
        }[]
      }>
    ) => {
      const { symbol, data } = action.payload
      state.klines[symbol] = data
    },
    updateOrderbook: (
      state,
      action: PayloadAction<{
        symbol: string
        data: {
          bids: [string, string][]
          asks: [string, string][]
          timestamp: number
        }
      }>
    ) => {
      const { symbol, data } = action.payload
      state.orderbook[symbol] = data
    },
    updateTrades: (
      state,
      action: PayloadAction<{
        symbol: string
        data: {
          id: string
          price: string
          quantity: string
          timestamp: number
          isBuyerMaker: boolean
        }[]
      }>
    ) => {
      const { symbol, data } = action.payload
      state.trades[symbol] = data
    }
  }
})

export const {
  setLoading,
  setError,
  setSymbols,
  setSelectedSymbol,
  updateKlines,
  updateOrderbook,
  updateTrades
} = marketSlice.actions

export default marketSlice.reducer 