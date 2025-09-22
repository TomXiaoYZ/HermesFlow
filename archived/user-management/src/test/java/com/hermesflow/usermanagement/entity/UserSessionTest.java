package com.hermesflow.usermanagement.entity;

import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.DisplayName;
import org.junit.jupiter.api.Nested;
import org.junit.jupiter.api.Test;

import jakarta.validation.ConstraintViolation;
import jakarta.validation.Validation;
import jakarta.validation.Validator;
import jakarta.validation.ValidatorFactory;

import java.net.InetAddress;
import java.net.UnknownHostException;
import java.time.LocalDateTime;
import java.util.Set;
import java.util.UUID;

import static org.junit.jupiter.api.Assertions.*;

@DisplayName("UserSession Entity Tests")
class UserSessionTest {

    private Validator validator;
    private User testUser;

    @BeforeEach
    void setUp() {
        ValidatorFactory factory = Validation.buildDefaultValidatorFactory();
        validator = factory.getValidator();
        
        // 创建测试用户
        testUser = new User();
        testUser.setId(UUID.randomUUID());
        testUser.setUsername("testuser");
        testUser.setEmail("test@example.com");
    }

    @Nested
    @DisplayName("Constructor Tests")
    class ConstructorTests {

        @Test
        @DisplayName("Default constructor should create empty session")
        void testDefaultConstructor() {
            UserSession session = new UserSession();
            
            assertNull(session.getId());
            assertNull(session.getUser());
            assertNull(session.getSessionToken());
            assertNull(session.getRefreshToken());
            assertNull(session.getIpAddress());
            assertNull(session.getUserAgent());
            assertNull(session.getExpiresAt());
            assertTrue(session.getIsActive());
            assertNull(session.getLastActivityAt());
        }

        @Test
        @DisplayName("Four-parameter constructor should set properties correctly")
        void testFourParameterConstructor() {
            LocalDateTime expiresAt = LocalDateTime.now().plusHours(1);
            UserSession session = new UserSession(testUser, "session-token", "refresh-token", expiresAt);
            
            assertEquals(testUser, session.getUser());
            assertEquals("session-token", session.getSessionToken());
            assertEquals("refresh-token", session.getRefreshToken());
            assertEquals(expiresAt, session.getExpiresAt());
            assertTrue(session.getIsActive());
            assertNotNull(session.getLastActivityAt());
        }
    }

    @Nested
    @DisplayName("Session State Management")
    class SessionStateTests {

        @Test
        @DisplayName("isExpired should return true for expired session")
        void testIsExpiredTrue() {
            LocalDateTime pastTime = LocalDateTime.now().minusHours(1);
            UserSession session = new UserSession(testUser, "token", "refresh", pastTime);
            
            assertTrue(session.isExpired());
        }

        @Test
        @DisplayName("isExpired should return false for valid session")
        void testIsExpiredFalse() {
            LocalDateTime futureTime = LocalDateTime.now().plusHours(1);
            UserSession session = new UserSession(testUser, "token", "refresh", futureTime);
            
            assertFalse(session.isExpired());
        }

        @Test
        @DisplayName("isValid should return true for active and non-expired session")
        void testIsValidTrue() {
            LocalDateTime futureTime = LocalDateTime.now().plusHours(1);
            UserSession session = new UserSession(testUser, "token", "refresh", futureTime);
            
            assertTrue(session.isValid());
        }

        @Test
        @DisplayName("isValid should return false for inactive session")
        void testIsValidFalseInactive() {
            LocalDateTime futureTime = LocalDateTime.now().plusHours(1);
            UserSession session = new UserSession(testUser, "token", "refresh", futureTime);
            session.setIsActive(false);
            
            assertFalse(session.isValid());
        }

        @Test
        @DisplayName("isValid should return false for expired session")
        void testIsValidFalseExpired() {
            LocalDateTime pastTime = LocalDateTime.now().minusHours(1);
            UserSession session = new UserSession(testUser, "token", "refresh", pastTime);
            
            assertFalse(session.isValid());
        }

        @Test
        @DisplayName("updateActivity should update last activity time")
        void testUpdateActivity() {
            UserSession session = new UserSession(testUser, "token", "refresh", LocalDateTime.now().plusHours(1));
            LocalDateTime beforeUpdate = session.getLastActivityAt();
            
            // 等待一小段时间确保时间戳不同
            try {
                Thread.sleep(10);
            } catch (InterruptedException e) {
                Thread.currentThread().interrupt();
            }
            
            session.updateActivity();
            
            assertTrue(session.getLastActivityAt().isAfter(beforeUpdate));
        }

