/* Copyright (c) 2021 Niels Sonnich Poulsen (http://nielssp.dk)
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

import { getRelative } from "../util";
import { Api, ApiPage } from "./api";
import { Page, Router } from "./router";
import { appendComponent } from "./util";

export type CommentStatus = 'Pending' | 'Approved' | 'Rejected';

export interface Comment {
    id: number;
    thread_id: number;
    thread_name: string;
    parent_id?: number;
    status: CommentStatus;
    name: string;
    email: string;
    ip: string;
    website: string;
    markdown: string;
    html: string;
    created: string;
    created_timestamp: number;
    replies: number;
}

type Filter = {
    type: 'status',
    value: CommentStatus,
} | {
    type: 'parent_id',
    value: number,
} | {
    type: 'thread_id',
    value: number,
} | {
    type: 'id',
    value: number,
};

const dateFormat = new Intl.DateTimeFormat([], {
    year: 'numeric',
    month: 'long',
    day: 'numeric',
    hour: 'numeric',
    minute: 'numeric',
});

export class Comments implements Page {
    offset = 0;
    filter: Filter = {type: 'status', value: 'Pending'};
    asc: boolean = false;
    comments?: ApiPage<Comment>;

    constructor(
        private template: {
            root: HTMLElement,
            comments: HTMLElement,
            pageStart: HTMLElement,
            pageEnd: HTMLElement,
            total: HTMLElement,
            filterPending: HTMLButtonElement,
            filterApproved: HTMLButtonElement,
            filterRejected: HTMLButtonElement,
            refresh: HTMLButtonElement,
            sort: HTMLButtonElement,
            prev: HTMLButtonElement,
            next: HTMLButtonElement,
        },
        private services: {
            api: Api,
            router: Router,
        },
    ) {
        template.filterPending.onclick = () => this.applyFilter({type: 'status', value: 'Pending'});
        template.filterApproved.onclick = () => this.applyFilter({type: 'status', value: 'Approved'});
        template.filterRejected.onclick = () => this.applyFilter({type: 'status', value: 'Rejected'});
        template.refresh.onclick = () => this.fetchComments();
        template.sort.onclick = () => this.sort();
        template.next.onclick = () => this.next();
        template.prev.onclick = () => this.prev();
    }

    async applyFilter(filter: Filter) {
        this.template.filterPending.disabled = true;
        this.template.filterApproved.disabled = true;
        this.template.filterRejected.disabled = true;
        this.offset = 0;
        this.filter = filter;
        this.pushState();
        try {
            await this.fetchComments();
            this.updateButtons();
        } finally {
            this.template.filterPending.disabled = false;
            this.template.filterApproved.disabled = false;
            this.template.filterRejected.disabled = false;
        }
    }

    enter(args: {offset: string, asc: string, filterType: string, filterValue: string}): void {
        this.template.root.style.display = '';
        if ('offset' in args) {
            this.offset = parseInt(args.offset, 10);
        }
        if ('asc' in args) {
            this.asc = args.asc === 'true';
        }
        if (args.filterType && args.filterValue) {
            switch (args.filterType) {
                case 'status':
                    this.filter = {type: args.filterType, value: args.filterValue as CommentStatus};
                    break;
                case 'parent_id':
                case 'thread_id':
                case 'id':
                    this.filter = {type: args.filterType, value: parseInt(args.filterValue, 10)};
                    break;
            }
        }
        this.updateButtons();
        this.fetchComments();
    }

    get args() {
        return {
            offset: '' + this.offset,
            asc: '' + this.asc,
            filterType: this.filter.type,
            filterValue: '' + this.filter.value,
        };
    }

    pushState() {
        this.services.router.pushState(['comments'], this.args);
    }

    leave(): void {
        this.template.root.style.display = 'none';
    }

    updateButtons(): void {
        this.template.filterPending.setAttribute('aria-pressed', `${this.filter.value === 'Pending'}`);
        this.template.filterApproved.setAttribute('aria-pressed', `${this.filter.value === 'Approved'}`);
        this.template.filterRejected.setAttribute('aria-pressed', `${this.filter.value === 'Rejected'}`);
        this.template.sort.textContent = this.asc ? 'Date \u2191' : 'Date \u2193';
    }

    async fetchComments() {
        this.template.comments.classList.add('loading');
        this.template.refresh.disabled = true;
        this.template.prev.disabled = true;
        this.template.next.disabled = true;
        try {
            if (this.filter.type === 'id') {
                this.comments = {
                    content: [await this.services.api.get<Comment>(`admin/comments/${this.filter.value}`)],
                    remaining: 0,
                    limit: 1,
                };
                this.offset = 0;
            } else {
                this.comments = await this.services.api.get<ApiPage<Comment>>(
                    `admin/comments?offset=${this.offset}&${this.filter.type}=${this.filter.value}&asc=${this.asc}`);
            }
            this.template.pageStart.textContent = `${this.offset + 1}`;
            this.template.pageEnd.textContent = `${this.offset + this.comments.content.length}`;
            this.template.total.textContent = `${this.offset + this.comments.content.length + this.comments.remaining}`;
            this.template.comments.innerHTML = '';
            this.comments.content.forEach(comment => {
                appendComponent(this.template.comments, CommentRow, commentTemplate,
                    {comment, api: this.services.api, router: this.services.router, comments: this})
            });
            if (this.offset > 0) {
                this.template.prev.disabled = false;
            }
            if (this.comments.remaining > 0) {
                this.template.next.disabled = false;
            }
        } catch (error) {
            alert('Server error');
        } finally {
            this.template.comments.classList.remove('loading');
            this.template.refresh.disabled = false;
        }
    }

    async sort() {
        this.asc = !this.asc;
        this.pushState();
        this.template.sort.disabled = true;
        this.template.sort.textContent = this.asc ? 'Date \u2191' : 'Date \u2193';
        try {
            await this.fetchComments();
        } finally {
            this.template.sort.disabled = false;
        }
    }

    async next() {
        if (!this.comments || !this.comments.remaining) {
            return;
        }
        this.offset += this.comments.limit;
        this.pushState();
        await this.fetchComments();
        const clientRect = this.template.root.getBoundingClientRect();
        if (clientRect.top < 0) {
            window.scrollBy(0, clientRect.top);
        }
    }

    async prev() {
        if (!this.comments || !this.offset) {
            return;
        }
        this.offset = Math.max(0, this.offset - this.comments.limit);
        this.pushState();
        await this.fetchComments();
    }
}

const commentTemplate = `<div class="box-row flex-column stretch">
    <div><a data-bind="thread"></a></div>
    <div class="comment-row">
        <div data-bind="comment" class="comment grow">
            <div class="comment-header">
                <span class="author" data-bind="author"></span>
                <span class="email" data-bind="email"></span>
                <span class="ip" data-bind="ip"></span>
                <time data-bind="created"></time>
            </div>
            <div class="comment-body" data-bind="content"></div>
            <div><a href="#" data-bind="more">[more]</a></div>
            <div class="comment-actions">
                <a href="" data-bind="replies"></a>
                <a href="" data-bind="parent">Parent</a>
            </div>
        </div>
        <div data-bind="actions" class="button-group" style="margin-left: auto;">
            <button data-bind="edit">Edit</button>
            <button data-bind="approve">Approve</button>
            <button data-bind="reject">Reject</button>
        </div>
    </div>
    <div data-bind="editForm">
    </div>
</div>`;

class CommentRow {
    private expanded = false;

    constructor(
        private template: {
            root: HTMLElement,
            thread: HTMLLinkElement,
            comment: HTMLElement,
            author: HTMLElement,
            email: HTMLElement,
            ip: HTMLElement,
            created: HTMLTimeElement,
            content: HTMLElement,
            more: HTMLLinkElement,
            replies: HTMLLinkElement,
            parent: HTMLLinkElement,
            actions: HTMLElement,
            edit: HTMLButtonElement,
            approve: HTMLButtonElement,
            reject: HTMLButtonElement,
            editForm: HTMLElement,
        },
        private data: {
            comment: Comment,
            api: Api,
            router: Router,
            comments: Comments,
        }
    ) {
        const comment = data.comment;
        template.thread.textContent = comment.thread_name;
        data.router.link(template.thread, ['threads'], {
            'filterType': 'id',
            'filterValue': '' + data.comment.thread_id,
        });
        template.comment.classList.add(comment.status.toLowerCase());
        this.update(comment);
        const created = new Date(comment.created_timestamp * 1000);
        template.created.textContent = getRelative(created);
        template.created.title = dateFormat.format(created);
        template.created.dateTime = created.toISOString();
        template.more.onclick = e => this.more(e);
        template.replies.textContent = (n => n === 1 ? `${n} reply` : `${n} replies`)(comment.replies);
        data.router.link(template.replies, ['comments'], {
            'filterType': 'parent_id',
            'filterValue': '' + data.comment.id,
        });
        template.parent.style.display = comment.parent_id ? '' : 'none';
        data.router.link(template.parent, ['comments'], {
            'filterType': 'id',
            'filterValue': '' + data.comment.parent_id,
        });
        template.approve.disabled = comment.status === 'Approved';
        template.reject.disabled = comment.status === 'Rejected';
        template.edit.onclick = () => this.edit();
        template.approve.onclick = () => this.approve();
        template.reject.onclick = () => this.reject();
    }

    more(e: MouseEvent) {
        e.preventDefault();
        if (this.expanded) {
            this.template.content.style.maxHeight = '';
            this.template.more.textContent = '[more]';
            this.expanded = false;
        } else {
            this.template.content.style.maxHeight = `${this.template.content.scrollHeight}px`;
            this.template.more.textContent = '[less]';
            this.expanded = true;
        }
    }

    edit() {
        appendComponent(this.template.editForm, CommentForm, commentFormTemplate, {
            api: this.data.api,
            comment: this.data.comment,
            onCancel: () => this.closeEdit(),
            onDelete: () => this.delete(),
            onSave: comment => {
                this.update(comment);
                this.closeEdit();
            },
        });
        this.template.comment.style.display = 'none';
        this.template.actions.style.display = 'none';
        this.template.root.classList.add('editting');
    }

    update(comment: Comment) {
        this.template.content.style.maxHeight = '';
        this.template.more.textContent = '[more]';
        this.expanded = false;
        this.data.comment = comment;
        if (comment.website) {
            const link = document.createElement('a');
            link.textContent = comment.name || 'Anonymous';
            link.href = comment.website;
            this.template.author.innerHTML = '';
            this.template.author.appendChild(link);
        } else {
            this.template.author.textContent = comment.name || 'Anonynous';
        }
        if (comment.name) {
            this.template.author.style.fontStyle = '';
        } else {
            this.template.author.style.fontStyle = 'italic';
        }
        if (comment.email) {
            this.template.email.style.display = '';
            this.template.email.textContent = comment.email;
        } else {
            this.template.email.style.display = 'none';
        }
        if (comment.ip) {
            this.template.ip.style.display = '';
            this.template.ip.textContent = comment.ip;
        } else {
            this.template.ip.style.display = 'none';
        }
        this.template.content.textContent = comment.markdown;
        this.template.content.innerHTML = comment.html;
        if (this.template.content.getBoundingClientRect().height < this.template.content.scrollHeight) {
            this.template.more.style.display = '';
        } else {
            this.template.more.style.display = 'none';
        }
    }

    async delete() {
        if (confirm(`Are you sure you want to delete this comment?`)) {
            try {
                await this.data.api.delete(`admin/comments/${this.data.comment.id}`);
                this.template.root.parentNode?.removeChild(this.template.root);
            } catch (error) {
                alert('Server error');
            }
        }
    }

    closeEdit() {
        this.template.root.classList.remove('editting');
        this.template.editForm.innerHTML = '';
        this.template.comment.style.display = '';
        this.template.actions.style.display = '';
    }

    async approve() {
        this.template.comment.classList.remove(this.data.comment.status.toLowerCase());
        this.data.comment.status = 'Approved';
        try {
            await this.data.api.put(`admin/comments/${this.data.comment.id}`, this.data.comment);
            this.template.comment.classList.add(this.data.comment.status.toLowerCase());
            this.template.approve.disabled = true;
            this.template.reject.disabled = false;
        } catch (error) {
            alert('Server error');
        }
    }

    async reject() {
        this.template.comment.classList.remove(this.data.comment.status.toLowerCase());
        this.data.comment.status = 'Rejected';
        try {
            await this.data.api.put(`admin/comments/${this.data.comment.id}`, this.data.comment);
            this.template.comment.classList.add(this.data.comment.status.toLowerCase());
            this.template.approve.disabled = false;
            this.template.reject.disabled = true;
        } catch (error) {
            alert('Server error');
        }
    }
}

const commentFormTemplate = `<form class="margin-top">
    <div class="field">
        <label>
            Name
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
            Content
            <textarea data-bind="content" rows=6></textarea>
        </label>
    </div>
    <div class="flex-row space-between">
        <button data-bind="cancel" type="button">Cancel</button>
        <div>
            <button data-bind="delete" type="button">Delete</button>
            <button data-bind="submit" type="submit">Save</button>
        </div>
    </div>
</form>`;

class CommentForm {
    constructor(
        private template: {
            root: HTMLFormElement,
            name: HTMLInputElement,
            email: HTMLInputElement,
            website: HTMLInputElement,
            content: HTMLTextAreaElement,
            cancel: HTMLButtonElement,
            delete: HTMLButtonElement,
            submit: HTMLButtonElement,
        },
        private data: {
            api: Api,
            comment: Comment,
            onCancel: () => void,
            onDelete: () => void,
            onSave: (comment: Comment) => void,
        }
    ) {
        template.name.value = data.comment.name;
        template.email.value = data.comment.email;
        template.website.value = data.comment.website;
        template.content.value = data.comment.markdown;
        template.cancel.onclick = () => data.onCancel();
        template.delete.onclick = () => data.onDelete();
        template.root.onsubmit = e => this.submit(e);
    }

    async submit(e: Event) {
        e.preventDefault();
        this.template.submit.disabled = true;
        try {
            const comment = await this.data.api.put<Comment>(`admin/comments/${this.data.comment.id}`, {
                name: this.template.name.value,
                email: this.template.email.value,
                website: this.template.website.value,
                markdown: this.template.content.value,
                status: this.data.comment.status,
            });
            this.data.onSave(comment);
        } catch (error) {
            alert('Server error');
        } finally {
            this.template.submit.disabled = false;
        }
    }
}
