package com.hermesflow.usermanagement.entity;

import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.DisplayName;
import org.junit.jupiter.api.Nested;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.params.ParameterizedTest;
import org.junit.jupiter.params.provider.ValueSource;

import jakarta.validation.ConstraintViolation;
import jakarta.validation.Validation;
import jakarta.validation.Validator;
import jakarta.validation.ValidatorFactory;

import java.time.LocalDateTime;
import java.util.Set;
import java.util.UUID;

import static org.junit.jupiter.api.Assertions.*;

@DisplayName("TenantConfig Entity Tests")
class TenantConfigTest {

    private Validator validator;
    private Tenant testTenant;

    @BeforeEach
    void setUp() {
        ValidatorFactory factory = Validation.buildDefaultValidatorFactory();
        validator = factory.getValidator();
        
        // 创建测试租户
        testTenant = new Tenant();
        testTenant.setId(UUID.randomUUID());
        testTenant.setName("Test Tenant");
        testTenant.setCode("TEST");
    }

    @Nested
    @DisplayName("Constructor Tests")
    class ConstructorTests {

        @Test
        @DisplayName("Default constructor should create empty config")
        void testDefaultConstructor() {
            TenantConfig config = new TenantConfig();
            
            assertNull(config.getId());
            assertNull(config.getTenant());
            assertNull(config.getConfigKey());
            assertNull(config.getConfigValue());
            assertEquals("string", config.getConfigType());
            assertFalse(config.getIsEncrypted());
            assertNull(config.getDescription());
        }

        @Test
        @DisplayName("Three-parameter constructor should set basic properties")
        void testThreeParameterConstructor() {
            TenantConfig config = new TenantConfig(testTenant, "test.key", "test.value");
            
            assertEquals(testTenant, config.getTenant());
            assertEquals("test.key", config.getConfigKey());
            assertEquals("test.value", config.getConfigValue());
            assertEquals("string", config.getConfigType());
        }

        @Test
        @DisplayName("Four-parameter constructor should set all properties")
        void testFourParameterConstructor() {
            TenantConfig config = new TenantConfig(testTenant, "test.key", "123", "integer");
            
            assertEquals(testTenant, config.getTenant());
            assertEquals("test.key", config.getConfigKey());
            assertEquals("123", config.getConfigValue());
            assertEquals("integer", config.getConfigType());
        }
    }

    @Nested
    @DisplayName("Type Checking Methods")
    class TypeCheckingTests {

        @Test
        @DisplayName("isStringType should return true for string type")
        void testIsStringType() {
            TenantConfig config = new TenantConfig(testTenant, "key", "value", "string");
            assertTrue(config.isStringType());
            
            config.setConfigType("integer");
            assertFalse(config.isStringType());
        }

        @Test
        @DisplayName("isIntegerType should return true for integer type")
        void testIsIntegerType() {
            TenantConfig config = new TenantConfig(testTenant, "key", "123", "integer");
            assertTrue(config.isIntegerType());
            
            config.setConfigType("string");
            assertFalse(config.isIntegerType());
        }

        @Test
        @DisplayName("isBooleanType should return true for boolean type")
        void testIsBooleanType() {
            TenantConfig config = new TenantConfig(testTenant, "key", "true", "boolean");
            assertTrue(config.isBooleanType());
            
            config.setConfigType("string");
            assertFalse(config.isBooleanType());
        }

        @Test
        @DisplayName("isJsonType should return true for json type")
        void testIsJsonType() {
            TenantConfig config = new TenantConfig(testTenant, "key", "{}", "json");
            assertTrue(config.isJsonType());
            
            config.setConfigType("string");
            assertFalse(config.isJsonType());
        }
    }

    @Nested
    @DisplayName("Value Conversion Methods")
    class ValueConversionTests {

        @Test
        @DisplayName("getIntegerValue should return integer for valid integer type")
        void testGetIntegerValueValid() {
            TenantConfig config = new TenantConfig(testTenant, "key", "123", "integer");
            assertEquals(Integer.valueOf(123), config.getIntegerValue());
        }

