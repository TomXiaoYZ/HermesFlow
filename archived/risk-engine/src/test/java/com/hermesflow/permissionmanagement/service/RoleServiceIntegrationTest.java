package com.hermesflow.permissionmanagement.service;

import com.hermesflow.permissionmanagement.dto.RoleCreateDTO;
import com.hermesflow.permissionmanagement.dto.RoleDTO;
import com.hermesflow.permissionmanagement.dto.RoleUpdateDTO;
import com.hermesflow.permissionmanagement.entity.Role;
import com.hermesflow.permissionmanagement.repository.RoleRepository;
import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.Test;
import org.springframework.beans.factory.annotation.Autowired;
import org.springframework.boot.test.context.SpringBootTest;
import org.springframework.data.domain.Page;
import org.springframework.data.domain.PageRequest;
import org.springframework.test.context.ActiveProfiles;
import org.springframework.transaction.annotation.Transactional;

import java.util.List;
import java.util.UUID;

import static org.assertj.core.api.Assertions.assertThat;

/**
 * 角色服务集成测试
 */
@SpringBootTest
@ActiveProfiles("test")
@Transactional
class RoleServiceIntegrationTest {

    @Autowired
    private RoleService roleService;

    @Autowired
    private RoleRepository roleRepository;

    private UUID tenantId;
    private UUID userId;

    @BeforeEach
    void setUp() {
        tenantId = UUID.fromString("00000000-0000-0000-0000-000000000000");
        userId = UUID.randomUUID();
        
        // 清理测试数据
        roleRepository.deleteAll();
    }

    @Test
    void testCreateRole() {
        // 准备测试数据
        RoleCreateDTO createDTO = new RoleCreateDTO();
        createDTO.setRoleCode("TEST_ADMIN");
        createDTO.setRoleName("测试管理员");
        createDTO.setDescription("用于测试的管理员角色");
        createDTO.setRoleType(Role.RoleType.BUSINESS);

        // 执行测试
        RoleDTO result = roleService.createRole(tenantId, createDTO, userId);

        // 验证结果
        assertThat(result).isNotNull();
        assertThat(result.getRoleCode()).isEqualTo("TEST_ADMIN");
        assertThat(result.getRoleName()).isEqualTo("测试管理员");
        assertThat(result.getRoleType()).isEqualTo(Role.RoleType.BUSINESS);
        assertThat(result.isActive()).isTrue();
    }

    @Test
    void testGetRoleById() {
        // 准备测试数据
        Role role = createTestRole();
        role = roleRepository.save(role);

        // 执行测试
        RoleDTO result = roleService.getRoleById(role.getId());

        // 验证结果
        assertThat(result).isNotNull();
        assertThat(result.getId()).isEqualTo(role.getId());
        assertThat(result.getRoleCode()).isEqualTo(role.getRoleCode());
    }

    @Test
    void testUpdateRole() {
        // 准备测试数据
        Role role = createTestRole();
        role = roleRepository.save(role);

        RoleUpdateDTO updateDTO = new RoleUpdateDTO();
        updateDTO.setRoleName("更新后的角色名称");
        updateDTO.setDescription("更新后的描述");

        // 执行测试
        RoleDTO result = roleService.updateRole(role.getId(), updateDTO, userId);

        // 验证结果
        assertThat(result).isNotNull();
        assertThat(result.getRoleName()).isEqualTo("更新后的角色名称");
        assertThat(result.getDescription()).isEqualTo("更新后的描述");
    }

    @Test
    void testDeleteRole() {
        // 准备测试数据
        Role role = createTestRole();
        role = roleRepository.save(role);

        // 执行测试
        roleService.deleteRole(role.getId(), userId);

        // 验证结果
        Role deletedRole = roleRepository.findById(role.getId()).orElse(null);
        assertThat(deletedRole).isNotNull();
        assertThat(deletedRole.isActive()).isFalse();
    }

    @Test
    void testGetRolesByTenant() {
        // 准备测试数据
        Role role1 = createTestRole("TEST_ADMIN_1", "测试管理员1");
        Role role2 = createTestRole("TEST_ADMIN_2", "测试管理员2");
        roleRepository.saveAll(List.of(role1, role2));

        // 执行测试
        Page<RoleDTO> result = roleService.getRolesByTenant(
                tenantId, PageRequest.of(0, 10));

        // 验证结果
        assertThat(result).isNotNull();
        assertThat(result.getContent()).hasSize(2);
        assertThat(result.getContent())
                .extracting(RoleDTO::getRoleCode)
                .containsExactlyInAnyOrder("TEST_ADMIN_1", "TEST_ADMIN_2");
    }

