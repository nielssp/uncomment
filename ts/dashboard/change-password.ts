import { Api } from "./api";
import { Auth } from "./auth";
import { Page, Router } from "./router";

export class ChangePassword implements Page {
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

    async submit(e: Event) {
        e.preventDefault();
        if (this.template.form.newPassword.value !== this.template.form.confirmPassword.value) {
            this.template.info.style.display = '';
            this.template.info.className = 'info warning';
            this.template.info.textContent = 'The passwords do not match';
            return;
        }
        this.template.submit.disabled = true;
        this.template.info.style.display = 'none';
        try {
            await this.services.api.put('password', {
                existing_password: this.template.form.existingPassword.value,
                new_password: this.template.form.newPassword.value,
            });
            this.template.info.style.display = '';
            this.template.info.className = 'info success';
            this.template.info.textContent = 'Password changed';
            this.template.form.existingPassword.value = '';
            this.template.form.newPassword.value = '';
            this.template.form.confirmPassword.value = '';
        } catch (error) {
            this.template.info.style.display = '';
            this.template.info.className = 'info warning';
            this.template.info.textContent = 'Incorrect password';
        } finally {
            this.template.submit.disabled = false;
        }
    }
}
