/* Copyright (c) 2021 Niels Sonnich Poulsen (http://nielssp.dk)
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

import { Api } from "./api";
import { Page, Router } from "./router";

export class Import implements Page {
    constructor(
        private template: {
            root: HTMLElement,
            form: HTMLFormElement,
            info: HTMLElement,
            submit: HTMLButtonElement,
        },
        private services: {
            api: Api,
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
        const files: FileList = this.template.form.file.files;
        if (!files.length) {
            return;
        }
        this.template.submit.disabled = true;
        this.template.info.style.display = 'none';
        try {
            const data = new FormData();
            for (let i = 0; i < files.length; i++) {
                data.append('files[]', files[i]);
            }
            await this.services.api.post('admin/import', data);
            this.template.info.style.display = '';
            this.template.info.className = 'info success';
            this.template.info.textContent = 'Comments imported';
        } catch (error) {
            this.template.info.style.display = '';
            this.template.info.className = 'info warning';
            this.template.info.textContent = 'Error';
        } finally {
            this.template.submit.disabled = false;
        }
    }
}
