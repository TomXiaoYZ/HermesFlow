package com.hermesflow.usermanagement.security;

import com.hermesflow.usermanagement.entity.Tenant;
import com.hermesflow.usermanagement.entity.User;
import com.hermesflow.usermanagement.repository.TenantRepository;
import com.hermesflow.usermanagement.repository.UserRepository;
import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.Test;
import org.springframework.beans.factory.annotation.Autowired;
import org.springframework.boot.test.context.SpringBootTest;
import org.springframework.security.authentication.UsernamePasswordAuthenticationToken;
import org.springframework.security.core.Authentication;
import org.springframework.security.crypto.password.PasswordEncoder;
import org.springframework.test.context.ActiveProfiles;
import org.springframework.transaction.annotation.Transactional;

import static org.junit.jupiter.api.Assertions.*;

@SpringBootTest
@ActiveProfiles("test")
@Transactional
public class JwtAuthenticationTest {

    @Autowired
    private JwtUtils jwtUtils;

    @Autowired
    private UserDetailsServiceImpl userDetailsService;

    @Autowired
    private UserRepository userRepository;

    @Autowired
    private TenantRepository tenantRepository;

    @Autowired
    private PasswordEncoder passwordEncoder;

    private User testUser;
    private Tenant testTenant;

    @BeforeEach
    void setUp() {
        // 创建测试租户
        testTenant = new Tenant("Test Tenant", "test-tenant", Tenant.PlanType.BASIC);
        testTenant = tenantRepository.save(testTenant);

        // 创建测试用户
        testUser = new User(testTenant, "testuser", "test@example.com", 
                           passwordEncoder.encode("password123"));
        testUser = userRepository.save(testUser);
    }

    @Test
    void testJwtTokenGeneration() {
        // 创建认证对象
        UserPrincipal userPrincipal = UserPrincipal.create(testUser);
        Authentication authentication = new UsernamePasswordAuthenticationToken(
                userPrincipal, null, userPrincipal.getAuthorities());

        // 生成JWT令牌
        String token = jwtUtils.generateJwtToken(authentication);

        assertNotNull(token);
        assertTrue(token.length() > 0);
    }

    @Test
    void testJwtTokenValidation() {
        // 创建认证对象
        UserPrincipal userPrincipal = UserPrincipal.create(testUser);
        Authentication authentication = new UsernamePasswordAuthenticationToken(
                userPrincipal, null, userPrincipal.getAuthorities());

        // 生成JWT令牌
        String token = jwtUtils.generateJwtToken(authentication);

        // 验证令牌
        assertTrue(jwtUtils.validateJwtToken(token));

        // 从令牌中提取用户名
        String username = jwtUtils.getUserNameFromJwtToken(token);
        assertEquals("testuser", username);
    }

    @Test
    void testInvalidJwtToken() {
        String invalidToken = "invalid.jwt.token";
        assertFalse(jwtUtils.validateJwtToken(invalidToken));
    }

    @Test
    void testUserDetailsService() {
        // 测试用户详情服务
        UserPrincipal userDetails = (UserPrincipal) userDetailsService.loadUserByUsername("testuser");

        assertNotNull(userDetails);
        assertEquals("testuser", userDetails.getUsername());
        assertEquals("test@example.com", userDetails.getEmail());
        assertEquals(testTenant.getId().toString(), userDetails.getTenantId());
        assertTrue(userDetails.isEnabled());
        assertTrue(userDetails.isAccountNonExpired());
        assertTrue(userDetails.isAccountNonLocked());
        assertTrue(userDetails.isCredentialsNonExpired());
    }

    @Test
    void testGenerateTokenFromUsername() {
        String token = jwtUtils.generateTokenFromUsername("testuser");
        
        assertNotNull(token);
        assertTrue(jwtUtils.validateJwtToken(token));
        assertEquals("testuser", jwtUtils.getUserNameFromJwtToken(token));
    }
} 