        @Test
        @DisplayName("invalidate should set session as inactive")
        void testInvalidate() {
            UserSession session = new UserSession(testUser, "token", "refresh", LocalDateTime.now().plusHours(1));
            assertTrue(session.getIsActive());
            
            session.invalidate();
            
            assertFalse(session.getIsActive());
        }

        @Test
        @DisplayName("extend should update expiration time and activity")
        void testExtend() {
            LocalDateTime originalExpiry = LocalDateTime.now().plusHours(1);
            UserSession session = new UserSession(testUser, "token", "refresh", originalExpiry);
            LocalDateTime originalActivity = session.getLastActivityAt();
            
            LocalDateTime newExpiry = LocalDateTime.now().plusHours(2);
            
            // 等待一小段时间确保时间戳不同
            try {
                Thread.sleep(10);
            } catch (InterruptedException e) {
                Thread.currentThread().interrupt();
            }
            
            session.extend(newExpiry);
            
            assertEquals(newExpiry, session.getExpiresAt());
            assertTrue(session.getLastActivityAt().isAfter(originalActivity));
        }
    }

    @Nested
    @DisplayName("Validation Tests")
    class ValidationTests {

        @Test
        @DisplayName("Valid session should pass validation")
        void testValidSession() {
            UserSession session = new UserSession(testUser, "session-token", "refresh-token", LocalDateTime.now().plusHours(1));
            
            Set<ConstraintViolation<UserSession>> violations = validator.validate(session);
            assertTrue(violations.isEmpty());
        }

        @Test
        @DisplayName("Session with null user should fail validation")
        void testNullUser() {
            UserSession session = new UserSession(null, "session-token", "refresh-token", LocalDateTime.now().plusHours(1));
            
            Set<ConstraintViolation<UserSession>> violations = validator.validate(session);
            assertFalse(violations.isEmpty());
            assertTrue(violations.stream().anyMatch(v -> v.getMessage().contains("用户ID不能为空")));
        }

        @Test
        @DisplayName("Session with blank session token should fail validation")
        void testBlankSessionToken() {
            UserSession session = new UserSession(testUser, "", "refresh-token", LocalDateTime.now().plusHours(1));
            
            Set<ConstraintViolation<UserSession>> violations = validator.validate(session);
            assertFalse(violations.isEmpty());
            assertTrue(violations.stream().anyMatch(v -> v.getMessage().contains("会话令牌不能为空")));
        }

        @Test
        @DisplayName("Session with blank refresh token should fail validation")
        void testBlankRefreshToken() {
            UserSession session = new UserSession(testUser, "session-token", "", LocalDateTime.now().plusHours(1));
            
            Set<ConstraintViolation<UserSession>> violations = validator.validate(session);
            assertFalse(violations.isEmpty());
            assertTrue(violations.stream().anyMatch(v -> v.getMessage().contains("刷新令牌不能为空")));
        }

        @Test
        @DisplayName("Session with null expires at should fail validation")
        void testNullExpiresAt() {
            UserSession session = new UserSession(testUser, "session-token", "refresh-token", null);
            
            Set<ConstraintViolation<UserSession>> violations = validator.validate(session);
            assertFalse(violations.isEmpty());
            assertTrue(violations.stream().anyMatch(v -> v.getMessage().contains("过期时间不能为空")));
        }
    }

    @Nested
    @DisplayName("Network Information Tests")
    class NetworkInfoTests {

        @Test
        @DisplayName("IP address should be settable and retrievable")
        void testIpAddress() throws UnknownHostException {
            UserSession session = new UserSession(testUser, "token", "refresh", LocalDateTime.now().plusHours(1));
            InetAddress ipAddress = InetAddress.getByName("192.168.1.1");
            
            session.setIpAddress(ipAddress);
            
            assertEquals(ipAddress, session.getIpAddress());
        }

        @Test
        @DisplayName("User agent should be settable and retrievable")
        void testUserAgent() {
            UserSession session = new UserSession(testUser, "token", "refresh", LocalDateTime.now().plusHours(1));
            String userAgent = "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36";
            
            session.setUserAgent(userAgent);
            
            assertEquals(userAgent, session.getUserAgent());
        }
    }

    @Nested
    @DisplayName("Equality and Hash Code Tests")
    class EqualityTests {

