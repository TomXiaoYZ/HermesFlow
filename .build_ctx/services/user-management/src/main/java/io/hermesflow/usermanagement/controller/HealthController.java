package io.hermesflow.usermanagement.controller;

import org.springframework.web.bind.annotation.GetMapping;
import org.springframework.web.bind.annotation.RestController;
import java.util.Map;

@RestController
public class HealthController {
    
    @GetMapping("/health")
    public Map<String, String> health() {
        return Map.of(
            "status", "healthy",
            "service", "user-management",
            "version", "0.1.0"
        );
    }
}
