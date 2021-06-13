const mainTemplate = '<form data-bind="newCommentForm"></form><div class="comments" data-bind="comments"></div>';
const formTemplate = '<input type="text" name="name" placeholder="Name"/><br/><textarea name="content" placeholder="Comment"></textarea><br/><button type="submit">Submit</button>';
const commentTemplate = `<div class="comment"><div class="comment-header"><span class="author" data-bind="author"></span><time data-bind="created"></time></div><div class="comment-body" data-bind="content"></div><div class="comment-actions"><a href="#" data-bind="replyLink">Reply</a></div><form data-bind="replyForm"></form><div class="replies" data-bind="replies"></div></div>`;

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
}

interface CommentTemplate {
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
}

interface Comment {
    id: number;
    name: string;
    html: string;
    created: string;
    replies: Comment[];
}

interface NewComment {
    name: string;
    content: string;
}

async function postComment(config: Config, data: NewComment, parentId?: number): Promise<Comment> {
    let url = `${config.api}?t=${config.id}`;
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
        throw new Error();
    }
    return response.json();
}

function addCommentToContainer(config: Config, container: Element, comment: Comment) {
    const temp = document.createElement('div');
    const template = applyTemplate<CommentTemplate>(temp, commentTemplate);
    template.author.textContent = comment.name;
    template.created.textContent = comment.created;
    template.content.innerHTML = comment.html;
    let replyFormOpen = false;
    template.replyLink.addEventListener('click', e => {
        e.preventDefault();
        if (replyFormOpen) {
            template.replyForm.innerHTML = '';
            replyFormOpen = false;
        } else {
            applyTemplate(template.replyForm, formTemplate);
            replyFormOpen = true;
        }
    });
    template.replyForm.addEventListener('submit', async e => {
        e.preventDefault();
        const reply = await postComment(config, {
            name: (template.replyForm.name as any).value,
            content: template.replyForm.content.value,
        }, comment.id);
        addCommentToContainer(config, template.replies, reply);
        template.replyForm.innerHTML = '';
        replyFormOpen = false;
    });
    comment.replies.forEach(reply => addCommentToContainer(config, template.replies, reply));
    for (let i = 0; i < temp.children.length; i++) {
        container.appendChild(temp.children[i]);
    }
}

async function loadComments(config: Config, container: Element) {
    const response = await fetch(`${config.api}?t=${config.id}`);
    if (!response.ok) {
        // TODO: error message
        return;
    }
    const comments: Comment[] = await response.json();
    for (let comment of comments) {
        addCommentToContainer(config, container, comment);
    }
}

function load(config: Config) {
    config.target.classList.add('uncomment');
    const main = applyTemplate<MainTemplate>(config.target, mainTemplate);
    applyTemplate<FormTemplate>(main.newCommentForm, formTemplate);
    main.newCommentForm.addEventListener('submit', async e => {
        e.preventDefault();
        const comment = await postComment(config, {
            name: (main.newCommentForm.name as any).value,
            content: main.newCommentForm.content.value,
        });
        addCommentToContainer(config, main.comments, comment);
    });
    loadComments(config, main.comments);
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
    };
    load(config);
}

initFromScriptTag();
