export function createComponent<TRoot extends HTMLElement, TTemplate extends {root: TRoot}, TComponent, TServices extends {}>(
    cons: new (template: TTemplate, services: TServices) => TComponent,
    root: TRoot,
    services: TServices
): TComponent {
    const bindings: any = {root};
    root.querySelectorAll('[data-bind]').forEach(elem => {
        bindings[elem.getAttribute('data-bind')!] = elem;
        elem.removeAttribute('data-bind');
    });
    return new cons(bindings, services);
}