        @Test
        @DisplayName("getIntegerValue should return null for invalid integer value")
        void testGetIntegerValueInvalid() {
            TenantConfig config = new TenantConfig(testTenant, "key", "invalid", "integer");
            assertNull(config.getIntegerValue());
        }

        @Test
        @DisplayName("getIntegerValue should return null for non-integer type")
        void testGetIntegerValueWrongType() {
            TenantConfig config = new TenantConfig(testTenant, "key", "123", "string");
            assertNull(config.getIntegerValue());
        }

        @Test
        @DisplayName("getBooleanValue should return boolean for boolean type")
        void testGetBooleanValueValid() {
            TenantConfig trueConfig = new TenantConfig(testTenant, "key1", "true", "boolean");
            TenantConfig falseConfig = new TenantConfig(testTenant, "key2", "false", "boolean");
            
            assertTrue(trueConfig.getBooleanValue());
            assertFalse(falseConfig.getBooleanValue());
        }

        @Test
        @DisplayName("getBooleanValue should return null for non-boolean type")
        void testGetBooleanValueWrongType() {
            TenantConfig config = new TenantConfig(testTenant, "key", "true", "string");
            assertNull(config.getBooleanValue());
        }

        @Test
        @DisplayName("getIntegerValue should handle null config value")
        void testGetIntegerValueWithNullValue() {
            TenantConfig config = new TenantConfig(testTenant, "key", null, "integer");
            
            assertNull(config.getIntegerValue());
        }
    }

    @Nested
    @DisplayName("Validation Tests")
    class ValidationTests {

        @Test
        @DisplayName("Valid config should pass validation")
        void testValidConfig() {
            TenantConfig config = new TenantConfig(testTenant, "test.key", "test.value");
            
            Set<ConstraintViolation<TenantConfig>> violations = validator.validate(config);
            assertTrue(violations.isEmpty());
        }

        @Test
        @DisplayName("Config with null tenant should fail validation")
        void testNullTenant() {
            TenantConfig config = new TenantConfig(null, "test.key", "test.value");
            
            Set<ConstraintViolation<TenantConfig>> violations = validator.validate(config);
            assertFalse(violations.isEmpty());
            assertTrue(violations.stream().anyMatch(v -> v.getMessage().contains("租户ID不能为空")));
        }

        @Test
        @DisplayName("Config with blank key should fail validation")
        void testBlankConfigKey() {
            TenantConfig config = new TenantConfig(testTenant, "", "test.value");
            
            Set<ConstraintViolation<TenantConfig>> violations = validator.validate(config);
            assertFalse(violations.isEmpty());
            assertTrue(violations.stream().anyMatch(v -> v.getMessage().contains("配置键不能为空")));
        }

        @Test
        @DisplayName("Config with blank value should fail validation")
        void testBlankConfigValue() {
            TenantConfig config = new TenantConfig(testTenant, "test.key", "");
            
            Set<ConstraintViolation<TenantConfig>> violations = validator.validate(config);
            assertFalse(violations.isEmpty());
            assertTrue(violations.stream().anyMatch(v -> v.getMessage().contains("配置值不能为空")));
        }

        @ParameterizedTest
        @ValueSource(strings = {"aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa", "very_long_key_that_exceeds_the_maximum_allowed_length_of_one_hundred_characters_for_config_key_definitely"})
        @DisplayName("Config with oversized key should fail validation")
        void testOversizedConfigKey(String longKey) {
            TenantConfig config = new TenantConfig(testTenant, longKey, "test.value");
            
            Set<ConstraintViolation<TenantConfig>> violations = validator.validate(config);
            assertFalse(violations.isEmpty());
            assertTrue(violations.stream().anyMatch(v -> v.getMessage().contains("配置键长度不能超过100个字符")));
        }

        @Test
        @DisplayName("Config with oversized type should fail validation")
        void testOversizedConfigType() {
            TenantConfig config = new TenantConfig(testTenant, "test.key", "test.value");
            config.setConfigType("very_long_type_name_that_exceeds_twenty_characters");
            
            Set<ConstraintViolation<TenantConfig>> violations = validator.validate(config);
            assertFalse(violations.isEmpty());
            assertTrue(violations.stream().anyMatch(v -> v.getMessage().contains("配置类型长度不能超过20个字符")));
        }
    }

