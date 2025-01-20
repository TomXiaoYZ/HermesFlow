import { useCallback } from 'react'
import { useDispatch, useSelector } from 'react-redux'

import { RootState } from '@store/index'
import {
  setTheme,
  setLanguage,
  setNotifications,
  setTrading,
  setDisplay,
  setExchangeApi,
  removeExchangeApi
} from '@store/reducers/settings'
import { validateExchangeApiRequest } from '@store/sagas/settings'

export const useSettings = () => {
  const dispatch = useDispatch()
  const settings = useSelector((state: RootState) => state.settings)

  // 更新主题
  const updateTheme = useCallback(
    (theme: 'light' | 'dark') => {
      dispatch(setTheme(theme))
    },
    [dispatch]
  )

  // 更新语言
  const updateLanguage = useCallback(
    (language: 'zh-CN' | 'en-US') => {
      dispatch(setLanguage(language))
    },
    [dispatch]
  )

  // 更新通知设置
  const updateNotifications = useCallback(
    (notifications: typeof settings.notifications) => {
      dispatch(setNotifications(notifications))
    },
    [dispatch]
  )

  // 更新交易设置
  const updateTrading = useCallback(
    (trading: typeof settings.trading) => {
      dispatch(setTrading(trading))
    },
    [dispatch]
  )

  // 更新显示设置
  const updateDisplay = useCallback(
    (display: typeof settings.display) => {
      dispatch(setDisplay(display))
    },
    [dispatch]
  )

  // 添加交易所API
  const addExchangeApi = useCallback(
    async (exchange: string, apiKey: string, secretKey: string) => {
      dispatch(
        validateExchangeApiRequest({
          exchange,
          apiKey,
          secretKey
        })
      )
    },
    [dispatch]
  )

  // 删除交易所API
  const deleteExchangeApi = useCallback(
    (exchange: string) => {
      dispatch(removeExchangeApi(exchange))
    },
    [dispatch]
  )

  return {
    settings,
    updateTheme,
    updateLanguage,
    updateNotifications,
    updateTrading,
    updateDisplay,
    addExchangeApi,
    deleteExchangeApi
  }
} 