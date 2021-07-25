/* Copyright (c) 2021 Niels Sonnich Poulsen (http://nielssp.dk)
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

export function createComponent<TRoot extends HTMLElement, TTemplate extends {root: TRoot}, TComponent, TData>(
    cons: new (template: TTemplate, data: TData) => TComponent,
    root: TRoot,
    data: TData
): TComponent {
    const bindings: any = {root};
    root.querySelectorAll('[data-bind]').forEach(elem => {
        bindings[elem.getAttribute('data-bind')!] = elem;
        elem.removeAttribute('data-bind');
    });
    return new cons(bindings, data);
}

export function appendComponent<TTemplate extends {root: HTMLElement}, TComponent, TData>(
    parent: HTMLElement,
    cons: new (template: TTemplate, data: TData) => TComponent,
    template: string,
    data: TData
): TComponent {
    const temp = document.createElement('div');
    temp.innerHTML = template;
    const root = temp.children[0];
    const bindings: any = {root};
    root.querySelectorAll('[data-bind]').forEach(elem => {
        bindings[elem.getAttribute('data-bind')!] = elem;
        elem.removeAttribute('data-bind');
    });
    parent.appendChild(root);
    return new cons(bindings, data);
}

export function prependComponent<TTemplate extends {root: HTMLElement}, TComponent, TData>(
    parent: HTMLElement,
    cons: new (template: TTemplate, data: TData) => TComponent,
    template: string,
    data: TData
): TComponent {
    const temp = document.createElement('div');
    temp.innerHTML = template;
    const root = temp.children[0];
    const bindings: any = {root};
    root.querySelectorAll('[data-bind]').forEach(elem => {
        bindings[elem.getAttribute('data-bind')!] = elem;
        elem.removeAttribute('data-bind');
    });
    if (parent.children.length) {
        parent.insertBefore(root, parent.children[0]);
    } else {
        parent.appendChild(root);
    }
    return new cons(bindings, data);
}
