import { call, put, takeLatest, select } from 'redux-saga/effects'
import { createAction } from '@reduxjs/toolkit'

import { RootState } from '@store/index'
import { setLoading, setError, setSymbols, updateKlines, updateOrderbook, updateTrades } from '@store/reducers/market'
import { fetchSymbols, fetchKlines, fetchOrderbook, fetchTrades, subscribeToMarketData, unsubscribeFromMarketData } from '@api/market'

// Action Types
export const fetchMarketDataRequest = createAction('market/fetchMarketDataRequest')
export const subscribeMarketDataRequest = createAction<string>('market/subscribeMarketDataRequest')
export const unsubscribeMarketDataRequest = createAction<string>('market/unsubscribeMarketDataRequest')

// Sagas
function* fetchMarketData() {
  try {
    yield put(setLoading(true))
    
    // 获取交易对列表
    const symbols: string[] = yield call(fetchSymbols)
    yield put(setSymbols(symbols))

    // 获取第一个交易对的数据
    if (symbols.length > 0) {
      const symbol = symbols[0]
      
      // 获取K线数据
      const klines = yield call(fetchKlines, symbol)
      yield put(updateKlines({ symbol, data: klines }))

      // 获取订单簿数据
      const orderbook = yield call(fetchOrderbook, symbol)
      yield put(updateOrderbook({ symbol, data: orderbook }))

      // 获取最新成交数据
      const trades = yield call(fetchTrades, symbol)
      yield put(updateTrades({ symbol, data: trades }))
    }

    yield put(setError(null))
  } catch (error) {
    yield put(setError(error instanceof Error ? error.message : '获取市场数据失败'))
  } finally {
    yield put(setLoading(false))
  }
}

function* subscribeMarketData(action: ReturnType<typeof subscribeMarketDataRequest>) {
  try {
    const symbol = action.payload
    const currentSymbol: string | null = yield select((state: RootState) => state.market.selectedSymbol)

    // 如果已经订阅了其他交易对，先取消订阅
    if (currentSymbol && currentSymbol !== symbol) {
      yield call(unsubscribeFromMarketData, currentSymbol)
    }

    // 订阅新的交易对
    yield call(subscribeToMarketData, symbol, {
      onKline: (data) => {
        put(updateKlines({ symbol, data: [data] }))
      },
      onOrderbook: (data) => {
        put(updateOrderbook({ symbol, data }))
      },
      onTrade: (data) => {
        put(updateTrades({ symbol, data: [data] }))
      }
    })

  } catch (error) {
    yield put(setError(error instanceof Error ? error.message : '订阅市场数据失败'))
  }
}

function* unsubscribeMarketData(action: ReturnType<typeof unsubscribeMarketDataRequest>) {
  try {
    yield call(unsubscribeFromMarketData, action.payload)
  } catch (error) {
    yield put(setError(error instanceof Error ? error.message : '取消订阅市场数据失败'))
  }
}

export default function* marketSaga() {
  yield takeLatest(fetchMarketDataRequest.type, fetchMarketData)
  yield takeLatest(subscribeMarketDataRequest.type, subscribeMarketData)
  yield takeLatest(unsubscribeMarketDataRequest.type, unsubscribeMarketData)
} 