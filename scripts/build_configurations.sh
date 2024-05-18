#!/usr/bin/env bash
pkl eval -f yaml configuration/base.pkl > configuration/base.yaml
pkl eval -f yaml configuration/development.pkl > configuration/development.yaml
pkl eval -f yaml configuration/production.pkl > configuration/production.yaml
