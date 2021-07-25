/* Copyright (c) 2021 Niels Sonnich Poulsen (http://nielssp.dk)
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

import {language} from './languages/default';

export function getRelative(date: Date) {
    const mins = (new Date().getTime() - date.getTime()) / 60000 | 0;
    if (mins < 60) {
        return language.minutes(mins);
    }
    const hours = mins / 60 | 0;
    if (hours < 24) {
        return language.hours(hours);
    }
    const days = hours / 24 | 0;
    if (days < 7) {
        return language.days(days);
    } else if (days < 31) {
        return language.weeks(days / 7 | 0);
    } else if (days < 366) {
        return language.months(days / 30.44 | 0);
    } else {
        return language.years(days / 365.25 | 0);
    }
}
