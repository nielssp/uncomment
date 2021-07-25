/* Copyright (c) 2021 Niels Sonnich Poulsen (http://nielssp.dk)
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

export type Observer<T> = (event: T) => any;

export class Emitter<T> {
    private observers: Observer<T>[] = [];

    emit(event: T): void {
        for (let observer of this.observers) {
            if (observer(event) === false) {
                return;
            }
        }
    }

    observe(observer: Observer<T>): () => void {
        this.observers.push(observer);
        return () => this.unobserve(observer);
    }

    unobserve(observer: Observer<T>): void {
        this.observers = this.observers.filter(o => o !== observer);
    }
}
