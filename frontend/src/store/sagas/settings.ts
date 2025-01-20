import { call, put, takeLatest } from 'redux-saga/effects'
import { createAction, PayloadAction } from '@reduxjs/toolkit'

import { setError } from '@store/reducers/market'
import { setExchangeApi } from '@store/reducers/settings'
import { validateApiKeys } from '@api/exchange'

// Action Types
export const validateExchangeApiRequest = createAction<{
  exchange: string
  apiKey: string
  secretKey: string
}>('settings/validateExchangeApiRequest')

// Sagas
function* validateExchangeApi(
  action: PayloadAction<{
    exchange: string
    apiKey: string
    secretKey: string
  }>
) {
  try {
    const { exchange, apiKey, secretKey } = action.payload

    // 验证API密钥
    const isValid: boolean = yield call(validateApiKeys, exchange, apiKey, secretKey)

    if (isValid) {
      // 如果验证成功，保存API配置
      yield put(
        setExchangeApi({
          exchange,
          config: {
            enabled: true,
            apiKey,
            secretKey
          }
        })
      )
      yield put(setError(null))
    } else {
      yield put(setError('API密钥验证失败'))
    }
  } catch (error) {
    yield put(setError(error instanceof Error ? error.message : 'API密钥验证失败'))
  }
}

export default function* settingsSaga() {
  yield takeLatest(validateExchangeApiRequest.type, validateExchangeApi)
} 