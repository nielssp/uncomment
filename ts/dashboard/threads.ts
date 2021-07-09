/* Copyright (c) 2021 Niels Sonnich Poulsen (http://nielssp.dk)
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

import { Api, ApiPage } from "./api";
import { Page, Router } from "./router";
import { appendComponent } from "./util";

export interface Thread {
    id: number;
    name: string;
    title: string;
    comments: number;
}

type Filter = {
    type: 'id',
    value: number,
};

export class Threads implements Page {
    offset = 0;
    filter?: Filter;
    threads?: ApiPage<Thread>;

    constructor(
        private template: {
            root: HTMLElement,
            threads: HTMLElement,
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
        this.fetchThreads();
    }

    get args() {
        return {
            offset: '' + this.offset,
            filterType: this.filter ? this.filter.type : '',
            filterValue: this.filter ? '' + this.filter.value : '',
        };
    }

    pushState() {
        this.services.router.pushState(['threads'], this.args);
    }

    leave(): void {
        this.template.root.style.display = 'none';
    }

    async applyFilter(filter?: Filter) {
        this.offset = 0;
        this.filter = filter;
        this.pushState();
        await this.fetchThreads();
        this.updateButtons();
    }

    create() {
        appendComponent(this.template.threads, ThreadRow, threadTemplate, {
            thread: {
                id: 0,
                name: '',
                title: '',
                comments: 0,
            },
            api: this.services.api,
            router: this.services.router,
            threads: this,
            isNew: true,
        });
    }

    updateButtons(): void {
        this.template.filterAll.setAttribute('aria-pressed', `${!this.filter}`);
    }

    async fetchThreads() {
        this.template.threads.classList.add('loading');
        this.template.refresh.disabled = true;
        this.template.prev.disabled = true;
        this.template.next.disabled = true;
        try {
            if (this.filter && this.filter.type === 'id') {
                this.threads = {
                    content: [await this.services.api.get<Thread>(`admin/threads/${this.filter.value}`)],
                    remaining: 0,
                    limit: 1,
                };
                this.offset = 0;
            } else {
                this.threads = await this.services.api.get<ApiPage<Thread>>(
                    `admin/threads?offset=${this.offset}`);
            }
            this.template.pageStart.textContent = `${this.offset + 1}`;
            this.template.pageEnd.textContent = `${this.offset + this.threads.content.length}`;
            this.template.total.textContent = `${this.offset + this.threads.content.length + this.threads.remaining}`;
            this.template.threads.innerHTML = '';
            this.threads.content.forEach(thread => {
                appendComponent(this.template.threads, ThreadRow, threadTemplate,
                    {thread, api: this.services.api, router: this.services.router, threads: this})
            });
            if (this.offset > 0) {
                this.template.prev.disabled = false;
            }
            if (this.threads.remaining > 0) {
                this.template.next.disabled = false;
            }
        } catch (error) {
            alert('Server error');
        } finally {
            this.template.threads.classList.remove('loading');
            this.template.refresh.disabled = false;
        }
    }

    async next() {
        if (!this.threads || !this.threads.remaining) {
            return;
        }
        this.offset += this.threads.limit;
        this.pushState();
        await this.fetchThreads();
        const clientRect = this.template.root.getBoundingClientRect();
        if (clientRect.top < 0) {
            window.scrollBy(0, clientRect.top);
        }
    }

    async prev() {
        if (!this.threads || !this.offset) {
            return;
        }
        this.offset = Math.max(0, this.offset - this.threads.limit);
        this.pushState();
        await this.fetchThreads();
    }
}

const threadTemplate = `<div class="box-row flex-column stretch">
    <div class="thread-row" data-bind="thread">
        <div class="thread-info">
            <div data-bind="name"></div>
            <div data-bind="title"></div>
            <a href="" data-bind="comments"></a>
        </div>
        <div data-bind="actions" class="button-group" style="margin-left: auto;">
            <button data-bind="edit">Edit</button>
        </div>
    </div>
    <div data-bind="editForm">
    </div>
</div>`;

class ThreadRow {
    private expanded = false;

    constructor(
        private template: {
            root: HTMLElement,
            thread: HTMLElement,
            name: HTMLElement,
            title: HTMLElement,
            comments: HTMLLinkElement,
            actions: HTMLElement,
            edit: HTMLButtonElement,
            editForm: HTMLElement,
        },
        private data: {
            thread: Thread,
            api: Api,
            router: Router,
            threads: Threads,
            isNew?: boolean,
        }
    ) {
        const thread = data.thread;
        this.update(thread);
        template.comments.textContent = (n => n === 1 ? `${n} comments` : `${n} comments`)(thread.comments);
        template.comments.onclick = e => this.comments(e);
        template.edit.onclick = () => this.edit();
        if (data.isNew) {
            this.edit();
        }
    }

    comments(e: MouseEvent) {
        e.preventDefault();
        this.data.router.navigate(['comments'], {
            filterType: 'thread_id',
            filterValue: '' + this.data.thread.id,
        });
    }

    edit() {
        appendComponent(this.template.editForm, ThreadForm, threadFormTemplate, {
            api: this.data.api,
            thread: this.data.thread,
            onCancel: () => this.closeEdit(),
            onSave: thread => {
                this.update(thread);
                this.data.isNew = false;
                this.closeEdit();
            },
            isNew: this.data.isNew || false,
        });
        this.template.thread.style.display = 'none';
        this.template.actions.style.display = 'none';
        this.template.root.classList.add('editting');
    }

    update(thread: Thread) {
        this.template.name.textContent = thread.name;
        this.template.title.textContent = thread.title;
        this.template.title.style.display = thread.title ? '' : 'none';
    }

    closeEdit() {
        this.template.root.classList.remove('editting');
        this.template.editForm.innerHTML = '';
        this.template.thread.style.display = '';
        this.template.actions.style.display = '';
        if (this.data.isNew) {
            this.template.root.parentNode?.removeChild(this.template.root);
        }
    }
}

const threadFormTemplate = `<form class="margin-top">
    <div class="field">
        <label>
            Name
            <input type="text" data-bind="name"/>
        </label>
    </div>
    <div class="field">
        <label>
            Title
            <input type="text" data-bind="title"/>
        </label>
    </div>
    <div class="flex-row space-between">
        <button data-bind="cancel" type="button">Cancel</button>
        <button data-bind="submit" type="submit">Save</button>
    </div>
</form>`;

class ThreadForm {
    constructor(
        private template: {
            root: HTMLFormElement,
            name: HTMLInputElement,
            title: HTMLInputElement,
            cancel: HTMLButtonElement,
            submit: HTMLButtonElement,
        },
        private data: {
            api: Api,
            thread: Thread,
            onCancel: () => void,
            onSave: (thread: Thread) => void,
            isNew: boolean,
        }
    ) {
        template.name.value = data.thread.name;
        template.title.value = data.thread.title;
        if (!this.data.isNew) {
            template.name.disabled = true;
        }
        template.cancel.onclick = () => data.onCancel();
        template.root.onsubmit = e => this.submit(e);
    }

    async submit(e: Event) {
        e.preventDefault();
        this.template.submit.disabled = true;
        try {
            if (this.data.isNew) {
                this.data.onSave(await this.data.api.post<Thread>(`admin/threads`, {
                    name: this.template.name.value,
                    title: this.template.title.value,
                }));
            } else {
                this.data.onSave(await this.data.api.put<Thread>(`admin/threads/${this.data.thread.id}`, {
                    title: this.template.title.value,
                }));
            }
        } catch (error) {
            alert('Server error');
        } finally {
            this.template.submit.disabled = false;
        }
    }
}
