const webpack = require('webpack');
const path = require('path');
const MiniCssExtractPlugin = require('mini-css-extract-plugin');

const languages = ['en', 'da'];

module.exports = languages.map(language => {
    return {
        entry: {
            embed: './ts/embed.ts',
            count: './ts/count.ts',
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
                {
                    test: /\.(sa|sc|c)ss$/,
                    use: [
                        'style-loader',
                        'css-loader',
                        'sass-loader',
                    ],
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
}).concat([
    {
        entry: {
            dashboard: './ts/dashboard/main.ts',
        },
        module: {
            rules: [
                {
                    test: /\.ts$/,
                    use: 'ts-loader',
                    exclude: /node_modules/,
                },
                {
                    test: /\.(sa|sc|c)ss$/,
                    use: [
                        MiniCssExtractPlugin.loader,
                        'css-loader',
                        'sass-loader',
                    ],
                },
                {
                    test: /\.html$/i,
                    loader: 'html-loader',
                },
            ],
        },
        resolve: {
            extensions: ['.ts']
        },
        plugins: [
            new MiniCssExtractPlugin(),
        ],
    }
]);
