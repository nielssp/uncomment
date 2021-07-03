/* Copyright (c) 2021 Niels Sonnich Poulsen (http://nielssp.dk)
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

export interface Page {
    enter(args: Record<string, string>): void;
    leave(): void;
    get args(): Record<string, string>;
}

export type PageConstructor = (router: Router) => Page;

export interface Route {
    varName?: string;
    page?: Page;
    children: Record<string, Route>;
    catchAll?: Route;
}

export interface RouterConfig {
    [propName: string]: RouterConfig|PageConstructor;
}

export type Path = string[];

export class Router {
    private activePage?: Page;
    private root: Route;

    constructor(config: RouterConfig) {
        this.root = this.createRoutes(config);
        window.onhashchange = () => this.restore();
    }

    createRoutes(config: RouterConfig|PageConstructor, path: string = '/'): Route {
        const route: Route = {children: {}};
        if (typeof config === 'function') {
            route.page = config(this);
        } else {
            for (let prop in config) {
                if (config.hasOwnProperty(prop)) {
                    if (prop === '') {
                        if (typeof config[''] === 'function') {
                            route.page = config[''](this);
                        } else {
                            throw new Error('Invalid page constructor for: ' + path);
                        }
                    } else if (prop.startsWith('$')) {
                        if (route.catchAll) {
                            throw new Error('Multiple placeholders in route: ' + path);
                        }
                        route.catchAll = this.createRoutes(config[prop], path + prop + '/');
                        route.catchAll.varName = prop.replace(/^\$/, '');
                    } else {
                        route.children[prop] = this.createRoutes(config[prop], path + prop + '/');
                    }
                }
            }
        }
        return route;
    }

    restore() {
        if (window.location.hash) {
            const {path, args} = stringToPath(window.location.hash.replace(/^#/, ''));
            return !!this.open(path, args);
        }
        return false;
    }

    replaceState(path: Path, args: Record<string, string> = {}) {
        window.history.replaceState({path, args}, document.title, pathToString(path, args));
    }

    pushState(path: Path, args: Record<string, string> = {}) {
        window.history.pushState({path, args}, document.title, pathToString(path, args));
    }

    navigate(path: Path, args: Record<string, string> = {}): boolean {
        const page = this.open(path, args);
        if (page) {
            this.pushState(path, page.args);
            return true;
        }
        return false;
    }

    open(path: Path, args: Record<string, string> = {}): Page|undefined {
        let route = this.root;
        for (let split of path) {
            if (route.children.hasOwnProperty(split)) {
                route = route.children[split];
            } else if (route.catchAll) {
                route = route.catchAll;
                if (route.varName) {
                    args[route.varName] = split;
                }
            } else {
                console.warn(`route not found: ${path.join('/')}`);
                return undefined;
            }
        }
        if (!route.page) {
            console.warn(`no page for route: ${path.join('/')}`);
            return undefined;
        }
        try {
            if (this.activePage) {
                this.activePage.leave();
            }
            this.activePage = route.page;
            this.activePage.enter(args);
            return this.activePage;
        } catch (error) {
            console.error('Invalid args for route', path, error);
            return undefined;
        }
    }
}

function pathToString(path: Path, args: Record<string, string>) {
    let str = '#' + path.join('/');
    let query: string[] = [];
    for (let key in args) {
        if (args.hasOwnProperty(key)) {
            if (args[key]) {
                query.push(`${encodeURIComponent(key)}=${encodeURIComponent(args[key])}`);
            } else {
                query.push(`${encodeURIComponent(key)}`);
            }
        }
    }
    if (query.length) {
        str += `?${query.join('&')}`;
    }
    return str;
}

function stringToPath(str: string): {path: Path, args: Record<string, string>} {
    const splits = str.split('?');
    const path = splits[0].split('/');
    const args: Record<string, string> = {};
    if (splits.length > 1) {
        const query = splits[1].split('&');
        for (let keyValue of query) {
            if (keyValue.includes('=')) {
                const [key, value] = keyValue.split('=');
                args[decodeURIComponent(key)] = decodeURIComponent(value);
            } else {
                args[decodeURIComponent(keyValue)] = '';
            }
        }
    }
    return {path, args};
}
