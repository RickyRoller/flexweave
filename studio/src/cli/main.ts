#!/usr/bin/env bun

import { studioPackageStatus } from "../index";

const status = studioPackageStatus();

console.log(`${status.surface}: ${status.message}`);
