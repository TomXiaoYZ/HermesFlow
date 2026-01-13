package io.hermesflow.usermanagement.controller;

import org.junit.jupiter.api.Test;
import org.springframework.beans.factory.annotation.Autowired;
import org.springframework.boot.test.context.SpringBootTest;
import org.springframework.boot.test.web.client.TestRestTemplate;
import org.springframework.http.ResponseEntity;

import java.util.Map;

import static org.assertj.core.api.Assertions.assertThat;

@SpringBootTest(webEnvironment = SpringBootTest.WebEnvironment.RANDOM_PORT)
class HealthControllerTest {

    @Autowired
    private TestRestTemplate restTemplate;

    @Test
    void shouldReturnHealthy() {
        ResponseEntity<Map> response = restTemplate.getForEntity("/health", Map.class);
        
        assertThat(response.getStatusCode().is2xxSuccessful()).isTrue();
        Map<String, String> body = response.getBody();
        assertThat(body).isNotNull();
        assertThat(body.get("status")).isEqualTo("healthy");
        assertThat(body.get("service")).isEqualTo("user-management");
    }
}
