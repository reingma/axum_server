#!/usr/bin/env bash
##simple test for where we do 10000 requests, 100 in parallel
ab -c 100 -n 10000 http://127.0.0.1:8000/health_check 
