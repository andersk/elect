#!/usr/bin/env python

from setuptools import setup

setup(
    name='elect',
    version='1.0.0',
    description='Command line tool for tallying elections with the Schulze STV method',
    url='https://github.com/andersk/elect',
    author='Anders Kaseorg',
    author_email='andersk@mit.edu',
    license='MIT',
    classifiers=[
        'Environment :: Console',
        'Topic :: Scientific/Engineering :: Mathematics',
        'License :: OSI Approved :: MIT License',
        'Programming Language :: Python :: 2',
    ],
    py_modules=['elect'],
    install_requires=['python-vote-core'],
    entry_points={
        'console_scripts': [
            'elect=elect:main',
        ],
    },
)
