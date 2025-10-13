# Java 开发者完整指南

> **HermesFlow 交易/用户/风控服务 - Java 开发指南** | **适用于**: 交易引擎、用户管理、风控引擎

---

## 🎯 本指南目标

帮助 Java 开发者：
1. ✅ 快速上手 HermesFlow Java 服务开发
2. ✅ 掌握 Spring Boot 3.x + JDK 21 最佳实践
3. ✅ 理解多租户架构和安全设计
4. ✅ 高效调试和优化代码

---

## 📚 必读文档

- 📋 [执行模块 PRD](../prd/modules/03-execution-module.md) - 交易引擎需求
- 📋 [账户模块 PRD](../prd/modules/05-account-module.md) - 用户管理需求
- 📋 [风控模块 PRD](../prd/modules/04-risk-module.md) - 风控引擎需求
- 🏗️ [系统架构 - Java 服务层](../architecture/system-architecture.md#44-java-服务层)
- 📝 [编码规范 - Java 部分](../development/coding-standards.md#java-规范)

---

## 🚀 快速开始

### 环境搭建（20分钟）

#### 1. 安装 JDK 21

```bash
# macOS
brew install openjdk@21
echo 'export PATH="/opt/homebrew/opt/openjdk@21/bin:$PATH"' >> ~/.zshrc
source ~/.zshrc

# Linux (Ubuntu/Debian)
sudo apt update
sudo apt install openjdk-21-jdk

# 验证
java --version  # 应为 21+
javac --version
```

#### 2. 安装 Maven

```bash
# macOS
brew install maven

# Linux
sudo apt install maven

# 验证
mvn --version
```

#### 3. IDE 配置

**IntelliJ IDEA（强烈推荐）**:

1. 安装插件:
   - **Lombok** (必装)
   - **Spring Boot Assistant**
   - **SonarLint** (代码质量)
   - **JPA Buddy** (JPA 开发)

2. 配置 Annotation Processing:
   ```
   Settings → Build → Compiler → Annotation Processors
   勾选 "Enable annotation processing"
   ```

3. 配置 Code Style:
   ```
   Settings → Editor → Code Style → Java
   导入 Google Java Style Guide
   ```

#### 4. 克隆和构建

```bash
# 克隆代码
git clone <your-repo-url>/HermesFlow.git
cd HermesFlow/modules/trading-engine

# 构建
./mvnw clean install

# 运行测试
./mvnw test

# 启动服务
./mvnw spring-boot:run
```

---

## 📁 项目结构

```
modules/trading-engine/
├── pom.xml                  # Maven 配置
├── src/
│   ├── main/
│   │   ├── java/
│   │   │   └── com/hermesflow/trading/
│   │   │       ├── TradingApplication.java  # Spring Boot 入口
│   │   │       │
│   │   │       ├── controller/     # REST Controllers
│   │   │       │   ├── OrderController.java
│   │   │       │   └── PositionController.java
│   │   │       │
│   │   │       ├── service/        # 业务逻辑
│   │   │       │   ├── OrderService.java
│   │   │       │   └── PositionService.java
│   │   │       │
│   │   │       ├── repository/     # 数据访问
│   │   │       │   ├── OrderRepository.java
│   │   │       │   └── PositionRepository.java
│   │   │       │
│   │   │       ├── entity/         # JPA Entities
│   │   │       │   ├── Order.java
│   │   │       │   └── Position.java
│   │   │       │
│   │   │       ├── dto/            # 数据传输对象
│   │   │       │   ├── OrderRequest.java
│   │   │       │   └── OrderResponse.java
│   │   │       │
│   │   │       ├── config/         # 配置类
│   │   │       │   ├── SecurityConfig.java
│   │   │       │   ├── JpaConfig.java
│   │   │       │   └── KafkaConfig.java
│   │   │       │
│   │   │       ├── security/       # 安全相关
│   │   │       │   ├── JwtTokenProvider.java
│   │   │       │   └── TenantContext.java
│   │   │       │
│   │   │       └── exception/      # 异常处理
│   │   │           ├── GlobalExceptionHandler.java
│   │   │           └── OrderNotFoundException.java
│   │   │
│   │   └── resources/
│   │       ├── application.yml     # 主配置
│   │       ├── application-dev.yml # 开发环境
│   │       └── application-prod.yml # 生产环境
│   │
│   └── test/
│       ├── java/
│       │   └── com/hermesflow/trading/
│       │       ├── controller/     # Controller 测试
│       │       ├── service/        # Service 测试
│       │       └── repository/     # Repository 测试
│       └── resources/
│           └── application-test.yml
```

---

## 🔧 核心技术栈

### Spring Boot 3.2

#### 基本应用结构

```java
// TradingApplication.java
package com.hermesflow.trading;

import org.springframework.boot.SpringApplication;
import org.springframework.boot.autoconfigure.SpringBootApplication;

@SpringBootApplication
public class TradingApplication {
    public static void main(String[] args) {
        SpringApplication.run(TradingApplication.class, args);
    }
}
```

#### 配置文件

```yaml
# application.yml
spring:
  application:
    name: trading-engine
  
  # 虚拟线程（JDK 21）
  threads:
    virtual:
      enabled: true
  
  # 数据库
  datasource:
    url: jdbc:postgresql://localhost:5432/hermesflow
    username: ${DB_USERNAME}
    password: ${DB_PASSWORD}
  
  jpa:
    hibernate:
      ddl-auto: validate
    properties:
      hibernate:
        dialect: org.hibernate.dialect.PostgreSQLDialect
  
  # Redis
  redis:
    host: localhost
    port: 6379
  
  # Kafka
  kafka:
    bootstrap-servers: localhost:9092
    consumer:
      group-id: trading-engine
```

---

### JDK 21 Virtual Threads

HermesFlow 使用 Virtual Threads 提升并发性能。

#### 基本使用

```java
// 自动启用 Virtual Threads
@Service
public class OrderService {
    
    // Spring Boot 3.2+ 自动使用 Virtual Threads
    public CompletableFuture<Order> placeOrderAsync(OrderRequest request) {
        return CompletableFuture.supplyAsync(() -> {
            // 业务逻辑
            return placeOrder(request);
        });
    }
}
```

#### 显式使用

```java
import java.util.concurrent.Executors;

public class VirtualThreadExample {
    
    public void processOrders(List<Order> orders) {
        try (var executor = Executors.newVirtualThreadPerTaskExecutor()) {
            orders.forEach(order -> 
                executor.submit(() -> processOrder(order))
            );
        }
    }
}
```

**注意事项**:
- ❌ 避免在 `synchronized` 块中长时间阻塞（会 pin 线程）
- ✅ 使用 `ReentrantLock` 代替 `synchronized`
- ✅ 避免频繁的线程池创建

---

### Spring Data JPA

#### Entity 定义

```java
@Entity
@Table(name = "orders")
@EntityListeners(AuditingEntityListener.class)
public class Order {
    
    @Id
    @GeneratedValue(strategy = GenerationType.IDENTITY)
    private Long id;
    
    @Column(name = "tenant_id", nullable = false)
    private String tenantId;
    
    @Column(name = "user_id", nullable = false)
    private Long userId;
    
    @Column(nullable = false)
    private String symbol;
    
    @Enumerated(EnumType.STRING)
    private OrderType type;  // MARKET, LIMIT
    
    @Enumerated(EnumType.STRING)
    private OrderSide side;  // BUY, SELL
    
    @Column(nullable = false)
    private BigDecimal quantity;
    
    private BigDecimal price;
    
    @Enumerated(EnumType.STRING)
    private OrderStatus status;  // NEW, FILLED, CANCELLED
    
    @CreatedDate
    private LocalDateTime createdAt;
    
    @LastModifiedDate
    private LocalDateTime updatedAt;
    
    // 自动设置 tenant_id
    @PrePersist
    public void setTenantId() {
        this.tenantId = TenantContext.getCurrentTenantId();
    }
    
    // Getters, Setters, equals(), hashCode()
}
```

#### Repository

```java
@Repository
public interface OrderRepository extends JpaRepository<Order, Long> {
    
    // 自动包含 tenant_id 过滤（通过 RLS）
    List<Order> findByUserId(Long userId);
    
    // 自定义查询
    @Query("SELECT o FROM Order o WHERE o.tenantId = :tenantId AND o.status = :status")
    List<Order> findByTenantIdAndStatus(
        @Param("tenantId") String tenantId,
        @Param("status") OrderStatus status
    );
    
    // JOIN FETCH 避免 N+1 问题
    @Query("SELECT o FROM Order o JOIN FETCH o.user WHERE o.id = :id")
    Optional<Order> findByIdWithUser(@Param("id") Long id);
}
```

---

### Spring Security 6

#### 安全配置

```java
@Configuration
@EnableWebSecurity
@EnableMethodSecurity
public class SecurityConfig {
    
    @Autowired
    private JwtTokenProvider jwtTokenProvider;
    
    @Bean
    public SecurityFilterChain filterChain(HttpSecurity http) throws Exception {
        http
            .csrf(csrf -> csrf.disable())
            .cors(cors -> cors.configurationSource(corsConfigurationSource()))
            .sessionManagement(session -> 
                session.sessionCreationPolicy(SessionCreationPolicy.STATELESS)
            )
            .authorizeHttpRequests(auth -> auth
                .requestMatchers("/api/v1/auth/**", "/actuator/health").permitAll()
                .requestMatchers("/api/v1/admin/**").hasRole("ADMIN")
                .anyRequest().authenticated()
            )
            .addFilterBefore(
                new JwtAuthenticationFilter(jwtTokenProvider),
                UsernamePasswordAuthenticationFilter.class
            );
        
        return http.build();
    }
    
    @Bean
    public CorsConfigurationSource corsConfigurationSource() {
        CorsConfiguration config = new CorsConfiguration();
        config.setAllowedOrigins(Arrays.asList("http://localhost:3000"));
        config.setAllowedMethods(Arrays.asList("GET", "POST", "PUT", "DELETE"));
        config.setAllowedHeaders(Arrays.asList("*"));
        config.setAllowCredentials(true);
        
        UrlBasedCorsConfigurationSource source = new UrlBasedCorsConfigurationSource();
        source.registerCorsConfiguration("/**", config);
        return source;
    }
}
```

#### JWT Token Provider

```java
@Component
public class JwtTokenProvider {
    
    @Value("${jwt.secret}")
    private String jwtSecret;
    
    @Value("${jwt.expiration:86400000}") // 24 hours
    private long jwtExpiration;
    
    public String generateToken(Authentication authentication) {
        UserDetails userDetails = (UserDetails) authentication.getPrincipal();
        Date now = new Date();
        Date expiryDate = new Date(now.getTime() + jwtExpiration);
        
        return Jwts.builder()
            .setSubject(userDetails.getUsername())
            .claim("tenant_id", TenantContext.getCurrentTenantId())
            .setIssuedAt(now)
            .setExpiration(expiryDate)
            .signWith(SignatureAlgorithm.HS512, jwtSecret)
            .compact();
    }
    
    public boolean validateToken(String token) {
        try {
            Jwts.parser().setSigningKey(jwtSecret).parseClaimsJws(token);
            return true;
        } catch (JwtException | IllegalArgumentException e) {
            return false;
        }
    }
    
    public String getUsernameFromToken(String token) {
        Claims claims = Jwts.parser()
            .setSigningKey(jwtSecret)
            .parseClaimsJws(token)
            .getBody();
        return claims.getSubject();
    }
}
```

---

## 🎨 常用设计模式

### 1. Controller - Service - Repository 分层

```java
// Controller 层
@RestController
@RequestMapping("/api/v1/orders")
@RequiredArgsConstructor
public class OrderController {
    
    private final OrderService orderService;
    
    @PostMapping
    public ResponseEntity<OrderResponse> createOrder(@Valid @RequestBody OrderRequest request) {
        Order order = orderService.createOrder(request);
        return ResponseEntity.status(HttpStatus.CREATED)
            .body(OrderResponse.from(order));
    }
    
    @GetMapping("/{id}")
    public ResponseEntity<OrderResponse> getOrder(@PathVariable Long id) {
        return orderService.findById(id)
            .map(OrderResponse::from)
            .map(ResponseEntity::ok)
            .orElse(ResponseEntity.notFound().build());
    }
}

// Service 层
@Service
@Transactional
@RequiredArgsConstructor
public class OrderService {
    
    private final OrderRepository orderRepository;
    private final PositionService positionService;
    private final KafkaTemplate<String, OrderEvent> kafkaTemplate;
    
    public Order createOrder(OrderRequest request) {
        // 1. 验证
        validateOrder(request);
        
        // 2. 创建订单
        Order order = Order.builder()
            .symbol(request.getSymbol())
            .type(request.getType())
            .side(request.getSide())
            .quantity(request.getQuantity())
            .price(request.getPrice())
            .status(OrderStatus.NEW)
            .build();
        
        // 3. 保存
        order = orderRepository.save(order);
        
        // 4. 发送 Kafka 事件
        kafkaTemplate.send("orders", OrderEvent.from(order));
        
        return order;
    }
}

// Repository 层
@Repository
public interface OrderRepository extends JpaRepository<Order, Long> {
    // Spring Data JPA 自动实现
}
```

---

### 2. DTO Pattern

```java
// Request DTO
@Data
@Builder
public class OrderRequest {
    
    @NotBlank(message = "Symbol is required")
    private String symbol;
    
    @NotNull(message = "Type is required")
    private OrderType type;
    
    @NotNull(message = "Side is required")
    private OrderSide side;
    
    @NotNull(message = "Quantity is required")
    @DecimalMin(value = "0.0", inclusive = false)
    private BigDecimal quantity;
    
    @DecimalMin(value = "0.0", inclusive = false)
    private BigDecimal price;
}

// Response DTO
@Data
@Builder
public class OrderResponse {
    private Long id;
    private String symbol;
    private OrderType type;
    private OrderSide side;
    private BigDecimal quantity;
    private BigDecimal price;
    private OrderStatus status;
    private LocalDateTime createdAt;
    
    public static OrderResponse from(Order order) {
        return OrderResponse.builder()
            .id(order.getId())
            .symbol(order.getSymbol())
            .type(order.getType())
            .side(order.getSide())
            .quantity(order.getQuantity())
            .price(order.getPrice())
            .status(order.getStatus())
            .createdAt(order.getCreatedAt())
            .build();
    }
}
```

---

### 3. 全局异常处理

```java
@RestControllerAdvice
@Slf4j
public class GlobalExceptionHandler {
    
    @ExceptionHandler(OrderNotFoundException.class)
    public ResponseEntity<ErrorResponse> handleOrderNotFound(OrderNotFoundException ex) {
        log.error("Order not found: {}", ex.getMessage());
        return ResponseEntity.status(HttpStatus.NOT_FOUND)
            .body(ErrorResponse.builder()
                .error("ORDER_NOT_FOUND")
                .message(ex.getMessage())
                .timestamp(LocalDateTime.now())
                .build());
    }
    
    @ExceptionHandler(MethodArgumentNotValidException.class)
    public ResponseEntity<ErrorResponse> handleValidationError(MethodArgumentNotValidException ex) {
        Map<String, String> errors = ex.getBindingResult().getFieldErrors().stream()
            .collect(Collectors.toMap(
                FieldError::getField,
                error -> error.getDefaultMessage() != null ? error.getDefaultMessage() : ""
            ));
        
        return ResponseEntity.status(HttpStatus.BAD_REQUEST)
            .body(ErrorResponse.builder()
                .error("VALIDATION_ERROR")
                .message("Validation failed")
                .details(errors)
                .timestamp(LocalDateTime.now())
                .build());
    }
    
    @ExceptionHandler(Exception.class)
    public ResponseEntity<ErrorResponse> handleGenericError(Exception ex) {
        log.error("Unexpected error", ex);
        return ResponseEntity.status(HttpStatus.INTERNAL_SERVER_ERROR)
            .body(ErrorResponse.builder()
                .error("INTERNAL_ERROR")
                .message("An unexpected error occurred")
                .timestamp(LocalDateTime.now())
                .build());
    }
}
```

---

## 🔐 多租户实现

### Tenant Context

```java
public class TenantContext {
    
    private static final ThreadLocal<String> CURRENT_TENANT = new ThreadLocal<>();
    
    public static void setCurrentTenantId(String tenantId) {
        CURRENT_TENANT.set(tenantId);
    }
    
    public static String getCurrentTenantId() {
        return CURRENT_TENANT.get();
    }
    
    public static void clear() {
        CURRENT_TENANT.remove();
    }
}
```

### JWT Filter

```java
@Component
@RequiredArgsConstructor
public class JwtAuthenticationFilter extends OncePerRequestFilter {
    
    private final JwtTokenProvider jwtTokenProvider;
    
    @Override
    protected void doFilterInternal(HttpServletRequest request,
                                     HttpServletResponse response,
                                     FilterChain filterChain) 
            throws ServletException, IOException {
        
        String token = getJwtFromRequest(request);
        
        if (StringUtils.hasText(token) && jwtTokenProvider.validateToken(token)) {
            // 设置租户上下文
            String tenantId = jwtTokenProvider.getTenantIdFromToken(token);
            TenantContext.setCurrentTenantId(tenantId);
            
            // 设置认证信息
            Authentication authentication = jwtTokenProvider.getAuthentication(token);
            SecurityContextHolder.getContext().setAuthentication(authentication);
        }
        
        try {
            filterChain.doFilter(request, response);
        } finally {
            TenantContext.clear();
        }
    }
    
    private String getJwtFromRequest(HttpServletRequest request) {
        String bearerToken = request.getHeader("Authorization");
        if (StringUtils.hasText(bearerToken) && bearerToken.startsWith("Bearer ")) {
            return bearerToken.substring(7);
        }
        return null;
    }
}
```

---

## 🧪 测试

### 单元测试

```java
@ExtendWith(MockitoExtension.class)
class OrderServiceTest {
    
    @Mock
    private OrderRepository orderRepository;
    
    @Mock
    private KafkaTemplate<String, OrderEvent> kafkaTemplate;
    
    @InjectMocks
    private OrderService orderService;
    
    @Test
    void createOrder_Success() {
        // Given
        OrderRequest request = OrderRequest.builder()
            .symbol("BTC/USDT")
            .type(OrderType.LIMIT)
            .side(OrderSide.BUY)
            .quantity(new BigDecimal("1.0"))
            .price(new BigDecimal("50000.0"))
            .build();
        
        Order savedOrder = Order.builder()
            .id(1L)
            .symbol("BTC/USDT")
            .status(OrderStatus.NEW)
            .build();
        
        when(orderRepository.save(any(Order.class))).thenReturn(savedOrder);
        
        // When
        Order result = orderService.createOrder(request);
        
        // Then
        assertNotNull(result);
        assertEquals(1L, result.getId());
        assertEquals(OrderStatus.NEW, result.getStatus());
        verify(orderRepository).save(any(Order.class));
        verify(kafkaTemplate).send(eq("orders"), any(OrderEvent.class));
    }
}
```

### Controller 测试

```java
@WebMvcTest(OrderController.class)
@Import(SecurityConfig.class)
class OrderControllerTest {
    
    @Autowired
    private MockMvc mockMvc;
    
    @MockBean
    private OrderService orderService;
    
    @MockBean
    private JwtTokenProvider jwtTokenProvider;
    
    @Test
    void createOrder_Success() throws Exception {
        // Given
        OrderRequest request = OrderRequest.builder()
            .symbol("BTC/USDT")
            .type(OrderType.LIMIT)
            .side(OrderSide.BUY)
            .quantity(new BigDecimal("1.0"))
            .price(new BigDecimal("50000.0"))
            .build();
        
        Order order = Order.builder()
            .id(1L)
            .symbol("BTC/USDT")
            .status(OrderStatus.NEW)
            .build();
        
        when(orderService.createOrder(any(OrderRequest.class))).thenReturn(order);
        
        // When & Then
        mockMvc.perform(post("/api/v1/orders")
                .contentType(MediaType.APPLICATION_JSON)
                .content(new ObjectMapper().writeValueAsString(request))
                .header("Authorization", "Bearer valid-token"))
            .andExpect(status().isCreated())
            .andExpect(jsonPath("$.id").value(1))
            .andExpect(jsonPath("$.symbol").value("BTC/USDT"))
            .andExpect(jsonPath("$.status").value("NEW"));
    }
}
```

### 集成测试

```java
@SpringBootTest(webEnvironment = SpringBootTest.WebEnvironment.RANDOM_PORT)
@Testcontainers
class OrderIntegrationTest {
    
    @Container
    static PostgreSQLContainer<?> postgres = new PostgreSQLContainer<>("postgres:15")
        .withDatabaseName("hermesflow_test")
        .withUsername("test")
        .withPassword("test");
    
    @Autowired
    private TestRestTemplate restTemplate;
    
    @Test
    void createAndGetOrder_Success() {
        // Create order
        OrderRequest request = new OrderRequest(/* ... */);
        ResponseEntity<OrderResponse> createResponse = restTemplate.postForEntity(
            "/api/v1/orders",
            request,
            OrderResponse.class
        );
        
        assertEquals(HttpStatus.CREATED, createResponse.getStatusCode());
        Long orderId = createResponse.getBody().getId();
        
        // Get order
        ResponseEntity<OrderResponse> getResponse = restTemplate.getForEntity(
            "/api/v1/orders/" + orderId,
            OrderResponse.class
        );
        
        assertEquals(HttpStatus.OK, getResponse.getStatusCode());
        assertEquals(orderId, getResponse.getBody().getId());
    }
}
```

**覆盖率目标**: ≥ 80%

---

## 📚 推荐资源

- [Spring Boot 文档](https://docs.spring.io/spring-boot/docs/current/reference/html/)
- [Spring Data JPA 文档](https://docs.spring.io/spring-data/jpa/docs/current/reference/html/)
- [JDK 21 Virtual Threads](https://openjdk.org/jeps/444)

---

## 📞 获取帮助

- **Java Team**: Slack `#java-dev`
- **技术问题**: [FAQ](../FAQ.md)

---

**最后更新**: 2025-01-13  
**维护者**: @architect.mdc  
**版本**: v1.0

