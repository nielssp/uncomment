const url = 'http://localhost:5000';

const container = document.getElementById('comments');

function addComment(comment) {
  const box = document.createElement('p');
  box.textContent = `${comment.name}: ${comment.content}`;
  container.appendChild(box);
}

fetch(url).then(async response => {
  const comments = await response.json();
  comments.forEach(addComment);
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
  addComment(await response.json());
};
