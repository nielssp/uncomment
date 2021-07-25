/* Copyright (c) 2021 Niels Sonnich Poulsen (http://nielssp.dk)
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

import { language } from './languages/default';

export async function countComments(targets: Element[], api: string) {
    const threadNames = targets.map(target => target.getAttribute('data-uncomment-count'))
        .filter(threadName => threadName);
    let counts: Record<string, number> = {};
    if (threadNames.length) {
        const response = await fetch(`${api}/count?t=${threadNames.join(',')}`)
        if (response.ok) {
            counts = await response.json();
        }
    }
    for (let target of targets) {
        const threadName = target.getAttribute('data-uncomment-count');
        if (threadName && counts.hasOwnProperty(threadName)) {
            target.textContent = language.comments(counts[threadName]);
        } else {
            target.textContent = language.comments(0);
        }
    }
}

export function initCommentCounts(api: string) {
    const targets = document.querySelectorAll('[data-uncomment-count]');
    countComments(Array.from(targets), api);
}
