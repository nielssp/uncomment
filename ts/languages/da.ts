
const months = ['januar', 'februar', 'marts', 'april', 'maj', 'juni', 'juli', 'august', 'september', 'oktober', 'november', 'december'];

export const language = {
    comments: (n: number) => n === 1 ? '1 kommentar' : `${n} kommentarer`,
    name: 'Navn',
    email: 'Email',
    website: 'Website',
    comment: 'Kommentar',
    submit: 'Send',
    reply: 'Svar',
    cancel: 'Annullér',
    anonymous: 'Anonym',
    pendingReview: 'Afventer godkendelse',
    loadComments: 'Hent kommentarer',
    commentLoadError: 'Kommentarerne kunne ikke indlæses',
    missingContentError: 'Kommentaren kan ikke være tom',
    missingNameError: 'Anonyme kommentarer er ikke tilladt',
    missingEmailError: 'En email er nødvendig',
    tooManyCommentsError: 'For mange kommentarer',
    unknownError: 'Der opstod en ukendt fejl',
    date: (d: Date) => {
        return `${d.getDate()}. ${months[d.getMonth()]} ${d.getFullYear()} ${d.getHours()}:${(d.getMinutes() < 10 ? '0' : '') + d.getMinutes()}`;
    },
    minutes: (n: number) => n === 1 ? `et minut siden` : `${n} minutter siden`,
    hours: (n: number) => n === 1 ? `en time siden` : `${n} timer siden`,
    days: (n: number) => n === 1 ? `i går` : `${n} dage siden`,
    weeks: (n: number) => n === 1 ? `sidste uge` : `${n} uger siden`,
    months: (n: number) => n === 1 ? `sidste månded` : `${n} måneder siden`,
    years: (n: number) => n === 1 ? `sidste år` : `${n} år siden`,
};
