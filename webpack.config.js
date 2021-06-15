const webpack = require('webpack');
const path = require('path');

const languages = ['en', 'da'];

module.exports = languages.map(language => {
    return {
        entry: {
            embed: './ts/embed.ts',
        },
        output: {
            path: path.resolve(__dirname, 'dist/' + language),
        },
        module: {
            rules: [
                {
                    test: /\.ts$/,
                    use: 'ts-loader',
                    exclude: /node_modules/,
                },
            ]
        },
        resolve: {
            extensions: ['.ts']
        },
        plugins: [
            new webpack.NormalModuleReplacementPlugin(
                /languages\/default$/,
                './languages/' + language
            ),
        ],
    };
});
