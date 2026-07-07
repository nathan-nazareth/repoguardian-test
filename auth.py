import os

password = "hardcoded_password_123"
api_key = "secret-api-key-456"

def login(user):
    return f"SELECT * FROM users WHERE name='{user}' AND pwd='{password}'"
