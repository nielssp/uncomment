
const months = ['Jan', 'Feb', 'Mar', 'Apr', 'May', 'Jun', 'Jul', 'Aug', 'Sep', 'Oct', 'Nov', 'Dec'];

export const language = {
    comments: (n: number) => n === 1 ? '1 comment' : `${n} comments`,
    name: 'Name',
    email: 'Email',
    website: 'Website',
    comment: 'Comment',
    submit: 'Submit',
    reply: 'Reply',
    cancel: 'Cancel',
    anonymous: 'Anonymous',
    loadComments: 'Load comments',
    commentLoadError: 'Comments failed to load',
    missingContentError: 'Comment cannot be empty',
    missingNameError: 'Anonymous comments are not allowed',
    missingEmailError: 'An email is required',
    tooManyCommentsError: 'Too many comments',
    unknownError: 'An unknown error occurred',
    date: (d: Date) => {
        return `${d.getDate()} ${months[d.getMonth()]} ${d.getFullYear()} ${d.getHours()}:${(d.getMinutes() < 10 ? '0' : '') + d.getMinutes()}`;
    },
    minutes: (n: number) => n === 1 ? `a minute ago` : `${n} minutes ago`,
    hours: (n: number) => n === 1 ? `an hour ago` : `${n} hours ago`,
    days: (n: number) => n === 1 ? `yesterday` : `${n} days ago`,
    weeks: (n: number) => n === 1 ? `last week` : `${n} weeks ago`,
    months: (n: number) => n === 1 ? `last month` : `${n} months ago`,
    years: (n: number) => n === 1 ? `last year` : `${n} years ago`,
};
