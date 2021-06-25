require('./dashboard.scss');


const login = document.getElementById('login')!;
const comments = document.getElementById('comments')!;

fetch('/auth').then(response => {
    if (response.ok) {
        comments.style.display = 'initial';
        fetch('/admin/comments').then(async response => {
            if (response.ok) {
                console.log(await response.json());
            }
        });
    } else {
        login.style.display = 'initial';
    }
});

const loginForm = document.getElementById('login-form')! as HTMLFormElement;

loginForm.addEventListener('submit', async e => {
    e.preventDefault();
    const response = await fetch('/auth', {
        method: 'POST',
        headers: {
            'Content-Type': 'application/json'
        },
        body: JSON.stringify({
            username: loginForm.username.value,
            password: loginForm.password.value,
        }),
    });
});
