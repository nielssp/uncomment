/* Copyright (c) 2021 Niels Sonnich Poulsen (http://nielssp.dk)
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

import { language } from './languages/default';
import { getRelative } from './util';

const mainTemplate = '<form data-bind="newCommentForm"></form><div class="comments" data-bind="comments"></div>';
const formTemplate = `<input type="text" name="name" data-bind="name" placeholder="${language.name}"/><input type="email" name="email" data-bind="email" placeholder="${language.email}"/><input type="url" name="website" data-bind="website" placeholder="${language.website}"/><br/><textarea name="content" data-bind="content" placeholder="${language.comment}" required></textarea><br/><button type="submit">${language.submit}</button>`;
const commentTemplate = `<div class="comment" data-bind="comment"><div class="comment-header"><span class="author" data-bind="author"></span><time data-bind="created"></time></div><div class="comment-body" data-bind="content"></div><div class="comment-actions"><a href="#" data-bind="replyLink">${language.reply}</a></div><form data-bind="replyForm"></form><div class="replies" data-bind="replies"></div></div>`;

function applyTemplate<T extends {}>(target: Element, template: string): T {
    const bindings: any = {};
    target.innerHTML = template;
    target.querySelectorAll('[data-bind]').forEach(elem => {
        bindings[elem.getAttribute('data-bind')!] = elem;
        elem.removeAttribute('data-bind');
    });
    return bindings;
}

interface MainTemplate {
    newCommentForm: HTMLFormElement;
    comments: HTMLElement;
}

interface FormTemplate {
    name: HTMLInputElement;
    email: HTMLInputElement;
    website: HTMLInputElement;
    content: HTMLTextAreaElement;
}

interface CommentTemplate {
    comment: HTMLElement;
    author: HTMLElement;
    created: HTMLTimeElement;
    content: HTMLElement;
    replyLink: HTMLLinkElement;
    replyForm: HTMLFormElement;
    replies: HTMLElement;
}

interface Config {
    target: Element;
    api: string;
    id: string;
    relativeDates: boolean;
    newestFirst: boolean;
    requireName: boolean;
    requireEmail: boolean;
    clickToLoad: boolean;
}

interface Comment {
    id: number;
    name: string;
    website: string;
    html: string;
    created: string;
    created_timestamp: number;
    replies: Comment[];
}

interface NewComment {
    name: string;
    email: string;
    website: string;
    content: string;
}

async function postComment(config: Config, data: NewComment, parentId?: number): Promise<Comment> {
    let url = `${config.api}/comments?t=${config.id}`;
    if (parentId != undefined) {
        url += `&parent_id=${parentId}`;
    }
    const response = await fetch(url, {
        method: 'POST',
        headers: {
            'Content-Type': 'application/json'
        },
        body: JSON.stringify(data),
    });
    if (!response.ok) {
        switch (await response.text()) {
            case 'MISSING_CONTENT':
                alert(language.missingContentError);
                break;
            case 'MISSING_NAME':
                alert(language.missingNameError);
                break;
            case 'MISSING_EMAIL':
                alert(language.missingEmailError);
                break;
            case 'TOO_MANY_COMMENTS':
                alert(language.tooManyCommentsError);
                break;
            default:
                alert(language.unknownError);
                break;
        }
        throw new Error();
    }
    return response.json();
}

function createCommentForm(
    config: Config,
    form: HTMLFormElement,
    parentId: number|undefined,
    onSuccess: (comment: Comment, template: FormTemplate) => void,
) {
    const template: FormTemplate = applyTemplate(form, formTemplate);
    template.name.required = config.requireName;
    template.email.required = config.requireEmail;
    form.onsubmit = async e => {
        e.preventDefault();
        console.log(template);
        const comment = await postComment(config, {
            name: template.name.value,
            email: template.email.value,
            website: template.website.value,
            content: template.content.value,
        }, parentId);
        onSuccess(comment, template);
    };
}

function addCommentToContainer(config: Config, container: Element, comment: Comment, atStart = false) {
    const temp = document.createElement('div');
    const template = applyTemplate<CommentTemplate>(temp, commentTemplate);
    template.comment.id = `comment-${comment.id}`;
    if (!comment.name) {
        comment.name = language.anonymous;
    }
    if (comment.website) {
        const link = document.createElement('a');
        link.textContent = comment.name;
        link.href = comment.website;
        link.rel = 'noopener noreferrer'; 
        template.author.appendChild(link);
    } else {
        template.author.textContent = comment.name;
    }
    const permalink = document.createElement('a');
    const created = new Date(comment.created_timestamp * 1000);
    permalink.textContent = config.relativeDates ? getRelative(created) : language.date(created);
    permalink.href = `#${template.comment.id}`;
    template.created.appendChild(permalink);
    if (config.relativeDates) {
        template.created.title = language.date(created);
    }
    template.created.dateTime = created.toISOString();
    template.content.innerHTML = comment.html;
    let replyFormOpen = false;
    template.replyLink.onclick = e => {
        e.preventDefault();
        if (replyFormOpen) {
            template.replyForm.innerHTML = '';
            replyFormOpen = false;
            template.replyLink.textContent = language.reply;
        } else {
            createCommentForm(config, template.replyForm, comment.id, reply => {
                addCommentToContainer(config, template.replies, reply, config.newestFirst);
                template.replyForm.innerHTML = '';
                replyFormOpen = false;
                template.replyLink.textContent = language.reply;
            });
            replyFormOpen = true;
            template.replyLink.textContent = language.cancel;
        }
    };
    comment.replies.forEach(reply => addCommentToContainer(config, template.replies, reply));
    if (atStart && container.children.length) {
        container.insertBefore(temp.children[0], container.children[0]);
    } else {
        container.appendChild(temp.children[0]);
    }
}

async function loadComments(config: Config, container: Element) {
    try {
        const response = await fetch(`${config.api}/comments?t=${config.id}&newest_first=${config.newestFirst}`);
        if (!response.ok) {
            throw new Error(await response.text());
        }
        const comments: Comment[] = await response.json();
        for (let comment of comments) {
            addCommentToContainer(config, container, comment);
        }
    } catch (error) {
        console.error('Unable to fetch comments', error);
        const description = document.createElement('div');
        description.className = 'uncomment-error';
        description.textContent = language.commentLoadError;
        container.appendChild(description);
        const retry = document.createElement('button');
        retry.textContent = language.loadComments;
        container.appendChild(retry);
        retry.onclick = () => {
            container.innerHTML = '';
            loadComments(config, container);
        };
    }
}

function load(config: Config) {
    config.target.classList.add('uncomment');
    const main = applyTemplate<MainTemplate>(config.target, mainTemplate);
    createCommentForm(config, main.newCommentForm, undefined, (comment, template) => {
        template.content.value = '';
        addCommentToContainer(config, main.comments, comment, config.newestFirst);
    });
    if (config.clickToLoad) {
        const button = document.createElement('button');
        button.textContent = language.loadComments;
        main.comments.appendChild(button);
        button.onclick = () => {
            main.comments.innerHTML = '';
            loadComments(config, main.comments);
        };
    } else {
        loadComments(config, main.comments);
    }
}

function initFromScriptTag() {
    const script = document.querySelector('script[data-uncomment]');
    if (!script) {
        throw new Error('Uncomment script tag not found');
    }
    const scriptSrc = script.getAttribute('src');
    if (!scriptSrc) {
        throw new Error('Uncomment script has no src');
    }
    const api = scriptSrc.replace(/(\/\/[^\/]+)\/.*$/, '$1');
    const targetSelector = script.getAttribute('data-uncomment-target');
    if (!targetSelector) {
        throw new Error('Uncomment script has no target selector');
    }
    const target = document.querySelector(targetSelector);
    if (!target) {
        throw new Error('Uncomment target not found: ' + targetSelector);
    }
    const config: Config = {
        target,
        api,
        id: script.getAttribute('data-uncomment-id') || location.pathname,
        relativeDates: script.getAttribute('data-uncomment-relative-dates') !== 'false',
        newestFirst: script.getAttribute('data-uncomment-newest-first') === 'true',
        requireName: script.getAttribute('data-uncomment-require-name') === 'true',
        requireEmail: script.getAttribute('data-uncomment-require-email') === 'true',
        clickToLoad: script.getAttribute('data-uncomment-click-to-load') === 'true',
    };
    load(config);
}

initFromScriptTag();
