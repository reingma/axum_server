#!/usr/bin/env bash
wrk -t12 -c400 -d10s http://128.0.0.1:8000
