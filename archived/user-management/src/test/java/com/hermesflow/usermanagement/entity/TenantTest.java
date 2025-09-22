package com.hermesflow.usermanagement.entity;

import org.junit.jupiter.api.DisplayName;
import org.junit.jupiter.api.Nested;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.params.ParameterizedTest;
import org.junit.jupiter.params.provider.ValueSource;

import java.util.UUID;

import static org.assertj.core.api.Assertions.assertThat;

/**
 * Tenant实体单元测试
 * 
 * 测试租户实体的所有功能，包括：
 * - 构造函数和基本属性
 * - 业务方法逻辑
 * - 枚举类型
 * - 验证注解
 * - 关联关系
 */
@DisplayName("Tenant Entity Tests")
class TenantTest extends EntityTestBase {

    @Nested
    @DisplayName("Constructor Tests")
    class ConstructorTests {

        @Test
        @DisplayName("Should create tenant with default constructor")
        void shouldCreateTenantWithDefaultConstructor() {
            // When
            Tenant tenant = new Tenant();
            
            // Then
            assertThat(tenant.getId()).isNull();
            assertThat(tenant.getName()).isNull();
            assertThat(tenant.getCode()).isNull();
            assertThat(tenant.getPlanType()).isEqualTo(Tenant.PlanType.BASIC);
            assertThat(tenant.getStatus()).isEqualTo(Tenant.TenantStatus.ACTIVE);
            assertThat(tenant.getMaxUsers()).isEqualTo(10);
            assertThat(tenant.getMaxStrategies()).isEqualTo(5);
            assertThat(tenant.getMaxAssetSubscriptions()).isEqualTo(50);
        }

        @Test
        @DisplayName("Should create tenant with parameterized constructor")
        void shouldCreateTenantWithParameterizedConstructor() {
            // When
            Tenant tenant = new Tenant("Test Tenant", "TEST", Tenant.PlanType.PRO);
            
            // Then
            assertThat(tenant.getName()).isEqualTo("Test Tenant");
            assertThat(tenant.getCode()).isEqualTo("TEST");
            assertThat(tenant.getPlanType()).isEqualTo(Tenant.PlanType.PRO);
            assertThat(tenant.getStatus()).isEqualTo(Tenant.TenantStatus.ACTIVE);
        }

        @Test
        @DisplayName("Should create tenant with enterprise plan")
        void shouldCreateTenantWithEnterprisePlan() {
            // Given
            Tenant tenant = new Tenant("Test Tenant", "TEST", Tenant.PlanType.ENTERPRISE);
            
            // When & Then
            assertThat(tenant.getPlanType()).isEqualTo(Tenant.PlanType.ENTERPRISE);
        }
    }

    @Nested
    @DisplayName("Business Method Tests")
    class BusinessMethodTests {

        @Test
        @DisplayName("Should return true when tenant is active")
        void shouldReturnTrueWhenTenantIsActive() {
            // Given
            Tenant tenant = new Tenant();
            tenant.setStatus(Tenant.TenantStatus.ACTIVE);
            
            // When & Then
            assertThat(tenant.isActive()).isTrue();
        }

        @Test
        @DisplayName("Should return false when tenant is suspended")
        void shouldReturnFalseWhenTenantIsSuspended() {
            // Given
            Tenant tenant = new Tenant();
            tenant.setStatus(Tenant.TenantStatus.SUSPENDED);
            
            // When & Then
            assertThat(tenant.isActive()).isFalse();
        }

        @Test
        @DisplayName("Should check if can add user")
        void shouldCheckIfCanAddUser() {
            // Given
            Tenant tenant = new Tenant();
            tenant.setMaxUsers(10);
            
            // When & Then
            assertThat(tenant.canAddUser()).isTrue();
        }

        @Test
        @DisplayName("Should check if can add strategy")
        void shouldCheckIfCanAddStrategy() {
            // Given
            Tenant tenant = new Tenant();
            tenant.setMaxStrategies(5);
            
            // When & Then
            assertThat(tenant.canAddStrategy()).isTrue();
        }

        @Test
        @DisplayName("Should check if can add asset subscription")
        void shouldCheckIfCanAddAssetSubscription() {
            // Given
            Tenant tenant = new Tenant();
            tenant.setMaxAssetSubscriptions(50);
            
            // When & Then
            assertThat(tenant.canAddAssetSubscription()).isTrue();
        }
    }

    @Nested
    @DisplayName("Validation Tests")
    class ValidationTests {

        @Test
        @DisplayName("Should handle null name")
        void shouldHandleNullName() {
            // Given
            Tenant tenant = new Tenant();
            
            // When
            tenant.setName(null);
            
            // Then
            assertThat(tenant.getName()).isNull();
        }

        @Test
        @DisplayName("Should handle empty name")
        void shouldHandleEmptyName() {
            // Given
            Tenant tenant = new Tenant();
            
            // When
            tenant.setName("");
            
            // Then
            assertThat(tenant.getName()).isEmpty();
        }

        @Test
        @DisplayName("Should handle long name")
        void shouldHandleLongName() {
            // Given
            Tenant tenant = new Tenant();
            String longName = "a".repeat(200);
            
            // When
            tenant.setName(longName);
            
            // Then
            assertThat(tenant.getName()).isEqualTo(longName);
        }

