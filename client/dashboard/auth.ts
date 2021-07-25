/* Copyright (c) 2021 Niels Sonnich Poulsen (http://nielssp.dk)
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

import { Api } from "./api";
import { Emitter } from "./emitter";

export interface Credentials {
    username: string,
    password: string,
}

export interface User {
    id: number,
    username: string,
    name: string,
    email: string,
    website: string,
    trusted: boolean,
    admin: boolean,
}

export class Auth {
    private _user?: Promise<User>;
    readonly userChange = new Emitter<User|undefined>();

    constructor(
        private api: Api,
    ) {
    }

    get user(): Promise<User> {
        if (!this._user) {
            this._user = this.api.get<User>('auth').then(user => {
                this.userChange.emit(user);
                return user;
            }, e => {
                this.userChange.emit(undefined);
                return Promise.reject(e);
            });
        }
        return this._user;
    }

    get loggedIn(): Promise<boolean> {
        return this.user.then(() => true, () => false);
    }

    async authenticate(credentials: Credentials) {
        this._user = this.api.post<User>('auth', credentials);
        const user = await this._user;
        this.userChange.emit(user);
    }

    reset() {
        this._user = undefined;
        this.userChange.emit(undefined);
    }

    async logOut() {
        try {
            await this.api.delete('auth');
        } finally {
            this.reset();
        }
    }
}
