import os

password = os.environ.get("APP_DB_PASSWORD")
api_key = os.environ.get("APP_API_KEY")

if not password:
    raise RuntimeError("Environment variable APP_DB_PASSWORD is required")
if not api_key:
    raise RuntimeError("Environment variable APP_API_KEY is required")


def login(user, pwd):
    query = "SELECT * FROM users WHERE name=? AND pwd=?"
    return query, [user, pwd]
