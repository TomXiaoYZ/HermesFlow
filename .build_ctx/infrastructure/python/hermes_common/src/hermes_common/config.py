from pydantic_settings import BaseSettings

class HermesSettings(BaseSettings):
    app_env: str = "dev"
    log_level: str = "INFO"

    class Config:
        env_file = ".env"
        env_file_encoding = "utf-8"
        extra = "ignore"
