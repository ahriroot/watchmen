from setuptools import setup

with open("README_EN.md", "r", encoding="utf-8") as fh:
    long_description = fh.read()

setup(
    name="watchmen-rust",
    version="0.0.1",
    author="ahriknow",
    author_email="ahriknow@ahriknow.com",
    description="Watchmen is a daemon process manager that for you manage and keep your application online 24/7",
    long_description=long_description,
    long_description_content_type="text/markdown",
    license="Apache-2.0",
    keywords=[],
    classifiers=[
        "Development Status :: 5 - Production/Stable",
        "License :: OSI Approved :: MIT License",
        "Programming Language :: Python :: 3",
        "Programming Language :: Python :: 3.5",
        "Programming Language :: Python :: 3.6",
        "Programming Language :: Python :: 3.7",
        "Programming Language :: Python :: 3.8",
        "Programming Language :: Python :: 3.9",
        "Programming Language :: Python :: 3.10",
        "Programming Language :: Python :: 3.11",
        "Programming Language :: Python :: 3.12",
    ],
    packages=[
        "target/release/watchmen",
        "target/release/watchmend",
        "LICENSE",
        "README_EN.md",
        "README.md",
        "setup.py",
    ],
    entry_points={
        "console_scripts": [
            "watchmen=target/release/watchmen",
            "watchmend=target/release/watchmend",
        ],
    },
    python_requires=">=3.0",
)
