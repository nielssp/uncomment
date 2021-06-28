import { Api } from "./api";
import { Comments } from "./comments";
import { Login } from "./login";
import { Router } from "./router";
import { createComponent } from "./util";

require('./dashboard.scss');


const api = new Api('');

const router = new Router({
    '': router => createComponent(Login, document.getElementById('login')!, {api, router}),
    'comments': () => createComponent(Comments, document.getElementById('comments')!, {api}),
});

fetch('/auth').then(response => {
    if (response.ok) {
        router.navigate(['comments']);
    } else {
        router.navigate([]);
    }
});