        @Test
        @DisplayName("Should handle null code")
        void shouldHandleNullCode() {
            // Given
            Tenant tenant = new Tenant();
            
            // When
            tenant.setCode(null);
            
            // Then
            assertThat(tenant.getCode()).isNull();
        }

        @Test
        @DisplayName("Should handle empty code")
        void shouldHandleEmptyCode() {
            // Given
            Tenant tenant = new Tenant();
            
            // When
            tenant.setCode("");
            
            // Then
            assertThat(tenant.getCode()).isEmpty();
        }

        @Test
        @DisplayName("Should handle long code")
        void shouldHandleLongCode() {
            // Given
            Tenant tenant = new Tenant();
            String longCode = "a".repeat(100);
            
            // When
            tenant.setCode(longCode);
            
            // Then
            assertThat(tenant.getCode()).isEqualTo(longCode);
        }

        @Test
        @DisplayName("Should handle null description")
        void shouldHandleNullDescription() {
            // Given
            Tenant tenant = new Tenant();
            
            // When
            tenant.setDescription(null);
            
            // Then
            assertThat(tenant.getDescription()).isNull();
        }

        @Test
        @DisplayName("Should handle empty description")
        void shouldHandleEmptyDescription() {
            // Given
            Tenant tenant = new Tenant();
            
            // When
            tenant.setDescription("");
            
            // Then
            assertThat(tenant.getDescription()).isEmpty();
        }

        @Test
        @DisplayName("Should handle long description")
        void shouldHandleLongDescription() {
            // Given
            Tenant tenant = new Tenant();
            String longDescription = "a".repeat(1000);
            
            // When
            tenant.setDescription(longDescription);
            
            // Then
            assertThat(tenant.getDescription()).isEqualTo(longDescription);
        }
    }

    @Nested
    @DisplayName("Equals and HashCode Tests")
    class EqualsAndHashCodeTests {

        @Test
        @DisplayName("Should return true when tenants have same ID")
        void shouldReturnTrueWhenTenantsHaveSameId() {
            // Given
            UUID id = UUID.randomUUID();
            Tenant tenant1 = new Tenant();
            tenant1.setId(id);
            Tenant tenant2 = new Tenant();
            tenant2.setId(id);
            
            // When & Then
            assertThat(tenant1).isEqualTo(tenant2);
        }

        @Test
        @DisplayName("Should return false when tenants have different IDs")
        void shouldReturnFalseWhenTenantsHaveDifferentIds() {
            // Given
            Tenant tenant1 = new Tenant();
            tenant1.setId(UUID.randomUUID());
            Tenant tenant2 = new Tenant();
            tenant2.setId(UUID.randomUUID());
            
            // When & Then
            assertThat(tenant1).isNotEqualTo(tenant2);
        }

        @Test
        @DisplayName("Should return same hash code for same tenant")
        void shouldReturnSameHashCodeForSameTenant() {
            // Given
            Tenant tenant = new Tenant();
            
            // When & Then
            assertThat(tenant.hashCode()).isEqualTo(tenant.hashCode());
        }
    }

    @Nested
    @DisplayName("ToString Tests")
    class ToStringTests {

        @Test
        @DisplayName("Should include key information in toString")
        void shouldIncludeKeyInformationInToString() {
            // Given
            Tenant tenant = new Tenant("Test Tenant", "TEST", Tenant.PlanType.PRO);
            
            // When
            String result = tenant.toString();
            
            // Then
            assertThat(result).contains("Test Tenant");
            assertThat(result).contains("TEST");
            assertThat(result).contains("PRO");
            assertThat(result).contains("ACTIVE");
        }
    }

    @Nested
    @DisplayName("Enum Tests")
    class EnumTests {

        @Test
        @DisplayName("Should return correct plan type values")
        void shouldReturnCorrectPlanTypeValues() {
            // When & Then
            assertThat(Tenant.PlanType.BASIC.getDisplayName()).isEqualTo("基础版");
            assertThat(Tenant.PlanType.PRO.getDisplayName()).isEqualTo("专业版");
            assertThat(Tenant.PlanType.ENTERPRISE.getDisplayName()).isEqualTo("企业版");
        }

        @Test
        @DisplayName("Should return correct status values")
        void shouldReturnCorrectStatusValues() {
            // When & Then
            assertThat(Tenant.TenantStatus.ACTIVE.getDisplayName()).isEqualTo("激活");
            assertThat(Tenant.TenantStatus.SUSPENDED.getDisplayName()).isEqualTo("暂停");
            assertThat(Tenant.TenantStatus.DELETED.getDisplayName()).isEqualTo("已删除");
        }

        @Test
        @DisplayName("Should return correct plan limits")
        void shouldReturnCorrectPlanLimits() {
            // When & Then
            assertThat(Tenant.PlanType.BASIC.getMaxUsers()).isEqualTo(10);
            assertThat(Tenant.PlanType.PRO.getMaxUsers()).isEqualTo(50);
            assertThat(Tenant.PlanType.ENTERPRISE.getMaxUsers()).isEqualTo(200);
        }
    }
} 