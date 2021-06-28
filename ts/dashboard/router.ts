export interface Page {
    enter(args: Record<string, string|number>): void;
    leave(): void;
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

export type Path = string|(string|number)[];

export class Router {
    private activePage?: Page;
    private root: Route;

    constructor(config: RouterConfig) {
        this.root = this.createRoutes(config);
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

    navigate(path: Path): void {
        if (typeof path === 'string') {
            path = path.split('/').filter(s => s);
        }
        let route = this.root;
        const args: Record<string, string|number> = {};
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
                return;
            }
        }
        if (!route.page) {
            console.warn(`no page for route: ${path.join('/')}`);
            return;
        }
        try {
            if (this.activePage) {
                this.activePage.leave();
            }
            this.activePage = route.page;
            this.activePage.enter(args);
        } catch (error) {
            console.error('Invalid args for route', path, error);
        }
    }
}
