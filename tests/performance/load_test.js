import http from 'k6/http';
import { check, sleep } from 'k6';
import { Rate } from 'k6/metrics';

// 自定义指标
const errorRate = new Rate('errors');

export let options = {
  stages: [
    { duration: '2m', target: 100 },  // 2分钟内逐步增加到100并发
    { duration: '5m', target: 100 },  // 保持100并发5分钟
    { duration: '2m', target: 200 },  // 增加到200并发
    { duration: '5m', target: 200 },  // 保持200并发5分钟
    { duration: '2m', target: 0 },    // 逐步降到0
  ],
  thresholds: {
    'http_req_duration': ['p(95)<500'],  // 95%请求<500ms
    'http_req_failed': ['rate<0.01'],    // 错误率<1%
    'errors': ['rate<0.01'],             // 业务错误率<1%
  },
};

const BASE_URL = __ENV.BASE_URL || 'http://api.hermesflow.com';
const TOKEN = __ENV.API_TOKEN;

export default function () {
  // TC-PERF-001: 获取策略列表
  let response = http.get(`${BASE_URL}/api/v1/strategies`, {
    headers: { 'Authorization': `Bearer ${TOKEN}` },
  });
  
  check(response, {
    'status is 200': (r) => r.status === 200,
    'response time < 500ms': (r) => r.timings.duration < 500,
  }) || errorRate.add(1);
  
  sleep(1);
  
  // TC-PERF-002: 获取市场数据
  response = http.get(`${BASE_URL}/api/v1/market-data/BTCUSDT`, {
    headers: { 'Authorization': `Bearer ${TOKEN}` },
  });
  
  check(response, {
    'status is 200': (r) => r.status === 200,
    'response time < 200ms': (r) => r.timings.duration < 200,
    'has price data': (r) => JSON.parse(r.body).price > 0,
  }) || errorRate.add(1);
  
  sleep(1);
  
  // TC-PERF-003: 创建策略（写操作）
  if (__ITER % 10 == 0) {  // 每10次迭代创建一次
    response = http.post(`${BASE_URL}/api/v1/strategies`,
      JSON.stringify({
        name: `Load Test Strategy ${__ITER}`,
        code: 'def run(): pass'
      }),
      {
        headers: {
          'Authorization': `Bearer ${TOKEN}`,
          'Content-Type': 'application/json',
        },
      }
    );
    
    check(response, {
      'create status is 201': (r) => r.status === 201,
      'create response time < 1000ms': (r) => r.timings.duration < 1000,
    }) || errorRate.add(1);
  }
  
  sleep(1);
}