    @Nested
    @DisplayName("Equality and Hash Code Tests")
    class EqualityTests {

        @Test
        @DisplayName("Configs with same ID should be equal")
        void testEqualityWithSameId() {
            UUID id = UUID.randomUUID();
            TenantConfig config1 = new TenantConfig(testTenant, "key1", "value1");
            TenantConfig config2 = new TenantConfig(testTenant, "key2", "value2");
            config1.setId(id);
            config2.setId(id);
            
            assertEquals(config1, config2);
            assertEquals(config1.hashCode(), config2.hashCode());
        }

        @Test
        @DisplayName("Configs with different IDs should not be equal")
        void testEqualityWithDifferentIds() {
            TenantConfig config1 = new TenantConfig(testTenant, "key", "value");
            TenantConfig config2 = new TenantConfig(testTenant, "key", "value");
            config1.setId(UUID.randomUUID());
            config2.setId(UUID.randomUUID());
            
            assertNotEquals(config1, config2);
        }

        @Test
        @DisplayName("Config with null ID should not be equal to config with ID")
        void testEqualityWithNullId() {
            TenantConfig config1 = new TenantConfig(testTenant, "key", "value");
            TenantConfig config2 = new TenantConfig(testTenant, "key", "value");
            config2.setId(UUID.randomUUID());
            
            assertNotEquals(config1, config2);
        }

        @Test
        @DisplayName("Config should be equal to itself")
        void testEqualityWithSelf() {
            TenantConfig config = new TenantConfig(testTenant, "key", "value");
            assertEquals(config, config);
        }

        @Test
        @DisplayName("Config should not be equal to null or different type")
        void testEqualityWithNullAndDifferentType() {
            TenantConfig config = new TenantConfig(testTenant, "key", "value");
            
            assertNotEquals(config, null);
            assertNotEquals(config, "string");
        }
    }

    @Nested
    @DisplayName("ToString Tests")
    class ToStringTests {

        @Test
        @DisplayName("toString should include key information")
        void testToString() {
            TenantConfig config = new TenantConfig(testTenant, "test.key", "test.value", "string");
            config.setId(UUID.randomUUID());
            config.setIsEncrypted(true);
            
            String toString = config.toString();
            
            assertAll(
                () -> assertTrue(toString.contains("TenantConfig")),
                () -> assertTrue(toString.contains("id=")),
                () -> assertTrue(toString.contains("configKey='test.key'")),
                () -> assertTrue(toString.contains("configType='string'")),
                () -> assertTrue(toString.contains("isEncrypted=true"))
            );
        }
    }

    @Nested
    @DisplayName("Audit Fields Tests")
    class AuditFieldsTests {

        @Test
        @DisplayName("Audit fields should be settable")
        void testAuditFields() {
            TenantConfig config = new TenantConfig(testTenant, "key", "value");
            LocalDateTime now = LocalDateTime.now();
            
            config.setCreatedAt(now);
            config.setUpdatedAt(now);
            
            assertEquals(now, config.getCreatedAt());
            assertEquals(now, config.getUpdatedAt());
        }
    }

    @Nested
    @DisplayName("Edge Cases Tests")
    class EdgeCasesTests {

        @Test
        @DisplayName("Config should handle null encryption flag")
        void testNullEncryptionFlag() {
            TenantConfig config = new TenantConfig(testTenant, "key", "value");
            config.setIsEncrypted(null);
            
            assertNull(config.getIsEncrypted());
        }

        @Test
        @DisplayName("Config should handle null config type")
        void testNullConfigType() {
            TenantConfig config = new TenantConfig(testTenant, "key", "value");
            config.setConfigType(null);
            
            assertNull(config.getConfigType());
            assertFalse(config.isStringType());
            assertFalse(config.isIntegerType());
            assertFalse(config.isBooleanType());
            assertFalse(config.isJsonType());
        }

        @Test
        @DisplayName("getBooleanValue should handle null config value")
        void testGetBooleanValueWithNullValue() {
            TenantConfig config = new TenantConfig(testTenant, "key", null, "boolean");
            
            assertFalse(config.getBooleanValue());
        }
    }
} 