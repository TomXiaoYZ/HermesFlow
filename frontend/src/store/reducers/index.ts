import { combineReducers } from '@reduxjs/toolkit'

import marketReducer from './market'
import settingsReducer from './settings'

const rootReducer = combineReducers({
  market: marketReducer,
  settings: settingsReducer
})

export default rootReducer 