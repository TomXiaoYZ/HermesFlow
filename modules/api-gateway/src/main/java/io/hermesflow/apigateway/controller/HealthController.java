package io.hermesflow.apigateway.controller;

import org.springframework.web.bind.annotation.GetMapping;
import org.springframework.web.bind.annotation.RestController;
import java.util.Map;

@RestController
public class HealthController {
    
    @GetMapping("/health")
    public Map<String, String> health() {
        return Map.of(
            "status", "healthy",
            "service", "api-gateway",
            "version", "0.1.0"
        );
    }
}
