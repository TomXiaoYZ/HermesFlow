package com.hermesflow.usermanagement.entity;

import org.junit.jupiter.api.DisplayName;
import org.junit.jupiter.api.Nested;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.params.ParameterizedTest;
import org.junit.jupiter.params.provider.ValueSource;
import org.springframework.security.core.GrantedAuthority;

import java.time.LocalDateTime;
import java.util.Collection;
import java.util.UUID;

import static org.assertj.core.api.Assertions.assertThat;
import static java.util.UUID.randomUUID;

/**
 * User 实体测试类
 */
@DisplayName("User Entity Tests")
class UserTest extends EntityTestBase {

    @Nested
    @DisplayName("Constructor Tests")
    class ConstructorTests {

        @Test
        @DisplayName("Should create user with default constructor")
        void shouldCreateUserWithDefaultConstructor() {
            // When
            User user = new User();
            
            // Then
            assertThat(user.getId()).isNull();
            assertThat(user.getTenant()).isNull();
            assertThat(user.getUsername()).isNull();
            assertThat(user.getEmail()).isNull();
            assertThat(user.getPasswordHash()).isNull();
            assertThat(user.getStatus()).isEqualTo(User.UserStatus.ACTIVE);
            assertThat(user.getEmailVerified()).isFalse();
            assertThat(user.getPhoneVerified()).isFalse();
            assertThat(user.getFailedLoginAttempts()).isEqualTo(0);
        }

        @Test
        @DisplayName("Should create user with parameterized constructor")
        void shouldCreateUserWithParameterizedConstructor() {
            // Given
            Tenant tenant = new Tenant();
            
            // When
            User user = new User(tenant, "testuser", "test@example.com", "hashedpassword");
            
            // Then
            assertThat(user.getTenant()).isEqualTo(tenant);
            assertThat(user.getUsername()).isEqualTo("testuser");
            assertThat(user.getEmail()).isEqualTo("test@example.com");
            assertThat(user.getPasswordHash()).isEqualTo("hashedpassword");
            assertThat(user.getStatus()).isEqualTo(User.UserStatus.ACTIVE);
            assertThat(user.getPasswordChangedAt()).isNotNull();
        }
    }

    @Nested
    @DisplayName("UserDetails Tests")
    class UserDetailsTests {

        private Tenant createActiveTenant() {
            Tenant tenant = new Tenant();
            tenant.setId(randomUUID());
            tenant.setName("Test Tenant");
            tenant.setCode("TEST");
            tenant.setStatus(Tenant.TenantStatus.ACTIVE);
            return tenant;
        }

        private Tenant createSuspendedTenant() {
            Tenant tenant = new Tenant();
            tenant.setId(randomUUID());
            tenant.setName("Test Tenant");
            tenant.setCode("TEST");
            tenant.setStatus(Tenant.TenantStatus.SUSPENDED);
            return tenant;
        }

        @Test
        @DisplayName("Should return authorities")
        void shouldReturnAuthorities() {
            // Given
            User user = new User();
            user.setTenant(createActiveTenant());
            
            // When
            Collection<? extends GrantedAuthority> authorities = user.getAuthorities();
            
            // Then
            assertThat(authorities).isNotNull();
            assertThat(authorities).hasSize(1);
            assertThat(authorities.iterator().next().getAuthority()).isEqualTo("ROLE_USER");
        }

        @Test
        @DisplayName("Should return password hash as password")
        void shouldReturnPasswordHashAsPassword() {
            // Given
            User user = new User();
            user.setPasswordHash("hashedPassword");
            
            // When & Then
            assertThat(user.getPassword()).isEqualTo("hashedPassword");
        }

        @Test
        @DisplayName("Should return username")
        void shouldReturnUsername() {
            // Given
            User user = new User();
            user.setUsername("testuser");
            
            // When & Then
            assertThat(user.getUsername()).isEqualTo("testuser");
        }

