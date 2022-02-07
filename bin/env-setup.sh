#!/bin/sh
while read -r line; do export $line; done < local.env