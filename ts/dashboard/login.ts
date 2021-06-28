import { Api } from "./api";
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
            api: Api,
            router: Router,
        },
    ) {
        template.info.style.display = 'none';
        template.form.addEventListener('submit', e => this.submit(e));
    }

    enter(): void {
        this.template.root.style.display = '';
    }

    leave(): void {
        this.template.root.style.display = 'none';
    }

    async submit(e: Event) {
        e.preventDefault();
        this.template.submit.disabled = true;
        try {
            await this.services.api.post('auth', {
                username: this.template.form.username.value,
                password: this.template.form.password.value,
            });
            this.services.router.navigate(['comments']);
        } finally {
            this.template.submit.disabled = false;
        }
    }
}
