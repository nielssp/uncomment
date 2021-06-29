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

export class Comments implements Page {
    offset = 0;
    filter: CommentStatus = 'Pending';
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
        template.filterPending.onclick = () => this.applyFilter('Pending');
        template.filterApproved.onclick = () => this.applyFilter('Approved');
        template.filterRejected.onclick = () => this.applyFilter('Rejected');
        template.refresh.onclick = () => this.fetchComments();
        template.sort.onclick = () => this.sort();
        template.next.onclick = () => this.next();
        template.prev.onclick = () => this.prev();
    }

    async applyFilter(filter: CommentStatus) {
        this.template.filterPending.disabled = true;
        this.template.filterApproved.disabled = true;
        this.template.filterRejected.disabled = true;
        this.filter = filter;
        try {
            await this.fetchComments();
            this.template.filterPending.setAttribute('aria-pressed', `${this.filter === 'Pending'}`);
            this.template.filterApproved.setAttribute('aria-pressed', `${this.filter === 'Approved'}`);
            this.template.filterRejected.setAttribute('aria-pressed', `${this.filter === 'Rejected'}`);
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
            this.comments = await this.services.api.get<ApiPage<Comment>>(
                `admin/comments?offset=${this.offset}&status=${this.filter}&asc=${this.asc}`);
            this.template.pageStart.textContent = `${this.offset + 1}`;
            this.template.pageEnd.textContent = `${this.offset + this.comments.content.length}`;
            this.template.total.textContent = `${this.offset + this.comments.content.length + this.comments.remaining}`;
            this.template.comments.innerHTML = '';
            this.comments.content.forEach(comment => {
                appendComponent(this.template.comments, CommentRow, commentTemplate, {comment, api: this.services.api})
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

const commentTemplate = `<div class="box-row">
<div>
    <div><a data-bind="thread"></a></div>
    <div data-bind="comment" class="comment">
        <div class="comment-header">
            <span class="author" data-bind="author"></span>
            <span class="email" data-bind="email"></span>
            <time data-bind="created"></time>
        </div>
        <div class="comment-body" data-bind="content"></div>
        <div class="comment-actions">
            <a data-bind="reply">Reply</a>
            <a data-bind="replies"></a>
            <a data-bind="parent">Parent</a>
        </div>
    </div>
    <div class="button-group">
        <button data-bind="edit">Edit</button>
        <button data-bind="approve">Approve</button>
        <button data-bind="reject">Reject</button>
    </div>
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
            reply: HTMLLinkElement,
            replies: HTMLLinkElement,
            parent: HTMLLinkElement,
            approve: HTMLButtonElement,
            reject: HTMLButtonElement,
        },
        private data: {
            comment: Comment,
            api: Api,
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
        template.parent.style.display = comment.parent_id ? '' : 'none';
        template.approve.disabled = comment.status === 'Approved';
        template.reject.disabled = comment.status === 'Rejected';
        template.approve.onclick = () => this.approve();
        template.reject.onclick = () => this.reject();
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
