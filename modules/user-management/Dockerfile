# Build stage
FROM eclipse-temurin:21-jdk AS builder
WORKDIR /app

# Copy Maven wrapper and pom.xml
COPY mvnw .
COPY .mvn .mvn
COPY pom.xml .

# Download dependencies
RUN chmod +x mvnw && ./mvnw dependency:go-offline -B

# Copy source code
COPY src ./src

# Build application
RUN ./mvnw package -DskipTests

# Runtime stage
FROM eclipse-temurin:21-jre
WORKDIR /app

# Copy jar from builder
COPY --from=builder /app/target/*.jar app.jar

EXPOSE 8010

ENTRYPOINT ["java", "-jar", "app.jar"]