        @Test
        @DisplayName("Should return true for account non expired when active")
        void shouldReturnTrueForAccountNonExpiredWhenActive() {
            // Given
            User user = new User();
            user.setStatus(User.UserStatus.ACTIVE);
            
            // When & Then
            assertThat(user.isAccountNonExpired()).isTrue();
        }

        @Test
        @DisplayName("Should return false for account non expired when expired")
        void shouldReturnFalseForAccountNonExpiredWhenExpired() {
            // Given
            User user = new User();
            user.setStatus(User.UserStatus.EXPIRED);
            
            // When & Then
            assertThat(user.isAccountNonExpired()).isFalse();
        }

        @Test
        @DisplayName("Should return true for account non locked when not locked")
        void shouldReturnTrueForAccountNonLockedWhenNotLocked() {
            // Given
            User user = new User();
            user.setStatus(User.UserStatus.ACTIVE);
            
            // When & Then
            assertThat(user.isAccountNonLocked()).isTrue();
        }

        @Test
        @DisplayName("Should return false for account non locked when locked")
        void shouldReturnFalseForAccountNonLockedWhenLocked() {
            // Given
            User user = new User();
            user.setStatus(User.UserStatus.LOCKED);
            
            // When & Then
            assertThat(user.isAccountNonLocked()).isFalse();
        }

        @Test
        @DisplayName("Should return true for credentials non expired when not expired")
        void shouldReturnTrueForCredentialsNonExpiredWhenNotExpired() {
            // Given
            User user = new User();
            user.setPasswordChangedAt(LocalDateTime.now().minusDays(30));
            
            // When & Then
            assertThat(user.isCredentialsNonExpired()).isTrue();
        }

        @Test
        @DisplayName("Should return false for credentials non expired when expired")
        void shouldReturnFalseForCredentialsNonExpiredWhenExpired() {
            // Given
            User user = new User();
            user.setPasswordChangedAt(LocalDateTime.now().minusDays(100));
            
            // When & Then
            assertThat(user.isCredentialsNonExpired()).isFalse();
        }

        @Test
        @DisplayName("Should return true for enabled when active")
        void shouldReturnTrueForEnabledWhenActive() {
            // Given
            User user = new User();
            user.setStatus(User.UserStatus.ACTIVE);
            user.setTenant(createActiveTenant());
            
            // When & Then
            assertThat(user.isEnabled()).isTrue();
        }

        @Test
        @DisplayName("Should return false for enabled when inactive")
        void shouldReturnFalseForEnabledWhenInactive() {
            // Given
            User user = new User();
            user.setStatus(User.UserStatus.INACTIVE);
            user.setTenant(createActiveTenant());
            
            // When & Then
            assertThat(user.isEnabled()).isFalse();
        }

        @Test
        @DisplayName("Should return false for enabled when tenant is suspended")
        void shouldReturnFalseForEnabledWhenTenantIsSuspended() {
            // Given
            User user = new User();
            user.setStatus(User.UserStatus.ACTIVE);
            user.setTenant(createSuspendedTenant());
            
            // When & Then
            assertThat(user.isEnabled()).isFalse();
        }
    }

    @Nested
    @DisplayName("Business Method Tests")
    class BusinessMethodTests {

        @Test
        @DisplayName("Should return true when user is active")
        void shouldReturnTrueWhenUserIsActive() {
            // Given
            User user = new User();
            user.setStatus(User.UserStatus.ACTIVE);
            
            // When & Then
            assertThat(user.isActive()).isTrue();
        }

        @Test
        @DisplayName("Should return false when user is not active")
        void shouldReturnFalseWhenUserIsNotActive() {
            // Given
            User user = new User();
            user.setStatus(User.UserStatus.INACTIVE);
            
            // When & Then
            assertThat(user.isActive()).isFalse();
        }