    @Test
    void testSearchRoles() {
        // 准备测试数据
        Role role1 = createTestRole("USER_ADMIN", "用户管理员");
        Role role2 = createTestRole("USER_VIEWER", "用户查看者");
        Role role3 = createTestRole("SYSTEM_ADMIN", "系统管理员");
        roleRepository.saveAll(List.of(role1, role2, role3));

        // 执行测试 - 搜索包含"用户"的角色
        Page<RoleDTO> result = roleService.searchRoles(
                tenantId, "用户", PageRequest.of(0, 10));

        // 验证结果
        assertThat(result).isNotNull();
        assertThat(result.getContent()).hasSize(2);
        assertThat(result.getContent())
                .extracting(RoleDTO::getRoleName)
                .allMatch(name -> name.contains("用户"));
    }

    @Test
    void testGetRolesByType() {
        // 准备测试数据
        Role systemRole = createTestRole("SYSTEM_ADMIN", "系统管理员");
        systemRole.setRoleType(Role.RoleType.SYSTEM);
        
        Role businessRole = createTestRole("BUSINESS_ADMIN", "业务管理员");
        businessRole.setRoleType(Role.RoleType.BUSINESS);
        
        roleRepository.saveAll(List.of(systemRole, businessRole));

        // 执行测试
        List<RoleDTO> result = roleService.getRolesByType(tenantId, Role.RoleType.SYSTEM);

        // 验证结果
        assertThat(result).hasSize(1);
        assertThat(result.get(0).getRoleType()).isEqualTo(Role.RoleType.SYSTEM);
        assertThat(result.get(0).getRoleCode()).isEqualTo("SYSTEM_ADMIN");
    }

    @Test
    void testCreateRoleHierarchy() {
        // 准备测试数据 - 创建父角色
        RoleCreateDTO parentRoleDTO = new RoleCreateDTO();
        parentRoleDTO.setRoleCode("PARENT_ADMIN");
        parentRoleDTO.setRoleName("父级管理员");
        parentRoleDTO.setDescription("父级管理员角色");
        parentRoleDTO.setRoleType(Role.RoleType.BUSINESS);

        RoleDTO parentRole = roleService.createRole(tenantId, parentRoleDTO, userId);

        // 创建子角色
        RoleCreateDTO childRoleDTO = new RoleCreateDTO();
        childRoleDTO.setRoleCode("CHILD_ADMIN");
        childRoleDTO.setRoleName("子级管理员");
        childRoleDTO.setDescription("子级管理员角色");
        childRoleDTO.setRoleType(Role.RoleType.BUSINESS);
        childRoleDTO.setParentRoleId(parentRole.getId());

        // 执行测试
        RoleDTO childRole = roleService.createRole(tenantId, childRoleDTO, userId);

        // 验证结果
        assertThat(childRole).isNotNull();
        assertThat(childRole.getParentRoleId()).isEqualTo(parentRole.getId());
    }

    @Test
    void testActivateRole() {
        // 准备测试数据
        Role role = createTestRole();
        role.setActive(false);
        role = roleRepository.save(role);

        // 执行测试
        roleService.activateRole(role.getId(), userId);

        // 验证结果
        Role activatedRole = roleRepository.findById(role.getId()).orElse(null);
        assertThat(activatedRole).isNotNull();
        assertThat(activatedRole.isActive()).isTrue();
    }

    @Test
    void testDeactivateRole() {
        // 准备测试数据
        Role role = createTestRole();
        role = roleRepository.save(role);

        // 执行测试
        roleService.deactivateRole(role.getId(), userId);

        // 验证结果
        Role deactivatedRole = roleRepository.findById(role.getId()).orElse(null);
        assertThat(deactivatedRole).isNotNull();
        assertThat(deactivatedRole.isActive()).isFalse();
    }

    private Role createTestRole() {
        return createTestRole("TEST_ADMIN", "测试管理员");
    }

    private Role createTestRole(String code, String name) {
        Role role = new Role();
        role.setTenantId(tenantId);
        role.setRoleCode(code);
        role.setRoleName(name);
        role.setDescription("测试角色描述");
        role.setRoleType(Role.RoleType.BUSINESS);
        role.setActive(true);
        role.setCreatedBy(userId);
        role.setUpdatedBy(userId);
        return role;
    }
} 