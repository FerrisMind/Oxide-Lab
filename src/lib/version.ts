/**
 * Application version utilities
 * 
 * Automatically reads version from package.json
 */

import packageJson from '../../package.json';

/**
 * Application version from package.json
 */
export const APP_VERSION = packageJson.version;

/**
 * Application name from package.json
 */
export const APP_NAME = packageJson.name;

/**
 * Application description from package.json
 */
export const APP_DESCRIPTION = packageJson.description;

/**
 * Application license from package.json
 */
export const APP_LICENSE = packageJson.license;
