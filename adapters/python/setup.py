from setuptools import setup, find_packages

setup(
    name="trace-py",
    version="1.0.0",
    description="Python adapter for griid-trace - Local-first AI agent observability",
    author="griid-trace",
    url="https://github.com/griid-trace/griid-trace",
    packages=find_packages(),
    classifiers=[
        "Development Status :: 4 - Beta",
        "Intended Audience :: Developers",
        "License :: OSI Approved :: MIT License",
        "Programming Language :: Python :: 3",
        "Programming Language :: Python :: 3.8",
        "Programming Language :: Python :: 3.9",
        "Programming Language :: Python :: 3.10",
        "Programming Language :: Python :: 3.11",
    ],
    python_requires=">=3.8",
    extras_require={
        "dev": ["pytest>=7.0"],
    },
)
