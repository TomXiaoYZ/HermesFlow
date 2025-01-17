from setuptools import setup, find_packages

setup(
    name="hermesflow",
    version="0.1.0",
    packages=find_packages(),
    install_requires=[
        "aiohttp==3.9.3",
        "python-dotenv==1.0.1",
        "ujson==5.9.0",
        "pytest==8.0.2",
        "pytest-asyncio==0.23.5",
    ],
) 