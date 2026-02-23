# user-management - FOLDER_INDEX

> Java Spring Boot 3.2 service for user authentication, RBAC, and multi-tenancy. Issues JWT tokens consumed by the gateway.

## Module Map

```
src/main/java/io/hermesflow/usermanagement/
  UserManagementApplication.java       # Spring Boot entry point

  config/
    SecurityConfig.java                # Spring Security config (CORS, CSRF, auth rules)

  controller/
    AuthController.java                # POST /api/auth/login, token refresh
    HealthController.java              # GET /actuator/health

  model/
    User.java                          # User entity (JPA)
    Role.java                          # Role entity (RBAC)
    Permission.java                    # Permission entity
    LoginRequest.java                  # Login DTO
    AuthResponse.java                  # JWT token response DTO

  repository/
    UserRepository.java                # JPA: User CRUD
    RoleRepository.java                # JPA: Role CRUD
    PermissionRepository.java          # JPA: Permission CRUD

  service/
    JwtService.java                    # JWT generation/validation (HS256, Base64 secret)
    DataInitializer.java               # Seeds initial admin user on startup

src/test/java/io/hermesflow/usermanagement/
  controller/
    HealthControllerTest.java          # Health endpoint test
```

## Auth Flow
1. Client → `POST /api/auth/login` with `{username, password}`
2. `AuthController` validates credentials via `UserRepository`
3. `JwtService` generates JWT (HS256, `JWT_SECRET` env var)
4. Client receives `{token, refreshToken, expiresIn}`
5. Gateway validates JWT on protected routes via `jwt_auth.rs`

## Dependencies
- Spring Boot 3.2, Spring Security, Spring Data JPA
- PostgreSQL (TimescaleDB)
- `io.jsonwebtoken:jjwt` (JWT)
