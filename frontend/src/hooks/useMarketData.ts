import { useEffect } from 'react'
import { useDispatch, useSelector } from 'react-redux'

import { RootState } from '@store/index'
import {
  fetchMarketDataRequest,
  subscribeMarketDataRequest,
  unsubscribeMarketDataRequest
} from '@store/sagas/market'
import { setSelectedSymbol } from '@store/reducers/market'

export const useMarketData = (symbol?: string) => {
  const dispatch = useDispatch()
  const {
    loading,
    error,
    symbols,
    selectedSymbol,
    klines,
    orderbook,
    trades
  } = useSelector((state: RootState) => state.market)

  // 初始化市场数据
  useEffect(() => {
    dispatch(fetchMarketDataRequest())
  }, [dispatch])

  // 订阅指定交易对的数据
  useEffect(() => {
    if (symbol) {
      dispatch(setSelectedSymbol(symbol))
      dispatch(subscribeMarketDataRequest(symbol))

      return () => {
        dispatch(unsubscribeMarketDataRequest(symbol))
      }
    }
  }, [dispatch, symbol])

  return {
    loading,
    error,
    symbols,
    selectedSymbol,
    klines: symbol ? klines[symbol] : [],
    orderbook: symbol ? orderbook[symbol] : { bids: [], asks: [], timestamp: 0 },
    trades: symbol ? trades[symbol] : []
  }
} 