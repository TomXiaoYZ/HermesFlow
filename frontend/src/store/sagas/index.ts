import { all } from 'redux-saga/effects'

import marketSaga from './market'
import settingsSaga from './settings'

export default function* rootSaga() {
  yield all([
    marketSaga(),
    settingsSaga()
  ])
} 