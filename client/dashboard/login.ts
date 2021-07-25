/* Copyright (c) 2021 Niels Sonnich Poulsen (http://nielssp.dk)
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

import { Auth } from "./auth";
import { Page, Router } from "./router";

export class Login implements Page {
    constructor(
        private template: {
            root: HTMLElement,
            form: HTMLFormElement,
            info: HTMLElement,
            submit: HTMLButtonElement,
        },
        private services: {
            auth: Auth,
            router: Router,
        },
    ) {
        template.info.style.display = 'none';
        template.form.addEventListener('submit', e => this.submit(e));
    }

    enter(): void {
        this.template.root.style.display = '';
        this.template.info.style.display = 'none';
    }

    leave(): void {
        this.template.root.style.display = 'none';
    }

    get args() {
        return {};
    }

    async submit(e: Event) {
        e.preventDefault();
        this.template.submit.disabled = true;
        this.template.info.style.display = 'none';
        try {
            await this.services.auth.authenticate({
                username: this.template.form.username.value,
                password: this.template.form.password.value,
            });
            if (!this.services.router.restore()) {
                this.services.router.navigate(['comments']);
            }
        } catch (error) {
            this.template.info.style.display = '';
        } finally {
            this.template.submit.disabled = false;
        }
    }
}
