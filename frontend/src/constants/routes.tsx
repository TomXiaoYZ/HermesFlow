import { lazy } from 'react'
import { DashboardOutlined, LineChartOutlined, SettingOutlined } from '@ant-design/icons'

const Dashboard = lazy(() => import('@pages/Dashboard'))
const Market = lazy(() => import('@pages/Market'))
const Settings = lazy(() => import('@pages/Settings'))

export const routes = [
  {
    path: '/',
    element: <Dashboard />,
    name: '仪表盘',
    icon: <DashboardOutlined />,
    showInMenu: true
  },
  {
    path: '/market',
    element: <Market />,
    name: '市场',
    icon: <LineChartOutlined />,
    showInMenu: true
  },
  {
    path: '/settings',
    element: <Settings />,
    name: '设置',
    icon: <SettingOutlined />,
    showInMenu: true
  }
] 