        @Test
        @DisplayName("Sessions with same ID should be equal")
        void testEqualityWithSameId() {
            UUID id = UUID.randomUUID();
            UserSession session1 = new UserSession(testUser, "token1", "refresh1", LocalDateTime.now().plusHours(1));
            UserSession session2 = new UserSession(testUser, "token2", "refresh2", LocalDateTime.now().plusHours(2));
            session1.setId(id);
            session2.setId(id);
            
            assertEquals(session1, session2);
            assertEquals(session1.hashCode(), session2.hashCode());
        }

        @Test
        @DisplayName("Sessions with different IDs should not be equal")
        void testEqualityWithDifferentIds() {
            UserSession session1 = new UserSession(testUser, "token", "refresh", LocalDateTime.now().plusHours(1));
            UserSession session2 = new UserSession(testUser, "token", "refresh", LocalDateTime.now().plusHours(1));
            session1.setId(UUID.randomUUID());
            session2.setId(UUID.randomUUID());
            
            assertNotEquals(session1, session2);
        }

        @Test
        @DisplayName("Session with null ID should not be equal to session with ID")
        void testEqualityWithNullId() {
            UserSession session1 = new UserSession(testUser, "token", "refresh", LocalDateTime.now().plusHours(1));
            UserSession session2 = new UserSession(testUser, "token", "refresh", LocalDateTime.now().plusHours(1));
            session2.setId(UUID.randomUUID());
            
            assertNotEquals(session1, session2);
        }

        @Test
        @DisplayName("Session should be equal to itself")
        void testEqualityWithSelf() {
            UserSession session = new UserSession(testUser, "token", "refresh", LocalDateTime.now().plusHours(1));
            assertEquals(session, session);
        }

        @Test
        @DisplayName("Session should not be equal to null or different type")
        void testEqualityWithNullAndDifferentType() {
            UserSession session = new UserSession(testUser, "token", "refresh", LocalDateTime.now().plusHours(1));
            
            assertNotEquals(session, null);
            assertNotEquals(session, "string");
        }
    }

    @Nested
    @DisplayName("ToString Tests")
    class ToStringTests {

        @Test
        @DisplayName("toString should include key information without exposing full token")
        void testToString() {
            UserSession session = new UserSession(testUser, "very-long-session-token", "refresh", LocalDateTime.now().plusHours(1));
            session.setId(UUID.randomUUID());
            session.setIsActive(true);
            
            String toString = session.toString();
            
            assertAll(
                () -> assertTrue(toString.contains("UserSession")),
                () -> assertTrue(toString.contains("id=")),
                () -> assertTrue(toString.contains("sessionToken=")),
                () -> assertTrue(toString.contains("isActive=true")),
                () -> assertFalse(toString.contains("very-long-session-token")), // 完整token不应该暴露
                () -> assertTrue(toString.contains("very-long-...")) // 应该只显示前10个字符
            );
        }
    }

    @Nested
    @DisplayName("Audit Fields Tests")
    class AuditFieldsTests {

        @Test
        @DisplayName("Created at should be settable")
        void testCreatedAt() {
            UserSession session = new UserSession(testUser, "token", "refresh", LocalDateTime.now().plusHours(1));
            LocalDateTime now = LocalDateTime.now();
            
            session.setCreatedAt(now);
            
            assertEquals(now, session.getCreatedAt());
        }
    }

    @Nested
    @DisplayName("Edge Cases Tests")
    class EdgeCasesTests {

        @Test
        @DisplayName("Session should handle null active flag")
        void testNullActiveFlag() {
            UserSession session = new UserSession(testUser, "token", "refresh", LocalDateTime.now().plusHours(1));
            session.setIsActive(null);
            
            assertNull(session.getIsActive());
            assertThrows(NullPointerException.class, session::isValid); // null会抛出NullPointerException
        }

        @Test
        @DisplayName("Session should handle null last activity")
        void testNullLastActivity() {
            UserSession session = new UserSession(testUser, "token", "refresh", LocalDateTime.now().plusHours(1));
            session.setLastActivityAt(null);
            
            assertNull(session.getLastActivityAt());
        }

        @Test
        @DisplayName("updateActivity should work even with null last activity")
        void testUpdateActivityWithNullLastActivity() {
            UserSession session = new UserSession(testUser, "token", "refresh", LocalDateTime.now().plusHours(1));
            session.setLastActivityAt(null);
            
            session.updateActivity();
            
            assertNotNull(session.getLastActivityAt());
        }
    }
} 