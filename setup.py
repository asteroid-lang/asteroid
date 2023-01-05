from setuptools import setup, find_packages

with open("README.md", "r") as fh:
    long_description = fh.read()

setup(
    name="asteroid-lang",
    version="1.1.4",
    author="University of Rhode Island",
    author_email="lutzhamel@uri.edu",
    description="A modern, multi-paradigm programming language.",
    long_description=long_description,
    long_description_content_type="text/markdown",
    url="https://asteroid-lang.org",
    project_urls={
        "Documentation": "https://asteroid-lang.readthedocs.io",
        "Bug Tracker":   "https://github.com/asteroid-lang/asteroid/issues",
    },
    packages=find_packages(),
    package_data={"asteroid": ["modules/*"]},
    install_requires=[
        "pandas",
        "numpy",
        "matplotlib"
    ],
    classifiers=[
        "Programming Language :: Python :: 3",
        "License :: OSI Approved :: MIT License",
        "Operating System :: OS Independent",
    ],
    python_requires=">=3.6",
    entry_points={
        "console_scripts": [
            "asteroid = asteroid:main",
        ],
    },
)
