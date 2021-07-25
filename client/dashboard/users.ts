/* Copyright (c) 2021 Niels Sonnich Poulsen (http://nielssp.dk)
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

import { Api, ApiPage } from "./api";
import { Page, Router } from "./router";
import { appendComponent, prependComponent } from "./util";

export interface User {
    id: number;
    username: string;
    name: string;
    email: string;
    website: string;
    trusted: boolean;
    admin: boolean;
}

type Filter = {
    type: 'id',
    value: number,
};

export class Users implements Page {
    offset = 0;
    filter?: Filter;
    users?: ApiPage<User>;

    constructor(
        private template: {
            root: HTMLElement,
            users: HTMLElement,
            pageStart: HTMLElement,
            pageEnd: HTMLElement,
            total: HTMLElement,
            filterAll: HTMLButtonElement,
            create: HTMLButtonElement,
            refresh: HTMLButtonElement,
            prev: HTMLButtonElement,
            next: HTMLButtonElement,
        },
        private services: {
            api: Api,
            router: Router,
        },
    ) {
        template.filterAll.onclick = () => this.applyFilter();
        template.create.onclick = () => this.create();
        template.refresh.onclick = () => this.fetchUsers();
        template.next.onclick = () => this.next();
        template.prev.onclick = () => this.prev();
    }

    enter(args: {offset: string, filterType: string, filterValue: string}): void {
        this.template.root.style.display = '';
        if ('offset' in args) {
            this.offset = parseInt(args.offset, 10);
        }
        if (args.filterType && args.filterValue) {
            switch (args.filterType) {
                case 'id':
                    this.filter = {type: args.filterType, value: parseInt(args.filterValue, 10)};
                    break;
            }
        }
        this.updateButtons();
        this.fetchUsers();
    }

    get args() {
        return {
            offset: '' + this.offset,
            filterType: this.filter ? this.filter.type : '',
            filterValue: this.filter ? '' + this.filter.value : '',
        };
    }

    pushState() {
        this.services.router.pushState(['users'], this.args);
    }

    leave(): void {
        this.template.root.style.display = 'none';
    }

    async applyFilter(filter?: Filter) {
        this.offset = 0;
        this.filter = filter;
        this.pushState();
        await this.fetchUsers();
        this.updateButtons();
    }

    create() {
        prependComponent(this.template.users, UserRow, userTemplate, {
            user: {
                id: 0,
                username: '',
                name: '',
                email: '',
                website: '',
                trusted: false,
                admin: false,
            },
            api: this.services.api,
            router: this.services.router,
            users: this,
            isNew: true,
        });
    }

    updateButtons(): void {
        this.template.filterAll.setAttribute('aria-pressed', `${!this.filter}`);
    }

    async fetchUsers() {
        this.template.users.classList.add('loading');
        this.template.refresh.disabled = true;
        this.template.prev.disabled = true;
        this.template.next.disabled = true;
        try {
            if (this.filter && this.filter.type === 'id') {
                this.users = {
                    content: [await this.services.api.get<User>(`admin/users/${this.filter.value}`)],
                    remaining: 0,
                    limit: 1,
                };
                this.offset = 0;
            } else {
                this.users = await this.services.api.get<ApiPage<User>>(
                    `admin/users?offset=${this.offset}`);
            }
            this.template.pageStart.textContent = `${this.offset + 1}`;
            this.template.pageEnd.textContent = `${this.offset + this.users.content.length}`;
            this.template.total.textContent = `${this.offset + this.users.content.length + this.users.remaining}`;
            this.template.users.innerHTML = '';
            this.users.content.forEach(user => {
                appendComponent(this.template.users, UserRow, userTemplate,
                    {user, api: this.services.api, router: this.services.router, users: this})
            });
            if (this.offset > 0) {
                this.template.prev.disabled = false;
            }
            if (this.users.remaining > 0) {
                this.template.next.disabled = false;
            }
        } catch (error) {
            alert('Server error');
        } finally {
            this.template.users.classList.remove('loading');
            this.template.refresh.disabled = false;
        }
    }

    async next() {
        if (!this.users || !this.users.remaining) {
            return;
        }
        this.offset += this.users.limit;
        this.pushState();
        await this.fetchUsers();
        const clientRect = this.template.root.getBoundingClientRect();
        if (clientRect.top < 0) {
            window.scrollBy(0, clientRect.top);
        }
    }

    async prev() {
        if (!this.users || !this.offset) {
            return;
        }
        this.offset = Math.max(0, this.offset - this.users.limit);
        this.pushState();
        await this.fetchUsers();
    }
}

const userTemplate = `<div class="box-row flex-column stretch">
    <div class="user-row" data-bind="user">
        <div class="user-info">
            <div data-bind="username"></div>
            <div data-bind="name"></div>
            <div data-bind="website"></div>
            <div data-bind="email"></div>
        </div>
        <div data-bind="actions" class="button-group" style="margin-left: auto;">
            <button data-bind="edit">Edit</button>
        </div>
    </div>
    <div data-bind="editForm">
    </div>
</div>`;

class UserRow {
    private expanded = false;

    constructor(
        private template: {
            root: HTMLElement,
            user: HTMLElement,
            username: HTMLElement,
            name: HTMLElement,
            website: HTMLElement,
            email: HTMLElement,
            actions: HTMLElement,
            edit: HTMLButtonElement,
            editForm: HTMLElement,
        },
        private data: {
            user: User,
            api: Api,
            router: Router,
            users: Users,
            isNew?: boolean,
        }
    ) {
        const user = data.user;
        this.update(user);
        template.edit.onclick = () => this.edit();
        if (data.isNew) {
            this.edit();
        }
    }

    edit() {
        appendComponent(this.template.editForm, UserForm, userFormTemplate, {
            api: this.data.api,
            user: this.data.user,
            onCancel: () => this.closeEdit(),
            onSave: user => {
                this.update(user);
                this.data.isNew = false;
                this.closeEdit();
            },
            isNew: this.data.isNew || false,
        });
        this.template.user.style.display = 'none';
        this.template.actions.style.display = 'none';
        this.template.root.classList.add('editting');
    }

    update(user: User) {
        this.data.user = user;
        this.template.username.textContent = user.username;
        this.template.name.textContent = user.name;
        this.template.name.style.display = user.name ? '' : 'none';
        this.template.email.textContent = user.email;
        this.template.email.style.display = user.email ? '' : 'none';
        this.template.website.textContent = user.website;
        this.template.website.style.display = user.website ? '' : 'none';
    }

    closeEdit() {
        this.template.root.classList.remove('editting');
        this.template.editForm.innerHTML = '';
        this.template.user.style.display = '';
        this.template.actions.style.display = '';
        if (this.data.isNew) {
            this.template.root.parentNode?.removeChild(this.template.root);
        }
    }
}

const userFormTemplate = `<form class="margin-top">
    <div class="info warning" data-bind="info"></div>
    <div class="field">
        <label>
            Username
            <input type="text" data-bind="username" required/>
        </label>
    </div>
    <div data-bind="passwordFields">
        <div class="field">
            <label>
                Password
                <input type="password" data-bind="password"/>
            </label>
        </div>
        <div class="field">
            <label>
                Confirm Password
                <input type="password" data-bind="confirmPassword"/>
            </label>
        </div>
    </div>
    <div class="field">
        <label>
            Display Name
            <input type="text" data-bind="name"/>
        </label>
    </div>
    <div class="field">
        <label>
            Email
            <input type="text" data-bind="email"/>
        </label>
    </div>
    <div class="field">
        <label>
            Website
            <input type="text" data-bind="website"/>
        </label>
    </div>
    <div class="field">
        <label>
            <input type="checkbox" data-bind="trusted"/>
            Trusted
        </label>
    </div>
    <div class="field">
        <label>
            <input type="checkbox" data-bind="admin"/>
            Admin
        </label>
    </div>
    <div class="flex-row space-between">
        <button data-bind="cancel" type="button">Cancel</button>
        <button data-bind="submit" type="submit">Save</button>
    </div>
</form>`;

class UserForm {
    constructor(
        private template: {
            root: HTMLFormElement,
            info: HTMLElement,
            username: HTMLInputElement,
            passwordFields: HTMLElement,
            password: HTMLInputElement,
            confirmPassword: HTMLInputElement,
            name: HTMLInputElement,
            email: HTMLInputElement,
            website: HTMLInputElement,
            trusted: HTMLInputElement,
            admin: HTMLInputElement,
            cancel: HTMLButtonElement,
            submit: HTMLButtonElement,
        },
        private data: {
            api: Api,
            user: User,
            onCancel: () => void,
            onSave: (user: User) => void,
            isNew: boolean,
        }
    ) {
        template.info.style.display = 'none';
        template.username.value = data.user.username;
        if (data.isNew) {
            template.password.required = true;
            template.confirmPassword.required = true;
        } else {
            template.passwordFields.style.display = 'none';
        }
        template.name.value = data.user.name;
        template.email.value = data.user.email;
        template.website.value = data.user.website;
        template.trusted.checked = data.user.trusted;
        template.admin.checked = data.user.admin;
        template.cancel.onclick = () => data.onCancel();
        template.root.onsubmit = e => this.submit(e);
    }

    async submit(e: Event) {
        e.preventDefault();
        this.template.submit.disabled = true;
        try {
            if (this.data.isNew) {
                if (this.template.password.value !== this.template.confirmPassword.value) {
                    this.template.info.style.display = '';
                    this.template.info.className = 'info warning';
                    this.template.info.textContent = 'The passwords do not match';
                    return;
                }
                this.data.onSave(await this.data.api.post<User>(`admin/users`, {
                    username: this.template.username.value,
                    password: this.template.password.value,
                    confirm_password: this.template.confirmPassword.value,
                    name: this.template.name.value,
                    email: this.template.email.value,
                    website: this.template.website.value,
                    trusted: this.template.trusted.checked,
                    admin: this.template.admin.checked,
                }));
            } else {
                this.data.onSave(await this.data.api.put<User>(`admin/users/${this.data.user.id}`, {
                    username: this.template.username.value,
                    name: this.template.name.value,
                    email: this.template.email.value,
                    website: this.template.website.value,
                    trusted: this.template.trusted.checked,
                    admin: this.template.admin.checked,
                }));
            }
        } catch (error) {
            alert('Server error');
        } finally {
            this.template.submit.disabled = false;
        }
    }
}
