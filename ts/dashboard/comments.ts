import { language } from "../languages/default";
import { getRelative } from "../util";
import { Api, ApiPage } from "./api";
import { Page } from "./router";
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
    type: 'id',
    value: number,
};

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
        try {
            await this.fetchComments();
            this.template.filterPending.setAttribute('aria-pressed', `${this.filter.value === 'Pending'}`);
            this.template.filterApproved.setAttribute('aria-pressed', `${this.filter.value === 'Approved'}`);
            this.template.filterRejected.setAttribute('aria-pressed', `${this.filter.value === 'Rejected'}`);
        } finally {
            this.template.filterPending.disabled = false;
            this.template.filterApproved.disabled = false;
            this.template.filterRejected.disabled = false;
        }
    }

    enter(): void {
        this.template.root.style.display = '';
        this.fetchComments();
    }

    leave(): void {
        this.template.root.style.display = 'none';
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
                    {comment, api: this.services.api, comments: this})
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
                <time data-bind="created"></time>
            </div>
            <div class="comment-body" data-bind="content"></div>
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
    constructor(
        private template: {
            root: HTMLElement,
            thread: HTMLLinkElement,
            comment: HTMLElement,
            author: HTMLElement,
            email: HTMLElement,
            created: HTMLTimeElement,
            content: HTMLElement,
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
            comments: Comments,
        }
    ) {
        const comment = data.comment;
        template.thread.textContent = comment.thread_name;
        template.comment.classList.add(comment.status.toLowerCase());
        if (comment.website) {
            const link = document.createElement('a');
            link.textContent = comment.name;
            link.href = comment.website;
            template.author.appendChild(link);
        } else {
            template.author.textContent = comment.name;
        }
        if (comment.email) {
            template.email.textContent = comment.email;
        } else {
            template.email.style.display = 'none';
        }
        const created = new Date(comment.created_timestamp * 1000);
        template.created.textContent = getRelative(created);
        template.created.title = language.date(created);
        template.created.dateTime = created.toISOString();
        template.content.textContent = comment.markdown;
        template.replies.textContent = (n => n === 1 ? `${n} reply` : `${n} replies`)(comment.replies);
        template.replies.onclick = e => this.replies(e);
        template.parent.style.display = comment.parent_id ? '' : 'none';
        template.parent.onclick = e => this.parent(e);
        template.approve.disabled = comment.status === 'Approved';
        template.reject.disabled = comment.status === 'Rejected';
        template.edit.onclick = () => this.edit();
        template.approve.onclick = () => this.approve();
        template.reject.onclick = () => this.reject();
    }

    replies(e: MouseEvent) {
        e.preventDefault();
        this.data.comments.applyFilter({type: 'parent_id', value: this.data.comment.id});
    }

    parent(e: MouseEvent) {
        e.preventDefault();
        this.data.comments.applyFilter({type: 'id', value: this.data.comment.parent_id!});
    }

    edit() {
        appendComponent(this.template.editForm, CommentForm, commentFormTemplate, {
            api: this.data.api,
            comment: this.data.comment,
            onCancel: () => this.closeEdit(),
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
        this.data.comment = comment;
        if (comment.website) {
            const link = document.createElement('a');
            link.textContent = comment.name;
            link.href = comment.website;
            this.template.author.innerHTML = '';
            this.template.author.appendChild(link);
        } else {
            this.template.author.textContent = comment.name;
        }
        if (comment.email) {
            this.template.email.style.display = '';
            this.template.email.textContent = comment.email;
        } else {
            this.template.email.style.display = 'none';
        }
        this.template.content.textContent = comment.markdown;
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

const commentFormTemplate = `<form>
    <div class="field">
        <label>Name</label>
        <input type="text" data-bind="name"/>
    </div>
    <div class="field">
        <label>Email</label>
        <input type="text" data-bind="email"/>
    </div>
    <div class="field">
        <label>Website</label>
        <input type="text" data-bind="website"/>
    </div>
    <div class="field">
        <label>Content</label>
        <textarea data-bind="content" rows=6></textarea>
    </div>
    <div class="flex-row space-between">
        <button data-bind="cancel" type="button">Cancel</button>
        <button data-bind="submit" type="submit">Save</button>
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
            submit: HTMLButtonElement,
        },
        private data: {
            api: Api,
            comment: Comment,
            onCancel: () => void,
            onSave: (comment: Comment) => void,
        }
    ) {
        template.name.value = data.comment.name;
        template.email.value = data.comment.email;
        template.website.value = data.comment.website;
        template.content.value = data.comment.markdown;
        template.cancel.onclick = () => data.onCancel();
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
