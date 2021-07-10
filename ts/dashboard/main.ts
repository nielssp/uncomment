/* Copyright (c) 2021 Niels Sonnich Poulsen (http://nielssp.dk)
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

import { Api } from "./api";
import { Auth } from "./auth";
import { ChangePassword } from "./change-password";
import { Comments } from "./comments";
import { Login } from "./login";
import { Menu } from "./menu";
import { Router } from "./router";
import { Threads } from "./threads";
import { Users } from "./users";
import { createComponent } from "./util";

require('./dashboard.scss');

const api = new Api('');

const auth = new Auth(api);

const router = new Router({
    '': router => createComponent(Login, document.getElementById('login')!, {auth, router}),
    'comments': router => createComponent(Comments, document.getElementById('comments')!, {api, router}),
    'threads': router => createComponent(Threads, document.getElementById('threads')!, {api, router}),
    'users': router => createComponent(Users, document.getElementById('users')!, {api, router}),
    'change-password': router => createComponent(ChangePassword, document.getElementById('change-password')!,
        {api, router}),
});

createComponent(Menu, document.getElementById('menu')!, {auth, router});

api.addResponseListener(response => {
    if (response.status === 401) {
        auth.reset();
        router.open([]);
    }
});

auth.loggedIn.then(loggedIn => {
    if (loggedIn) {
        if (!router.restore()) {
            router.navigate(['comments']);
        }
    } else {
        router.open([]);
    }
})
