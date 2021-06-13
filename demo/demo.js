const url = 'http://localhost:5000?t=test';

const root = document.getElementById('comments');

function reply(comment, replies) {
  const form = document.createElement('form');
  const name = document.createElement('input');
  form.appendChild(name);
  const content = document.createElement('textarea');
  form.appendChild(content);
  const submit = document.createElement('button');
  submit.textContent = 'Submit';
  submit.type = 'submit';
  form.appendChild(submit);
  replies.parentElement.insertBefore(form, replies);
  form.onsubmit = async function (e) {
    e.preventDefault();
    const response = await fetch(url + '&parent_id=' + comment.id, {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json'
      },
      body: JSON.stringify({
        name: name.value,
        content: content.value,
      }),
    });
    replies.parentElement.removeChild(form);
    addComment(replies, await response.json());
  };
}

function addComment(root, comment) {
  const container = document.createElement('div');
  const name = document.createElement('div');
  name.textContent = `${comment.name} - ${comment.created}`;
  container.appendChild(name);
  const content = document.createElement('div');
  content.innerHTML = comment.html;
  container.appendChild(content);
  const actions = document.createElement('div');
  const replies = document.createElement('div');
  const replyAction = document.createElement('button');
  replyAction.textContent = 'reply';
  replyAction.onclick = function () {
    reply(comment, replies);
  };
  actions.appendChild(replyAction);
  container.appendChild(actions);
  comment.replies.forEach(function (c) {
    addComment(replies, c);
  });
  container.appendChild(replies);
  root.appendChild(container);
}

fetch(url).then(async response => {
  const comments = await response.json();
  comments.forEach(function (c) {
    addComment(root, c);
  });
});

const f = document.getElementById('new-comment');
f.onsubmit = async e => {
  e.preventDefault();
  const response = await fetch(url, {
    method: 'POST',
    headers: {
      'Content-Type': 'application/json'
    },
    body: JSON.stringify({
      name: f.name.value,
      content: f.content.value,
    }),
  });
  addComment(root, await response.json());
};
