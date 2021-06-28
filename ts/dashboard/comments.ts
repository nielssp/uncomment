import { Api, ApiPage } from "./api";
import { Page } from "./router";

export type CommentStatus = 'Pending' | 'Approved' | 'Rejected';

export interface Comment {
    id: number,
    thread_id: number,
    thread_name: String,
    parent_id?: number,
    status: CommentStatus,
    name: string,
    email: string,
    website: string,
    markdown: string,
    html: string,
    created: string,
    created_timestamp: number,
}

export class Comments implements Page {
    constructor(
        private template: {
            root: HTMLElement,
            comments: HTMLElement,
            pageStart: HTMLElement,
            pageEnd: HTMLElement,
            total: HTMLElement,
        },
        private services: {
            api: Api,
        },
    ) {
    }

    enter(): void {
        this.template.root.style.display = '';
        this.fetchComments();
    }

    leave(): void {
        this.template.root.style.display = 'none';
    }

    async fetchComments() {
        const comments = await this.services.api.get<ApiPage<Comment>>('admin/comments');
        this.template.pageStart.textContent = '1';
        this.template.pageEnd.textContent = `${comments.content.length}`;
        this.template.total.textContent = `${comments.content.length + comments.remaining}`;
        comments.content.forEach(comment => {
            const elem = document.createElement('div');
            elem.className = 'box-row';
            elem.textContent = comment.markdown;
            this.template.comments.appendChild(elem);
        });
    }
}
