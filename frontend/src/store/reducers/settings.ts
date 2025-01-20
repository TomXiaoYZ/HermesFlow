import { createSlice, PayloadAction } from '@reduxjs/toolkit'

interface SettingsState {
  theme: 'light' | 'dark'
  language: 'zh-CN' | 'en-US'
  notifications: {
    enabled: boolean
    types: {
      priceAlert: boolean
      orderUpdate: boolean
      systemMessage: boolean
    }
  }
  trading: {
    defaultLeverage: number
    confirmBeforeOrder: boolean
    showOrderPreview: boolean
    autoClosePosition: boolean
  }
  display: {
    orderBookDepth: number
    klineInterval: string
    tradesLimit: number
    decimalPlaces: {
      price: number
      amount: number
    }
  }
  api: {
    exchanges: {
      [key: string]: {
        enabled: boolean
        apiKey: string
        secretKey: string
      }
    }
  }
}

const initialState: SettingsState = {
  theme: 'light',
  language: 'zh-CN',
  notifications: {
    enabled: true,
    types: {
      priceAlert: true,
      orderUpdate: true,
      systemMessage: true
    }
  },
  trading: {
    defaultLeverage: 1,
    confirmBeforeOrder: true,
    showOrderPreview: true,
    autoClosePosition: false
  },
  display: {
    orderBookDepth: 20,
    klineInterval: '1m',
    tradesLimit: 50,
    decimalPlaces: {
      price: 2,
      amount: 4
    }
  },
  api: {
    exchanges: {}
  }
}

const settingsSlice = createSlice({
  name: 'settings',
  initialState,
  reducers: {
    setTheme: (state, action: PayloadAction<'light' | 'dark'>) => {
      state.theme = action.payload
    },
    setLanguage: (state, action: PayloadAction<'zh-CN' | 'en-US'>) => {
      state.language = action.payload
    },
    setNotifications: (state, action: PayloadAction<typeof state.notifications>) => {
      state.notifications = action.payload
    },
    setTrading: (state, action: PayloadAction<typeof state.trading>) => {
      state.trading = action.payload
    },
    setDisplay: (state, action: PayloadAction<typeof state.display>) => {
      state.display = action.payload
    },
    setExchangeApi: (
      state,
      action: PayloadAction<{
        exchange: string
        config: {
          enabled: boolean
          apiKey: string
          secretKey: string
        }
      }>
    ) => {
      const { exchange, config } = action.payload
      state.api.exchanges[exchange] = config
    },
    removeExchangeApi: (state, action: PayloadAction<string>) => {
      delete state.api.exchanges[action.payload]
    }
  }
})

export const {
  setTheme,
  setLanguage,
  setNotifications,
  setTrading,
  setDisplay,
  setExchangeApi,
  removeExchangeApi
} = settingsSlice.actions

export default settingsSlice.reducer 