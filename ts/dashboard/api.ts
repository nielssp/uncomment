export interface ApiError {
    status: number;
    statusText: string;
    code: string;
}

export interface ApiPage<T> {
    content: T[];
    remaining: number;
}

export class Api {
    constructor(private baseUrl: string) {
    }

    async handleError(response: Response) {
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
            headers: {
                'Content-Type': 'application/json',
            },
            body: JSON.stringify(data),
        });
        await this.handleError(response);
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