        @Test
        @DisplayName("Should return true when user is locked")
        void shouldReturnTrueWhenUserIsLocked() {
            // Given
            User user = new User();
            user.setStatus(User.UserStatus.LOCKED);
            
            // When & Then
            assertThat(user.isLocked()).isTrue();
        }

        @Test
        @DisplayName("Should return false when user is not locked")
        void shouldReturnFalseWhenUserIsNotLocked() {
            // Given
            User user = new User();
            user.setStatus(User.UserStatus.ACTIVE);
            
            // When & Then
            assertThat(user.isLocked()).isFalse();
        }

        @Test
        @DisplayName("Should record login success")
        void shouldRecordLoginSuccess() {
            // Given
            User user = new User();
            user.setFailedLoginAttempts(3);
            user.setLockedUntil(LocalDateTime.now().plusMinutes(10));
            
            // When
            user.recordLoginSuccess();
            
            // Then
            assertThat(user.getLastLoginAt()).isNotNull();
            assertThat(user.getFailedLoginAttempts()).isEqualTo(0);
            assertThat(user.getLockedUntil()).isNull();
        }

        @Test
        @DisplayName("Should record login failure")
        void shouldRecordLoginFailure() {
            // Given
            User user = new User();
            user.setFailedLoginAttempts(0);
            
            // When
            user.recordLoginFailure();
            
            // Then
            assertThat(user.getFailedLoginAttempts()).isEqualTo(1);
        }

        @Test
        @DisplayName("Should lock user after 5 failed attempts")
        void shouldLockUserAfter5FailedAttempts() {
            // Given
            User user = new User();
            user.setFailedLoginAttempts(4);
            
            // When
            user.recordLoginFailure();
            
            // Then
            assertThat(user.getStatus()).isEqualTo(User.UserStatus.LOCKED);
            assertThat(user.getLockedUntil()).isNotNull();
        }

        @Test
        @DisplayName("Should unlock user")
        void shouldUnlockUser() {
            // Given
            User user = new User();
            user.setStatus(User.UserStatus.LOCKED);
            user.setFailedLoginAttempts(5);
            user.setLockedUntil(LocalDateTime.now().plusMinutes(10));
            
            // When
            user.unlock();
            
            // Then
            assertThat(user.getStatus()).isEqualTo(User.UserStatus.ACTIVE);
            assertThat(user.getFailedLoginAttempts()).isEqualTo(0);
            assertThat(user.getLockedUntil()).isNull();
        }

        @Test
        @DisplayName("Should return full name when both first and last name exist")
        void shouldReturnFullNameWhenBothFirstAndLastNameExist() {
            // Given
            User user = new User();
            user.setFirstName("John");
            user.setLastName("Doe");
            
            // When & Then
            assertThat(user.getFullName()).isEqualTo("John Doe");
        }

        @Test
        @DisplayName("Should return username when no names exist")
        void shouldReturnUsernameWhenNoNamesExist() {
            // Given
            User user = new User();
            user.setUsername("testuser");
            
            // When & Then
            assertThat(user.getFullName()).isEqualTo("testuser");
        }
    }

    @Nested
    @DisplayName("Validation Tests")
    class ValidationTests {

        @Test
        @DisplayName("Should handle null username")
        void shouldHandleNullUsername() {
            // Given
            User user = new User();
            
            // When
            user.setUsername(null);
            
            // Then
            assertThat(user.getUsername()).isNull();
        }

        @Test
        @DisplayName("Should handle empty username")
        void shouldHandleEmptyUsername() {
            // Given
            User user = new User();
            
            // When
            user.setUsername("");
            
            // Then
            assertThat(user.getUsername()).isEmpty();
        }

        @Test
        @DisplayName("Should handle long username")
        void shouldHandleLongUsername() {
            // Given
            User user = new User();
            String longUsername = "a".repeat(100);
            
            // When
            user.setUsername(longUsername);
            
            // Then
            assertThat(user.getUsername()).isEqualTo(longUsername);
        }

