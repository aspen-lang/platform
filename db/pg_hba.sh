#!/bin/env bash

echo 'host postgres postgres 172.21.0.0/16 trust' >> /var/lib/postgresql/data/pg_hba.conf
