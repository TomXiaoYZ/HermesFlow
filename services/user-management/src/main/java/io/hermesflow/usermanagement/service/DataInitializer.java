package io.hermesflow.usermanagement.service;

import io.hermesflow.usermanagement.model.Permission;
import io.hermesflow.usermanagement.model.Role;
import io.hermesflow.usermanagement.model.User;
import io.hermesflow.usermanagement.repository.PermissionRepository;
import io.hermesflow.usermanagement.repository.RoleRepository;
import io.hermesflow.usermanagement.repository.UserRepository;
import lombok.RequiredArgsConstructor;
import lombok.extern.slf4j.Slf4j;
import org.springframework.boot.CommandLineRunner;
import org.springframework.security.crypto.password.PasswordEncoder;
import org.springframework.stereotype.Component;
import org.springframework.transaction.annotation.Transactional;

@Component
@RequiredArgsConstructor
@Slf4j
public class DataInitializer implements CommandLineRunner {

    private final UserRepository userRepository;
    private final RoleRepository roleRepository;
    private final PermissionRepository permissionRepository;
    private final PasswordEncoder passwordEncoder;

    @org.springframework.beans.factory.annotation.Value("${admin.initial.password}")
    private String adminPassword;

    @Override
    @Transactional
    public void run(String... args) {
        log.info("Checking if admin user exists...");

        // Ensure Admin Role Exists
        Role adminRole = roleRepository.findByName("ADMIN")
            .orElseGet(() -> {
                log.info("Creating ADMIN role");
                return roleRepository.save(new Role("ADMIN"));
            });

        // Ensure Admin Permission Exists
        Permission adminPerm = permissionRepository.findByName("ALL_ACCESS")
            .orElseGet(() -> {
                log.info("Creating ALL_ACCESS permission");
                return permissionRepository.save(new Permission("ALL_ACCESS"));
            });
        
        // Link Permission to Role
        if (!adminRole.getPermissions().contains(adminPerm)) {
            adminRole.getPermissions().add(adminPerm);
            roleRepository.save(adminRole);
        }

        // Ensure Admin User Exists
        if (userRepository.findByUsername("admin").isEmpty()) {
            log.info("Creating default admin user");
            User admin = new User("admin", passwordEncoder.encode(adminPassword));
            admin.getRoles().add(adminRole);
            userRepository.save(admin);
            log.info("Admin user created successfully.");
        } else {
            log.info("Admin user already exists. Updating password from environment...");
            User admin = userRepository.findByUsername("admin").get();
            admin.setPassword(passwordEncoder.encode(adminPassword));
            userRepository.save(admin);
            log.info("Admin password updated.");
        }
    }
}