        @Test
        @DisplayName("Should handle null email")
        void shouldHandleNullEmail() {
            // Given
            User user = new User();
            
            // When
            user.setEmail(null);
            
            // Then
            assertThat(user.getEmail()).isNull();
        }

        @Test
        @DisplayName("Should handle empty email")
        void shouldHandleEmptyEmail() {
            // Given
            User user = new User();
            
            // When
            user.setEmail("");
            
            // Then
            assertThat(user.getEmail()).isEmpty();
        }

        @Test
        @DisplayName("Should handle long email")
        void shouldHandleLongEmail() {
            // Given
            User user = new User();
            String longEmail = "a".repeat(200) + "@example.com";
            
            // When
            user.setEmail(longEmail);
            
            // Then
            assertThat(user.getEmail()).isEqualTo(longEmail);
        }

        @Test
        @DisplayName("Should handle null password")
        void shouldHandleNullPassword() {
            // Given
            User user = new User();
            
            // When
            user.setPasswordHash(null);
            
            // Then
            assertThat(user.getPasswordHash()).isNull();
        }

        @Test
        @DisplayName("Should handle empty password")
        void shouldHandleEmptyPassword() {
            // Given
            User user = new User();
            
            // When
            user.setPasswordHash("");
            
            // Then
            assertThat(user.getPasswordHash()).isEmpty();
        }

        @Test
        @DisplayName("Should handle long password")
        void shouldHandleLongPassword() {
            // Given
            User user = new User();
            String longPassword = "a".repeat(500);
            
            // When
            user.setPasswordHash(longPassword);
            
            // Then
            assertThat(user.getPasswordHash()).isEqualTo(longPassword);
        }
    }

    @Nested
    @DisplayName("Equals and HashCode Tests")
    class EqualsAndHashCodeTests {

        @Test
        @DisplayName("Should return true when users have same ID")
        void shouldReturnTrueWhenUsersHaveSameId() {
            // Given
            UUID id = UUID.randomUUID();
            User user1 = new User();
            user1.setId(id);
            User user2 = new User();
            user2.setId(id);
            
            // When & Then
            assertThat(user1).isEqualTo(user2);
        }

        @Test
        @DisplayName("Should return false when users have different IDs")
        void shouldReturnFalseWhenUsersHaveDifferentIds() {
            // Given
            User user1 = new User();
            user1.setId(UUID.randomUUID());
            User user2 = new User();
            user2.setId(UUID.randomUUID());
            
            // When & Then
            assertThat(user1).isNotEqualTo(user2);
        }

        @Test
        @DisplayName("Should return same hash code for same user")
        void shouldReturnSameHashCodeForSameUser() {
            // Given
            User user = new User();
            
            // When & Then
            assertThat(user.hashCode()).isEqualTo(user.hashCode());
        }
    }

    @Nested
    @DisplayName("ToString Tests")
    class ToStringTests {

        @Test
        @DisplayName("Should include key information in toString")
        void shouldIncludeKeyInformationInToString() {
            // Given
            User user = new User();
            user.setUsername("testuser");
            user.setEmail("test@example.com");
            
            // When
            String result = user.toString();
            
            // Then
            assertThat(result).contains("testuser");
            assertThat(result).contains("test@example.com");
        }
    }

    @Nested
    @DisplayName("Enum Tests")
    class EnumTests {

        @Test
        @DisplayName("Should return correct user status values")
        void shouldReturnCorrectUserStatusValues() {
            // When & Then
            assertThat(User.UserStatus.ACTIVE.getDisplayName()).isEqualTo("激活");
            assertThat(User.UserStatus.INACTIVE.getDisplayName()).isEqualTo("未激活");
            assertThat(User.UserStatus.LOCKED.getDisplayName()).isEqualTo("锁定");
            assertThat(User.UserStatus.EXPIRED.getDisplayName()).isEqualTo("过期");
        }
    }
} 