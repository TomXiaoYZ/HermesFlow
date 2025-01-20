import { Suspense } from 'react'
import { Routes, Route } from 'react-router-dom'
import { Spin } from 'antd'
import styled from 'styled-components'

import MainLayout from '@layouts/MainLayout'
import { routes } from '@constants/routes'

const LoadingContainer = styled.div`
  display: flex;
  justify-content: center;
  align-items: center;
  height: 100vh;
`

function App() {
  return (
    <Suspense
      fallback={
        <LoadingContainer>
          <Spin size="large" />
        </LoadingContainer>
      }
    >
      <Routes>
        <Route element={<MainLayout />}>
          {routes.map((route) => (
            <Route key={route.path} path={route.path} element={route.element} />
          ))}
        </Route>
      </Routes>
    </Suspense>
  )
}

export default App 