import { Api } from "./api";
import { Auth } from "./auth";
import { ChangePassword } from "./change-password";
import { Comments } from "./comments";
import { Login } from "./login";
import { Menu } from "./menu";
import { Router } from "./router";
import { createComponent } from "./util";

require('./dashboard.scss');

const api = new Api('');

const auth = new Auth(api);

const router = new Router({
    '': router => createComponent(Login, document.getElementById('login')!, {auth, router}),
    'comments': () => createComponent(Comments, document.getElementById('comments')!, {api}),
    'change-password': router => createComponent(ChangePassword, document.getElementById('change-password')!,
        {api, router}),
});

createComponent(Menu, document.getElementById('menu')!, {auth, router});

auth.loggedIn.then(loggedIn => {
    if (loggedIn) {
        router.navigate(['comments']);
    } else {
        router.navigate([]);
    }
})
