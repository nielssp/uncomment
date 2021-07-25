/* Copyright (c) 2021 Niels Sonnich Poulsen (http://nielssp.dk)
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

export interface ApiError {
    status: number;
    statusText: string;
    code: string;
}

export interface ApiPage<T> {
    content: T[];
    remaining: number;
    limit: number;
}

export class Api {
    private listeners: ((response: Response) => void)[] = [];

    constructor(private baseUrl: string) {
    }

    addResponseListener(listener: (response: Response) => void) {
        this.listeners.push(listener);
    }

    async handleError(response: Response) {
        this.listeners.forEach(l => l(response));
        if (response.ok) {
            return;
        }
        try {
            let body = await response.json();
            if (typeof body !== 'string') {
                body = 'UNKNOWN_ERROR';
            }
            throw {
                status: response.status,
                statusText: response.statusText,
                code: body
            };
        } catch (error) {
            throw {
                status: response.status,
                statusText: response.statusText,
                code: 'CONNECTION_ERROR'
            };
        }
    }

    async get<T>(path: string): Promise<T> {
        const url = `${this.baseUrl}/${path}`;
        const response = await fetch(url);
        await this.handleError(response);
        return response.json();
    }

    async post<T>(path: string, data: any): Promise<T> {
        const url = `${this.baseUrl}/${path}`;
        const response = await fetch(url, {
            method: 'POST',
            headers: data instanceof FormData ? {} : {
                'Content-Type': 'application/json',
            },
            body: data instanceof FormData ? data : JSON.stringify(data),
        });
        await this.handleError(response);
        if (response.status === 204) {
            return undefined as any;
        }
        return response.json();
    }

    async put<T>(path: string, data: any): Promise<T> {
        const url = `${this.baseUrl}/${path}`;
        const response = await fetch(url, {
            method: 'PUT',
            headers: {
                'Content-Type': 'application/json',
            },
            body: JSON.stringify(data),
        });
        await this.handleError(response);
        if (response.status === 204) {
            return undefined as any;
        }
        return response.json();
    }

    async delete(path: string): Promise<void> {
        const url = `${this.baseUrl}/${path}`;
        const response = await fetch(url, {
            method: 'DELETE',
            headers: {
                'Content-Type': 'application/json',
            },
        });
        await this.handleError(response);
    }
}
