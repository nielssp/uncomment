/* Copyright (c) 2021 Niels Sonnich Poulsen (http://nielssp.dk)
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

import { initCommentCounts } from './comments';
import { language } from './languages/default';

function initFromScriptTag() {
    const script = document.querySelector('script[data-uncomment]');
    if (!script) {
        throw new Error('Uncomment script tag not found');
    }
    const scriptSrc = script.getAttribute('src');
    if (!scriptSrc) {
        throw new Error('Uncomment script has no src');
    }
    let api = script.getAttribute('data-uncomment-host');
    if (!api) {
        api = scriptSrc.replace(/(\/\/[^\/]+)\/.*$/, '$1');
    }
    initCommentCounts(api);
}

initFromScriptTag();
