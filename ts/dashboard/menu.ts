import { Api } from "./api";
import { Auth, User } from "./auth";
import { Router } from "./router";

export class Menu {
    constructor(
        private template: {
            root: HTMLElement,
            comments: HTMLLinkElement,
            threads: HTMLLinkElement,
            users: HTMLLinkElement,
            changePassword: HTMLLinkElement,
            logOut: HTMLLinkElement,
            logIn: HTMLLinkElement,
        },
        private services: {
            auth: Auth,
            router: Router,
        },
    ) {
        template.comments.onclick = e => this.comments(e);
        template.changePassword.onclick = e => this.changePassword(e);
        template.logOut.onclick = e => this.logOut(e);
        template.logIn.onclick = e => this.logIn(e);
        services.auth.userChange.observe(user => this.userChange(user));
    }

    comments(e: MouseEvent) {
        e.preventDefault();
        this.services.router.navigate(['comments']);
    }

    changePassword(e: MouseEvent) {
        e.preventDefault();
        this.services.router.navigate(['change-password']);
    }

    async logOut(e: MouseEvent) {
        e.preventDefault();
        this.services.auth.logOut();
        this.services.router.navigate([]);
    }

    async logIn(e: MouseEvent) {
        e.preventDefault();
        this.services.router.navigate([]);
    }

    userChange(user: User|undefined) {
        if (user) {
            this.template.comments.style.display = '';
            this.template.threads.style.display = '';
            this.template.users.style.display = '';
            this.template.changePassword.style.display = '';
            this.template.logOut.style.display = '';
            this.template.logIn.style.display = 'none';
        } else {
            this.template.comments.style.display = 'none';
            this.template.threads.style.display = 'none';
            this.template.users.style.display = 'none';
            this.template.changePassword.style.display = 'none';
            this.template.logOut.style.display = 'none';
            this.template.logIn.style.display = '';
        }
    }
}
